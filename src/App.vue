<script setup>
import { computed, nextTick, onMounted, ref } from 'vue'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { supabase } from './supabase'
import NoteTreeNode from './components/NoteTreeNode.vue'
import TaskTreeNode from './components/TaskTreeNode.vue'
import RichNoteEditor from './components/RichNoteEditor.vue'

const isTauri = '__TAURI_INTERNALS__' in window
const tauriWindow = isTauri ? getCurrentWindow() : null
const TEST_GROUP_ID = '338864c1-1cd8-4a83-a0a4-975c9a5e432a'

const activeView = ref('notes')
const compact = ref(false)
const compactMenuOpen = ref(false)
const quickNote = ref('')
const compactQuickNote = ref('')

const sidebarWidth = ref(Number(localStorage.getItem('notesody-sidebar-width')) || 320)
const resizingSidebar = ref(false)

const authReady = ref(false)
const session = ref(null)
const loginEmail = ref('boehm.alex@gmx.de')
const loginPassword = ref('')
const loginError = ref('')
const authBusy = ref(false)
const authMode = ref('login')
const signupName = ref('')
const profileOpen = ref(false)
const profileBusy = ref(false)
const profileError = ref('')
const profileMessage = ref('')
const profileName = ref('')
const profileEmail = ref('')
const myGroups = ref([])

const notesBusy = ref(false)
const noteError = ref('')
const notes = ref([])
const selectedNote = ref(null)
const editTitle = ref('')
const editContent = ref('')
const editColor = ref('#0284c7')
const editBusy = ref(false)
const createBusy = ref(false)

const taskItems = ref([])
const taskDetails = ref([])
const groupProfiles = ref([])
const selectedTaskItem = ref(null)
const taskTitle = ref('')
const taskContent = ref('')
const taskAssignedTo = ref('')
const taskDueAt = ref('')
const taskStatus = ref('open')
const taskResult = ref('')
const taskBusy = ref(false)
const taskError = ref('')
const taskTopicColor = ref('#0284c7')
const taskMap = computed(() => Object.fromEntries(taskDetails.value.map(t => [t.item_id, t])))
const taskTopics = computed(() => taskItems.value.filter(item => !item.parent_id).sort(sortNotes))

const rootNotes = computed(() => notes.value.filter(note => !note.parent_id).sort(sortNotes))

function sortNotes(a, b) {
  const ao = Number.isFinite(Number(a.sort_order)) ? Number(a.sort_order) : 999999
  const bo = Number.isFinite(Number(b.sort_order)) ? Number(b.sort_order) : 999999
  if (ao !== bo) return ao - bo
  return new Date(a.created_at) - new Date(b.created_at)
}

function startSidebarResize(event) {
  if (window.innerWidth <= 800) return
  resizingSidebar.value = true
  event.preventDefault()
  document.body.classList.add('sidebar-resizing')

  const onMove = (moveEvent) => {
    sidebarWidth.value = Math.min(520, Math.max(240, moveEvent.clientX))
  }

  const onUp = () => {
    resizingSidebar.value = false
    localStorage.setItem('notesody-sidebar-width', String(sidebarWidth.value))
    document.body.classList.remove('sidebar-resizing')
    window.removeEventListener('pointermove', onMove)
    window.removeEventListener('pointerup', onUp)
  }

  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', onUp)
}

const sections = {
  notes: { title: 'Notizen', description: 'Gedanken, Informationen und Dokumentationen festhalten.' },
  tasks: { title: 'Aufgaben', description: 'Eigene und gemeinsame pendente Aktivitäten verwalten.' },
  processes: { title: 'Prozesse', description: 'Abläufe grafisch darstellen und dokumentieren.' }
}

function formatDate(value) {
  if (!value) return ''
  return new Date(value).toLocaleString('de-DE')
}

function noteTitleFromText(text) {
  const firstLine = text.trim().split(/\r?\n/)[0] || 'Schnellnotiz'
  return firstLine.length > 60 ? `${firstLine.slice(0, 57)}...` : firstLine
}

function textToHtml(text) {
  const escaped = text
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
  return escaped.split(/\r?\n/).map(line => `<p>${line || '<br>'}</p>`).join('')
}

function plainPreview(html) {
  if (!html) return ''
  const div = document.createElement('div')
  div.innerHTML = html
  return (div.textContent || div.innerText || '').trim().slice(0, 130)
}

async function login() {
  loginError.value = ''
  authBusy.value = true
  const { data, error } = await supabase.auth.signInWithPassword({
    email: loginEmail.value.trim(),
    password: loginPassword.value
  })
  authBusy.value = false
  if (error) {
    loginError.value = error.message
    return
  }
  session.value = data.session
  loginPassword.value = ''
  await loadNotes()
}

async function signup() {
  loginError.value = ''
  authBusy.value = true
  const email = loginEmail.value.trim()
  const { data, error } = await supabase.auth.signUp({
    email,
    password: loginPassword.value,
    options: { data: { name: signupName.value.trim() } }
  })
  authBusy.value = false
  if (error) { loginError.value = error.message; return }
  if (data.user) {
    await supabase.from('profiles').upsert({ id: data.user.id, name: signupName.value.trim() || email, email }, { onConflict: 'id' })
  }
  if (data.session) { session.value = data.session; await loadNotes() }
  else { loginError.value = 'Registrierung angelegt. Bitte bestätige ggf. die E-Mail und melde dich anschließend an.'; authMode.value = 'login' }
}

const profileInitials = computed(() => {
  const source = profileName.value || session.value?.user?.email || 'BN'
  const parts = source.trim().split(/\s+/).filter(Boolean)
  return (parts.length > 1 ? parts[0][0] + parts[parts.length - 1][0] : source.slice(0, 2)).toUpperCase()
})

async function loadMyProfile() {
  if (!session.value?.user) return
  profileError.value = ''; profileMessage.value = ''
  const uid = session.value.user.id
  const { data: profile, error } = await supabase.from('profiles').select('id,name,email').eq('id', uid).maybeSingle()
  if (error) profileError.value = error.message
  profileName.value = profile?.name || session.value.user.user_metadata?.name || ''
  profileEmail.value = profile?.email || session.value.user.email || ''
  const { data: memberships, error: memberError } = await supabase.from('group_members').select('group_id,role').eq('user_id', uid)
  if (memberError) { profileError.value = memberError.message; myGroups.value = []; return }
  const ids = (memberships || []).map(m => m.group_id)
  if (!ids.length) { myGroups.value = []; return }
  const { data: groups } = await supabase.from('groups').select('id,name').in('id', ids)
  myGroups.value = (memberships || []).map(m => ({ ...m, name: (groups || []).find(g => g.id === m.group_id)?.name || 'Gruppe' }))
}

async function openProfile() { profileOpen.value = true; await loadMyProfile() }

async function saveProfile() {
  if (!session.value?.user) return
  profileBusy.value = true; profileError.value = ''; profileMessage.value = ''
  const { error } = await supabase.from('profiles').upsert({
    id: session.value.user.id, name: profileName.value.trim(), email: session.value.user.email
  }, { onConflict: 'id' })
  profileBusy.value = false
  if (error) { profileError.value = error.message; return }
  profileEmail.value = session.value.user.email || profileEmail.value
  profileMessage.value = 'Profil gespeichert.'
  await loadGroupProfiles()
}

async function logout() {
  await supabase.auth.signOut()
  session.value = null
  notes.value = []
  selectedNote.value = null
}

async function loadNotes(preferredId = selectedNote.value?.id) {
  if (!session.value?.user) return
  notesBusy.value = true
  noteError.value = ''

  const { data, error } = await supabase
    .from('items')
    .select('id,title,content,color,created_at,updated_at,created_by,group_id,parent_id,sort_order')
    .eq('type', 'note')
    .eq('group_id', TEST_GROUP_ID)
    .order('created_at', { ascending: true })

  notesBusy.value = false
  if (error) {
    noteError.value = error.message
    return
  }

  notes.value = data ?? []
  if (preferredId) {
    const refreshed = notes.value.find(note => note.id === preferredId)
    if (refreshed) selectNote(refreshed)
  }
}

async function saveQuickNote() {
  const cleanText = quickNote.value.trim()
  if (!cleanText || !session.value?.user) return

  notesBusy.value = true
  noteError.value = ''
  const now = new Date().toISOString()
  const { data, error } = await supabase
    .from('items')
    .insert({
      type: 'note',
      parent_id: null,
      title: noteTitleFromText(cleanText),
      content: textToHtml(cleanText),
      created_by: session.value.user.id,
      group_id: TEST_GROUP_ID,
      created_at: now,
      updated_at: now,
      sort_order: nextSortOrder(null)
    })
    .select('id')
    .single()

  notesBusy.value = false
  if (error) {
    noteError.value = error.message
    return
  }

  quickNote.value = ''
  await loadNotes(data.id)
}

async function saveCompactQuickNote() {
  quickNote.value = compactQuickNote.value
  await saveQuickNote()
  if (!noteError.value) compactQuickNote.value = ''
}

function selectNote(note) {
  selectedNote.value = note
  editTitle.value = note.title || ''
  editContent.value = note.content || '<p></p>'
  editColor.value = note.parent_id ? '#0284c7' : (note.color || '#0284c7')
}

async function createNote(parent = null) {
  if (!session.value?.user) return
  createBusy.value = true
  noteError.value = ''
  const now = new Date().toISOString()
  const { data, error } = await supabase
    .from('items')
    .insert({
      type: 'note',
      parent_id: parent?.id ?? null,
      title: parent ? 'Neue Unternotiz' : 'Neue Notiz',
      content: '<p></p>',
      color: parent ? null : '#0284c7',
      created_by: session.value.user.id,
      group_id: TEST_GROUP_ID,
      created_at: now,
      updated_at: now,
      sort_order: nextSortOrder(parent?.id ?? null)
    })
    .select('id')
    .single()
  createBusy.value = false

  if (error) {
    noteError.value = error.message
    return
  }
  await loadNotes(data.id)
}

function nextSortOrder(parentId) {
  const siblings = notes.value.filter(note => (note.parent_id ?? null) === (parentId ?? null))
  if (!siblings.length) return 10
  return Math.max(...siblings.map(note => Number(note.sort_order) || 0)) + 10
}

function descendantIds(noteId) {
  const ids = [noteId]
  for (const child of notes.value.filter(note => note.parent_id === noteId)) {
    ids.push(...descendantIds(child.id))
  }
  return ids
}

async function deleteNote(note = selectedNote.value) {
  if (!note) return
  const descendants = descendantIds(note.id)
  const childCount = descendants.length - 1
  const message = childCount
    ? `„${note.title || 'Ohne Titel'}“ und ${childCount} Unter${childCount === 1 ? 'notiz' : 'notizen'} wirklich löschen?\n\nDieser Vorgang kann nicht rückgängig gemacht werden.`
    : `„${note.title || 'Ohne Titel'}“ wirklich löschen?\n\nDieser Vorgang kann nicht rückgängig gemacht werden.`
  if (!window.confirm(message)) return

  editBusy.value = true
  noteError.value = ''
  const { error } = await supabase.from('items').delete().in('id', descendants)
  editBusy.value = false
  if (error) {
    noteError.value = error.message
    return
  }

  if (selectedNote.value && descendants.includes(selectedNote.value.id)) {
    selectedNote.value = null
    editTitle.value = ''
    editContent.value = ''
  }
  await loadNotes(null)
}

async function reorderNote({ draggedId, targetId, position }) {
  if (!draggedId || !targetId || draggedId === targetId) return
  const dragged = notes.value.find(note => note.id === draggedId)
  const target = notes.value.find(note => note.id === targetId)
  if (!dragged || !target) return

  // Nur innerhalb derselben Ebene sortieren. So wird die Hierarchie nicht versehentlich verändert.
  if ((dragged.parent_id ?? null) !== (target.parent_id ?? null)) return

  const parentId = dragged.parent_id ?? null
  const siblings = notes.value.filter(note => (note.parent_id ?? null) === parentId).sort(sortNotes)
  const withoutDragged = siblings.filter(note => note.id !== draggedId)
  const targetIndex = withoutDragged.findIndex(note => note.id === targetId)
  if (targetIndex < 0) return
  const insertIndex = position === 'after' ? targetIndex + 1 : targetIndex
  withoutDragged.splice(insertIndex, 0, dragged)

  // In 10er-Schritten speichern, damit die Reihenfolge eindeutig und gruppenweit gleich ist.
  const updates = withoutDragged.map((note, index) => ({ id: note.id, sort_order: (index + 1) * 10 }))
  notes.value = notes.value.map(note => {
    const update = updates.find(item => item.id === note.id)
    return update ? { ...note, sort_order: update.sort_order } : note
  })

  noteError.value = ''
  for (const update of updates) {
    const { error } = await supabase.from('items').update({ sort_order: update.sort_order }).eq('id', update.id)
    if (error) {
      noteError.value = error.message
      await loadNotes(selectedNote.value?.id)
      return
    }
  }
}

async function updateNote() {
  if (!selectedNote.value) return
  const cleanTitle = editTitle.value.trim() || 'Ohne Titel'
  editBusy.value = true
  noteError.value = ''

  const { error } = await supabase
    .from('items')
    .update({
      title: cleanTitle,
      content: editContent.value || '<p></p>',
      color: selectedNote.value.parent_id ? null : editColor.value,
      updated_at: new Date().toISOString()
    })
    .eq('id', selectedNote.value.id)

  editBusy.value = false
  if (error) {
    noteError.value = error.message
    return
  }
  await loadNotes(selectedNote.value.id)
}


async function loadTasks(preferredId = selectedTaskItem.value?.id) {
  if (!session.value?.user) return
  taskBusy.value = true
  taskError.value = ''
  const { data: itemsData, error: itemsError } = await supabase.from('items')
    .select('id,title,content,color,created_at,updated_at,created_by,group_id,parent_id,sort_order')
    .eq('type','task').eq('group_id',TEST_GROUP_ID).order('created_at',{ascending:true})
  if (itemsError) { taskError.value=itemsError.message; taskBusy.value=false; return }
  taskItems.value = itemsData ?? []
  const ids = taskItems.value.map(i=>i.id)
  if (ids.length) {
    const { data: details, error } = await supabase.from('tasks').select('item_id,assigned_to,status,due_at,result,completed_at,completed_by').in('item_id',ids)
    if (error) taskError.value=error.message
    else taskDetails.value=details ?? []
  } else taskDetails.value=[]
  await loadGroupProfiles()
  taskBusy.value=false
  if (preferredId) { const found=taskItems.value.find(i=>i.id===preferredId); if(found) selectTask(found) }
}

async function loadGroupProfiles() {
  const { data: members, error } = await supabase.from('group_members').select('user_id').eq('group_id',TEST_GROUP_ID)
  if (error) return
  const ids=(members??[]).map(m=>m.user_id)
  if (!ids.length) { groupProfiles.value=[]; return }
  const { data } = await supabase.from('profiles').select('id,name,email').in('id',ids).order('name')
  groupProfiles.value=data??[]
}

function selectTask(item) {
  selectedTaskItem.value=item
  const detail=taskMap.value[item.id] || {}
  taskTitle.value=item.title||''
  taskContent.value=item.content||'<p></p>'
  taskAssignedTo.value=detail.assigned_to||''
  taskDueAt.value=detail.due_at ? new Date(detail.due_at).toISOString().slice(0,10) : ''
  taskStatus.value=detail.status||'open'
  taskResult.value=detail.result||''
  taskTopicColor.value=item.color||'#0284c7'
}

async function createTaskTopic() {
  if (!session.value?.user) return
  taskBusy.value=true; taskError.value=''
  const now=new Date().toISOString()
  const {data,error}=await supabase.from('items').insert({type:'task',parent_id:null,title:'Neues Hauptthema',content:'',color:'#0284c7',created_by:session.value.user.id,group_id:TEST_GROUP_ID,created_at:now,updated_at:now,sort_order:nextTaskSortOrder(null)}).select('id').single()
  taskBusy.value=false
  if(error){taskError.value=error.message;return}
  await loadTasks(data.id)
}

async function createTask(topic) {
  if (!session.value?.user || !topic) return
  taskBusy.value=true; taskError.value=''
  const now=new Date().toISOString()
  const {data:item,error:itemError}=await supabase.from('items').insert({type:'task',parent_id:topic.id,title:'Neue Aufgabe',content:'<p></p>',color:null,created_by:session.value.user.id,group_id:TEST_GROUP_ID,created_at:now,updated_at:now,sort_order:nextTaskSortOrder(topic.id)}).select('id').single()
  if(itemError){taskError.value=itemError.message;taskBusy.value=false;return}
  const {error:taskInsertError}=await supabase.from('tasks').insert({item_id:item.id,assigned_to:null,status:'open',due_at:null,result:null,completed_at:null,completed_by:null})
  taskBusy.value=false
  if(taskInsertError){taskError.value=taskInsertError.message;await supabase.from('items').delete().eq('id',item.id);return}
  await loadTasks(item.id)
}

function nextTaskSortOrder(parentId){const siblings=taskItems.value.filter(i=>(i.parent_id??null)===(parentId??null));return siblings.length?Math.max(...siblings.map(i=>Number(i.sort_order)||0))+10:10}

async function saveTask() {
  if(!selectedTaskItem.value) return
  taskBusy.value=true; taskError.value=''
  const now=new Date().toISOString()
  const isTopic=!selectedTaskItem.value.parent_id
  const {error:itemError}=await supabase.from('items').update({title:taskTitle.value.trim()||'Ohne Titel',content:isTopic?'':(taskContent.value||'<p></p>'),color:isTopic?taskTopicColor.value:null,updated_at:now}).eq('id',selectedTaskItem.value.id)
  if(itemError){taskError.value=itemError.message;taskBusy.value=false;return}
  if(!isTopic){
    const done=taskStatus.value==='done'
    const {error}=await supabase.from('tasks').update({assigned_to:taskAssignedTo.value||null,due_at:taskDueAt.value?new Date(taskDueAt.value+'T12:00:00').toISOString():null,status:taskStatus.value,result:taskResult.value||null,completed_at:done?now:null,completed_by:done?session.value.user.id:null}).eq('item_id',selectedTaskItem.value.id)
    if(error){taskError.value=error.message;taskBusy.value=false;return}
  }
  taskBusy.value=false
  await loadTasks(selectedTaskItem.value.id)
}

async function deleteTaskItem(item=selectedTaskItem.value){
  if(!item)return
  const children=taskItems.value.filter(i=>i.parent_id===item.id)
  const msg=children.length?`„${item.title}“ und ${children.length} Aufgabe${children.length===1?'':'n'} wirklich löschen?`:`„${item.title}“ wirklich löschen?`
  if(!window.confirm(msg))return
  const ids=[item.id,...children.map(c=>c.id)]
  taskBusy.value=true;taskError.value=''
  const {error}=await supabase.from('items').delete().in('id',ids)
  taskBusy.value=false
  if(error){taskError.value=error.message;return}
  if(selectedTaskItem.value&&ids.includes(selectedTaskItem.value.id))selectedTaskItem.value=null
  await loadTasks(null)
}

async function reorderTaskTopic({draggedId,targetId,position}){
  const dragged=taskItems.value.find(i=>i.id===draggedId),target=taskItems.value.find(i=>i.id===targetId)
  if(!dragged||!target||(dragged.parent_id??null)!==(target.parent_id??null))return
  const parentId=dragged.parent_id??null
  const siblings=taskItems.value.filter(i=>(i.parent_id??null)===parentId).sort(sortNotes).filter(i=>i.id!==draggedId)
  const idx=siblings.findIndex(i=>i.id===targetId);if(idx<0)return
  siblings.splice(position==='after'?idx+1:idx,0,dragged)
  for(let i=0;i<siblings.length;i++){const order=(i+1)*10;await supabase.from('items').update({sort_order:order}).eq('id',siblings[i].id)}
  await loadTasks(selectedTaskItem.value?.id)
}

async function showNotebook() {
  if (isTauri) {
    try { await invoke('show_launcher') } catch (error) { console.error(error) }
    return
  }
  compact.value = true
  compactMenuOpen.value = true
  await nextTick()
}

async function leaveCompactMode() {
  compact.value = false
  compactMenuOpen.value = false
  await nextTick()
  if (!isTauri || !tauriWindow) return
  await tauriWindow.setAlwaysOnTop(false)
  await tauriWindow.setResizable(true)
  await tauriWindow.setDecorations(true)
  await tauriWindow.setMinSize(new LogicalSize(900, 650))
  await tauriWindow.setMaxSize(null)
  await tauriWindow.setSize(new LogicalSize(1150, 800))
  await tauriWindow.center()
}

async function openSection(section) {
  activeView.value = section
  if (section === 'tasks') await loadTasks()
  compact.value = false
  compactMenuOpen.value = false
}

async function newTask() { activeView.value = 'tasks'; await leaveCompactMode() }
async function newProcess() { activeView.value = 'processes'; await leaveCompactMode() }

onMounted(async () => {
  const { data } = await supabase.auth.getSession()
  session.value = data.session
  authReady.value = true
  if (session.value?.user) { await loadNotes(); await loadMyProfile() }

  supabase.auth.onAuthStateChange(async (_event, nextSession) => {
    session.value = nextSession
    if (nextSession?.user) { await loadNotes(); await loadMyProfile() }
    else notes.value = []
  })
})
</script>

<template>
  <div v-if="!authReady" class="auth-screen">
    <div class="auth-card"><strong>Bohemian Notesody</strong><span>Verbindung zu Supabase wird hergestellt …</span></div>
  </div>

  <div v-else-if="!session" class="auth-screen">
    <form class="auth-card" @submit.prevent="authMode === 'login' ? login() : signup()">
      <div class="brand auth-brand">
        <div class="brand-icon"><span class="brand-spine"></span><span class="brand-line"></span><span class="brand-line"></span></div>
        <div><div class="brand-name">Bohemian Notesody</div><div class="brand-subtitle">{{ authMode === 'login' ? 'Anmelden' : 'Konto erstellen' }}</div></div>
      </div>
      <label v-if="authMode === 'signup'">Name<input v-model="signupName" type="text" autocomplete="name" required></label>
      <label>E-Mail<input v-model="loginEmail" type="email" autocomplete="username" required></label>
      <label>Passwort<input v-model="loginPassword" type="password" autocomplete="current-password" required></label>
      <p v-if="loginError" class="form-error">{{ loginError }}</p>
      <button class="primary-button" type="submit" :disabled="authBusy">{{ authBusy ? 'Bitte warten …' : (authMode === 'login' ? 'Anmelden' : 'Registrieren') }}</button>
      <button class="auth-switch" type="button" @click="authMode = authMode === 'login' ? 'signup' : 'login'; loginError = ''">{{ authMode === 'login' ? 'Noch kein Konto? Registrieren' : 'Schon ein Konto? Anmelden' }}</button>
    </form>
  </div>

  <div v-else class="app-shell" :class="{ compact }">
    <div v-if="compact" class="compact-workspace">
      <div class="compact-wrapper">
        <div v-if="compactMenuOpen" class="compact-menu">
          <div class="compact-menu-header"><div class="compact-brand-icon"><span></span><span></span><span></span></div><div><strong>Bohemian Notesody</strong><span>Schnellzugriff</span></div></div>
          <div class="compact-section">
            <div class="compact-section-title">Schnellnotiz</div>
            <textarea v-model="compactQuickNote" placeholder="Was möchtest du festhalten?" @keydown.ctrl.enter.prevent="saveCompactQuickNote"></textarea>
            <div class="compact-save-row"><span>Strg + Enter</span><button class="primary-button compact-save-button" @click="saveCompactQuickNote">Speichern</button></div>
          </div>
          <div class="compact-actions">
            <button class="compact-action" @click="newTask"><span class="compact-action-icon task">✓</span><div><strong>Neue Aufgabe</strong><span>Pendente Aktivität anlegen</span></div><span class="compact-arrow">›</span></button>
            <button class="compact-action" @click="newProcess"><span class="compact-action-icon process">◇</span><div><strong>Neuer Prozess</strong><span>Ablauf grafisch erstellen</span></div><span class="compact-arrow">›</span></button>
          </div>
          <button class="open-full-app" @click="leaveCompactMode"><span>↗</span>Vollständig öffnen</button>
        </div>
      </div>
    </div>

    <template v-else>
      <header class="topbar">
        <div class="brand">
          <div class="brand-icon"><span class="brand-spine"></span><span class="brand-line"></span><span class="brand-line"></span></div>
          <div><div class="brand-name">Bohemian Notesody</div><div class="brand-subtitle">Notizen · Aufgaben · Prozesse</div></div>
        </div>
        <div class="topbar-actions">
          <button class="icon-button" title="Zum Notizbuch minimieren" @click="showNotebook">▭</button>
          <button class="user-button" title="Mein Profil" @click="openProfile">{{ profileInitials }}</button>
        </div>
      </header>

      <div class="workspace" :style="{ '--sidebar-width': `${sidebarWidth}px` }">
        <aside class="sidebar">
          <nav class="main-nav accordion-nav">
            <div class="nav-accordion" :class="{ open: activeView === 'notes' }">
              <div class="nav-accordion-head">
                <button class="nav-item accordion-trigger" :class="{ active: activeView === 'notes' }" @click="openSection('notes')">
                  <span class="accordion-chevron">{{ activeView === 'notes' ? '⌄' : '›' }}</span>
                  <span class="nav-icon">▤</span>
                  <span>Notizen</span>
                </button>
                <button v-if="activeView === 'notes'" class="accordion-add" type="button" title="Hauptnotiz anlegen" :disabled="createBusy" @click.stop="createNote(null)">+</button>
              </div>

              <div v-if="activeView === 'notes'" class="accordion-body note-tree-sidebar">
                <div v-if="notesBusy" class="tree-empty sidebar-tree-empty">Notizen werden geladen …</div>
                <div v-else-if="rootNotes.length === 0" class="tree-empty sidebar-tree-empty">Noch keine Notizen.</div>
                <div v-else class="note-tree-list sidebar-tree-list">
                  <NoteTreeNode
                    v-for="note in rootNotes"
                    :key="note.id"
                    :note="note"
                    :notes="notes"
                    :selected-id="selectedNote?.id"
                    @select="selectNote"
                    @add-child="createNote"
                    @delete="deleteNote"
                    @reorder="reorderNote"
                  />
                </div>
              </div>
            </div>

            <div class="nav-accordion" :class="{ open: activeView === 'tasks' }">
              <div class="nav-accordion-head">
                <button class="nav-item accordion-trigger" :class="{ active: activeView === 'tasks' }" @click="openSection('tasks')">
                  <span class="accordion-chevron">{{ activeView === 'tasks' ? '⌄' : '›' }}</span><span class="nav-icon">✓</span><span>Aufgaben</span>
                </button>
                <button v-if="activeView === 'tasks'" class="accordion-add" type="button" title="Hauptthema anlegen" @click.stop="createTaskTopic">+</button>
              </div>
              <div v-if="activeView === 'tasks'" class="accordion-body note-tree-sidebar">
                <div v-if="taskBusy" class="tree-empty sidebar-tree-empty">Aufgaben werden geladen …</div>
                <div v-else-if="!taskTopics.length" class="tree-empty sidebar-tree-empty">Noch keine Hauptthemen.</div>
                <TaskTreeNode v-for="topic in taskTopics" :key="topic.id" :topic="topic" :items="taskItems" :task-map="taskMap" :selected-id="selectedTaskItem?.id" @select="selectTask" @add-task="createTask" @delete="deleteTaskItem" @reorder="reorderTaskTopic" />
              </div>
            </div>

            <div class="nav-accordion" :class="{ open: activeView === 'processes' }">
              <button class="nav-item accordion-trigger" :class="{ active: activeView === 'processes' }" @click="openSection('processes')">
                <span class="accordion-chevron">{{ activeView === 'processes' ? '⌄' : '›' }}</span>
                <span class="nav-icon">◇</span>
                <span>Prozesse</span>
              </button>
              <div v-if="activeView === 'processes'" class="accordion-body accordion-placeholder">Prozessbaum folgt</div>
            </div>
          </nav>
          <div class="sidebar-bottom">
            <button class="nav-item muted"><span class="nav-icon">♙</span><span>Gruppen</span></button>
            <button class="nav-item muted" @click="logout"><span class="nav-icon">⇥</span><span>Abmelden</span></button>
          </div>
          <div class="sidebar-resize-handle" title="Seitenleiste breiter oder schmaler ziehen" @pointerdown="startSidebarResize"></div>
        </aside>

        <main class="main-content" :class="{ 'notes-wide': activeView === 'notes' }">
          <div class="page-heading">
            <div><h1>{{ sections[activeView].title }}</h1><p>{{ sections[activeView].description }}</p></div>
            <button v-if="activeView === 'processes'" class="primary-button">+ Neu</button>
          </div>

          <template v-if="activeView === 'notes'">
            <p v-if="noteError" class="form-error">{{ noteError }}</p>

            <section class="notes-workspace-card editor-only-workspace">
              <section class="note-editor-panel">
                <div v-if="!selectedNote" class="editor-welcome">
                  <div class="editor-welcome-icon">▤</div>
                  <h2>Notiz auswählen</h2>
                  <p>Wähle links eine Notiz aus oder lege eine neue Haupt- bzw. Unternotiz an.</p>
                </div>

                <template v-else>
                  <div class="note-editor-heading">
                    <div class="note-editor-title-wrap">
                      <label for="note-title">Titel</label>
                      <input id="note-title" v-model="editTitle" class="note-title-input" type="text" placeholder="Titel der Notiz">
                      <span>Zuletzt geändert: {{ formatDate(selectedNote.updated_at || selectedNote.created_at) }}</span>
                    </div>
                    <div class="note-editor-actions">
                      <button class="secondary-button" type="button" @click="createNote(selectedNote)">+ Unternotiz</button>
                      <button class="danger-button" type="button" :disabled="editBusy" @click="deleteNote(selectedNote)">Löschen</button>
                    </div>
                  </div>

                  <div v-if="!selectedNote.parent_id" class="note-color-row">
                    <span>Farbe der Hauptnotiz</span>
                    <div class="note-color-palette">
                      <button
                        v-for="color in ['#0284c7','#f59e0b','#dc2626','#111827','#94a3b8','#93c5fd','#fde68a','#059669','#4b5563']"
                        :key="color"
                        type="button"
                        class="note-color-dot"
                        :class="{ selected: editColor === color }"
                        :style="{ backgroundColor: color }"
                        :title="color"
                        @click="editColor = color"
                      ></button>
                    </div>
                  </div>

                  <RichNoteEditor v-model="editContent" />

                  <div class="note-save-row">
                    <span class="save-hint">Formatierung, Checkboxen und Hierarchie werden in Supabase gespeichert.</span>
                    <button class="primary-button" type="button" :disabled="editBusy" @click="updateNote">{{ editBusy ? 'Speichern …' : 'Änderungen speichern' }}</button>
                  </div>
                </template>
              </section>
            </section>

            <section v-if="notes.length" class="section-block recent-notes-block">
              <div class="section-title-row"><h2>Zuletzt bearbeitet</h2></div>
              <div class="note-list">
                <article v-for="note in [...notes].sort((a,b) => new Date(b.updated_at) - new Date(a.updated_at)).slice(0, 4)" :key="note.id" class="note-card note-card-clickable" @click="selectNote(note)">
                  <div class="note-marker"></div>
                  <div class="note-body"><strong>{{ note.title }}</strong><p>{{ plainPreview(note.content) }}</p><span>{{ formatDate(note.updated_at || note.created_at) }}</span></div>
                </article>
              </div>
            </section>
          </template>

          <template v-if="activeView === 'tasks'">
            <p v-if="taskError" class="form-error">{{ taskError }}</p>
            <section class="notes-workspace-card editor-only-workspace task-workspace-card">
              <section class="note-editor-panel">
                <div v-if="!selectedTaskItem" class="editor-welcome"><div class="editor-welcome-icon">✓</div><h2>Aufgabe auswählen</h2><p>Wähle links ein Hauptthema oder eine Aufgabe aus. Mit + legst du neue Hauptthemen bzw. Aufgaben an.</p></div>
                <template v-else>
                  <div class="note-editor-heading">
                    <div class="note-editor-title-wrap"><label>Titel</label><input v-model="taskTitle" class="note-title-input" type="text" placeholder="Titel"></div>
                    <div class="note-editor-actions"><button v-if="!selectedTaskItem.parent_id" class="secondary-button" type="button" @click="createTask(selectedTaskItem)">+ Aufgabe</button><button class="danger-button" type="button" @click="deleteTaskItem(selectedTaskItem)">Löschen</button></div>
                  </div>
                  <div v-if="!selectedTaskItem.parent_id" class="task-topic-editor">
                    <p>Dieses Element ist ein Hauptthema und selbst keine Aufgabe.</p>
                    <div class="note-color-row"><span>Farbe des Hauptthemas</span><div class="note-color-palette"><button v-for="color in ['#0284c7','#f59e0b','#dc2626','#111827','#94a3b8','#93c5fd','#fde68a','#059669','#4b5563']" :key="color" type="button" class="note-color-dot" :class="{selected:taskTopicColor===color}" :style="{backgroundColor:color}" @click="taskTopicColor=color"></button></div></div>
                  </div>
                  <template v-else>
                    <div class="task-meta-grid">
                      <label>Zugewiesen an<select v-model="taskAssignedTo"><option value="">Noch niemand</option><option v-for="profile in groupProfiles" :key="profile.id" :value="profile.id">{{ profile.name || profile.email }}</option></select></label>
                      <label>Fällig am<input v-model="taskDueAt" type="date"></label>
                      <label>Status<select v-model="taskStatus"><option value="open">Offen</option><option value="in_progress">In Bearbeitung</option><option value="done">Erledigt</option></select></label>
                    </div>
                    <div class="task-section-label">Aufgabenbeschreibung</div><RichNoteEditor v-model="taskContent" />
                    <label class="task-result-field">Ergebnis<textarea v-model="taskResult" placeholder="Ergebnis oder Rückmeldung zur Aufgabe …"></textarea></label>
                  </template>
                  <div class="note-save-row"><span class="save-hint">Alle Mitglieder der Gruppe sehen denselben aktuellen Stand.</span><button class="primary-button" type="button" :disabled="taskBusy" @click="saveTask">{{ taskBusy ? 'Speichern …' : 'Änderungen speichern' }}</button></div>
                </template>
              </section>
            </section>
          </template>

          <template v-if="activeView === 'processes'">
            <section class="placeholder-card process-preview"><div class="placeholder-icon">◇</div><h2>Prozesseditor</h2><p>Hier bauen wir später deinen kleinen Grafik-Köcher für Prozesse ein.</p></section>
          </template>
        </main>
      </div>
    </template>
    <div v-if="profileOpen" class="modal-backdrop" @click.self="profileOpen = false">
      <section class="note-editor-modal profile-modal">
        <div class="modal-header"><div><h2>Mein Profil</h2><span>Benutzerkonto und Gruppen</span></div><button class="icon-button" type="button" @click="profileOpen = false">×</button></div>
        <div class="profile-avatar">{{ profileInitials }}</div>
        <label>Name<input v-model="profileName" type="text" placeholder="Dein Name"></label>
        <label>E-Mail<input v-model="profileEmail" type="email" disabled><small>Diese Adresse gehört zu deinem Supabase-Login und wird später für Aufgabenzuweisungen verwendet.</small></label>
        <div class="profile-groups"><strong>Meine Gruppen</strong><div v-if="!myGroups.length" class="profile-empty">Noch keiner Gruppe zugeordnet.</div><div v-for="group in myGroups" :key="group.group_id" class="profile-group-row"><span>{{ group.name }}</span><small>{{ group.role || 'Mitglied' }}</small></div></div>
        <p v-if="profileError" class="form-error">{{ profileError }}</p><p v-if="profileMessage" class="form-success">{{ profileMessage }}</p>
        <div class="modal-actions"><button class="secondary-button" type="button" @click="profileOpen = false">Schließen</button><button class="primary-button" type="button" :disabled="profileBusy" @click="saveProfile">{{ profileBusy ? 'Speichern …' : 'Profil speichern' }}</button></div>
      </section>
    </div>
  </div>
</template>
