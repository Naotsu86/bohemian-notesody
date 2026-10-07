<script setup>
import { computed, ref } from 'vue'

const props = defineProps({
  note: { type: Object, required: true },
  notes: { type: Array, required: true },
  selectedId: { type: String, default: null },
  depth: { type: Number, default: 0 }
})

const emit = defineEmits(['select', 'add-child', 'delete', 'reorder'])
// Absichtlich immer geschlossen starten. Der Benutzer öffnet nur die Zweige, die er braucht.
const expanded = ref(false)
const dragOverPosition = ref(null)

function sortNotes(a, b) {
  const ao = Number.isFinite(Number(a.sort_order)) ? Number(a.sort_order) : 999999
  const bo = Number.isFinite(Number(b.sort_order)) ? Number(b.sort_order) : 999999
  if (ao !== bo) return ao - bo
  return new Date(a.created_at) - new Date(b.created_at)
}

const children = computed(() => props.notes.filter(n => n.parent_id === props.note.id).sort(sortNotes))

function onDragStart(event) {
  event.dataTransfer.effectAllowed = 'move'
  event.dataTransfer.setData('text/plain', props.note.id)
}

function onDragOver(event) {
  event.preventDefault()
  event.dataTransfer.dropEffect = 'move'
  const rect = event.currentTarget.getBoundingClientRect()
  dragOverPosition.value = event.clientY < rect.top + rect.height / 2 ? 'before' : 'after'
}

function onDrop(event) {
  event.preventDefault()
  const draggedId = event.dataTransfer.getData('text/plain')
  if (draggedId && draggedId !== props.note.id) {
    emit('reorder', { draggedId, targetId: props.note.id, position: dragOverPosition.value || 'before' })
  }
  dragOverPosition.value = null
}
</script>

<template>
  <div class="tree-node-wrap">
    <div
      class="tree-node"
      :class="{
        selected: selectedId === note.id,
        'root-tree-node': depth === 0,
        'drag-before': dragOverPosition === 'before',
        'drag-after': dragOverPosition === 'after'
      }"
      :style="{ paddingLeft: `${6 + depth * 14}px` }"
      draggable="true"
      @dragstart="onDragStart"
      @dragover="onDragOver"
      @dragleave="dragOverPosition = null"
      @drop="onDrop"
      @dragend="dragOverPosition = null"
      @click="emit('select', note)"
    >
      <button
        v-if="children.length"
        class="tree-toggle"
        type="button"
        :title="expanded ? 'Unterpunkte einklappen' : 'Unterpunkte aufklappen'"
        @click.stop="expanded = !expanded"
      >
        {{ expanded ? '⌄' : '›' }}
      </button>
      <span v-else class="tree-toggle-spacer"></span>

      <span v-if="depth === 0" class="tree-root-color" :style="{ backgroundColor: note.color || '#0284c7' }"></span>
      <span v-else class="tree-note-icon">▤</span>
      <span class="tree-note-title">{{ note.title || 'Ohne Titel' }}</span>

      <button class="tree-add-child" type="button" title="Unternotiz anlegen" @click.stop="emit('add-child', note)">+</button>
      <button class="tree-delete" type="button" title="Notiz löschen" @click.stop="emit('delete', note)">×</button>
    </div>

    <div v-if="expanded && children.length" class="tree-children">
      <NoteTreeNode
        v-for="child in children"
        :key="child.id"
        :note="child"
        :notes="notes"
        :selected-id="selectedId"
        :depth="depth + 1"
        @select="emit('select', $event)"
        @add-child="emit('add-child', $event)"
        @delete="emit('delete', $event)"
        @reorder="emit('reorder', $event)"
      />
    </div>
  </div>
</template>
