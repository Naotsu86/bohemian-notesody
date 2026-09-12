<script setup>
import { computed, ref } from 'vue'

const props = defineProps({
  note: { type: Object, required: true },
  notes: { type: Array, required: true },
  selectedId: { type: String, default: null },
  depth: { type: Number, default: 0 }
})

const emit = defineEmits(['select', 'add-child'])
const expanded = ref(true)
const children = computed(() => props.notes.filter(n => n.parent_id === props.note.id))
</script>

<template>
  <div class="tree-node-wrap">
    <div
      class="tree-node"
      :class="{ selected: selectedId === note.id, 'root-tree-node': depth === 0 }"
      :style="{ paddingLeft: `${6 + depth * 14}px` }"
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

      <span
        v-if="depth === 0"
        class="tree-root-color"
        :style="{ backgroundColor: note.color || '#0284c7' }"
      ></span>
      <span v-else class="tree-note-icon">▤</span>

      <span class="tree-note-title">{{ note.title || 'Ohne Titel' }}</span>

      <button
        class="tree-add-child"
        type="button"
        title="Unternotiz anlegen"
        @click.stop="emit('add-child', note)"
      >
        +
      </button>
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
      />
    </div>
  </div>
</template>
