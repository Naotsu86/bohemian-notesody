<script setup>
import { computed, nextTick, onMounted, ref } from 'vue'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { supabase } from './supabase'
import NoteTreeNode from './components/NoteTreeNode.vue'
import RichNoteEditor from './components/RichNoteEditor.vue'

const isTauri = '__TAURI_INTERNALS__' in window
const tauriWindow = isTauri ? getCurrentWindow() : null
const TEST_GROUP_ID = '338864c1-1cd8-4a83-a0a4-975c9a5e432a'

const activeView = ref('notes')
const compact = ref(false)
const compactMenuOpen = ref(false)
const quickNote = ref('')
const compactQuickNote = ref('')

const authReady = ref(false)
const session = ref(null)
const loginEmail = ref('boehm.alex@gmx.de')
const loginPassword = ref('')
const loginError = ref('')
const authBusy = ref(false)

const notesBusy = ref(false)
const noteError = ref('')
const notes = ref([])
const selectedNote = ref(null)
const editTitle = ref('')
const editContent = ref('')
const editColor = ref('#0284c7')
const editBusy = ref(false)
const createBusy = ref(false)

const rootNotes = computed(() => notes.value.filter(note => !note.parent_id))

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
    .select('id,title,content,color,created_at,updated_at,created_by,group_id,parent_id')
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
      updated_at: now
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
      updated_at: now
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

function openSection(section) {
  activeView.value = section
  compact.value = false
  compactMenuOpen.value = false
}

async function newTask() { activeView.value = 'tasks'; await leaveCompactMode() }
async function newProcess() { activeView.value = 'processes'; await leaveCompactMode() }

onMounted(async () => {
  const { data } = await supabase.auth.getSession()
  session.value = data.session
  authReady.value = true
  if (session.value?.user) await loadNotes()

  supabase.auth.onAuthStateChange(async (_event, nextSession) => {
    session.value = nextSession
    if (nextSession?.user) await loadNotes()
    else notes.value = []
  })
})
</script>

<template>
  <div v-if="!authReady" class="auth-screen">
    <div class="auth-card"><strong>Bohemian Notesody</strong><span>Verbindung zu Supabase wird hergestellt …</span></div>
  </div>

  <div v-else-if="!session" class="auth-screen">
    <form class="auth-card" @submit.prevent="login">
      <div class="brand auth-brand">
        <div class="brand-icon"><span class="brand-spine"></span><span class="brand-line"></span><span class="brand-line"></span></div>
        <div><div class="brand-name">Bohemian Notesody</div><div class="brand-subtitle">Anmelden</div></div>
      </div>
      <label>E-Mail<input v-model="loginEmail" type="email" autocomplete="username" required></label>
      <label>Passwort<input v-model="loginPassword" type="password" autocomplete="current-password" required></label>
      <p v-if="loginError" class="form-error">{{ loginError }}</p>
      <button class="primary-button" type="submit" :disabled="authBusy">{{ authBusy ? 'Anmelden …' : 'Anmelden' }}</button>
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
          <button class="user-button" title="Profil (Dashboard folgt)">AB</button>
        </div>
      </header>

      <div class="workspace">
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
                  />
                </div>
              </div>
            </div>

            <div class="nav-accordion" :class="{ open: activeView === 'tasks' }">
              <button class="nav-item accordion-trigger" :class="{ active: activeView === 'tasks' }" @click="openSection('tasks')">
                <span class="accordion-chevron">{{ activeView === 'tasks' ? '⌄' : '›' }}</span>
                <span class="nav-icon">✓</span>
                <span>Aufgaben</span>
              </button>
              <div v-if="activeView === 'tasks'" class="accordion-body accordion-placeholder">Aufgabenübersicht folgt</div>
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
        </aside>

        <main class="main-content" :class="{ 'notes-wide': activeView === 'notes' }">
          <div class="page-heading">
            <div><h1>{{ sections[activeView].title }}</h1><p>{{ sections[activeView].description }}</p></div>
            <button v-if="activeView !== 'notes'" class="primary-button">+ Neu</button>
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
                    <button class="secondary-button" type="button" @click="createNote(selectedNote)">+ Unternotiz</button>
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
            <section class="placeholder-card"><div class="placeholder-icon">✓</div><h2>Aufgaben & pendente Aktivitäten</h2><p>Hier entstehen später persönliche Aufgaben, Gruppenzuweisungen, Ergebnisse und Statusmeldungen.</p></section>
          </template>

          <template v-if="activeView === 'processes'">
            <section class="placeholder-card process-preview"><div class="placeholder-icon">◇</div><h2>Prozesseditor</h2><p>Hier bauen wir später deinen kleinen Grafik-Köcher für Prozesse ein.</p></section>
          </template>
        </main>
      </div>
    </template>
  </div>
</template>
