<script setup>
import { onBeforeUnmount, watch } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Underline from '@tiptap/extension-underline'
import { TextStyle, Color, FontSize } from '@tiptap/extension-text-style'
import { TaskList, TaskItem } from '@tiptap/extension-list'

const props = defineProps({
  modelValue: { type: String, default: '' }
})
const emit = defineEmits(['update:modelValue'])

const editor = useEditor({
  content: props.modelValue || '<p></p>',
  extensions: [
    StarterKit,
    Underline,
    TextStyle,
    Color,
    FontSize,
    TaskList,
    TaskItem.configure({ nested: true })
  ],
  editorProps: {
    attributes: { class: 'rich-editor-content' }
  },
  onUpdate({ editor }) {
    emit('update:modelValue', editor.getHTML())
  }
})

watch(() => props.modelValue, value => {
  if (!editor.value) return
  const html = value || '<p></p>'
  if (editor.value.getHTML() !== html) {
    editor.value.commands.setContent(html, { emitUpdate: false })
  }
})

function setFontSize(event) {
  const size = event.target.value
  if (!size) {
    editor.value.chain().focus().unsetFontSize().run()
    return
  }
  editor.value.chain().focus().setFontSize(size).run()
}

function setColor(event) {
  editor.value.chain().focus().setColor(event.target.value).run()
}

onBeforeUnmount(() => editor.value?.destroy())
</script>

<template>
  <div v-if="editor" class="rich-editor">
    <div class="editor-toolbar">
      <button type="button" :class="{ active: editor.isActive('bold') }" title="Fett" @click="editor.chain().focus().toggleBold().run()"><strong>B</strong></button>
      <button type="button" :class="{ active: editor.isActive('italic') }" title="Kursiv" @click="editor.chain().focus().toggleItalic().run()"><em>I</em></button>
      <button type="button" :class="{ active: editor.isActive('underline') }" title="Unterstreichen" @click="editor.chain().focus().toggleUnderline().run()"><u>U</u></button>
      <button type="button" :class="{ active: editor.isActive('strike') }" title="Durchstreichen" @click="editor.chain().focus().toggleStrike().run()"><s>S</s></button>

      <span class="toolbar-separator"></span>

      <select title="Schriftgröße" @change="setFontSize">
        <option value="">Größe</option>
        <option value="12px">12</option>
        <option value="14px">14</option>
        <option value="16px">16</option>
        <option value="18px">18</option>
        <option value="22px">22</option>
        <option value="28px">28</option>
      </select>

      <label class="color-picker" title="Schriftfarbe">
        A
        <input type="color" value="#1b2330" @input="setColor">
      </label>

      <span class="toolbar-separator"></span>

      <button type="button" :class="{ active: editor.isActive('bulletList') }" title="Aufzählung" @click="editor.chain().focus().toggleBulletList().run()">• Liste</button>
      <button type="button" :class="{ active: editor.isActive('orderedList') }" title="Nummerierte Liste" @click="editor.chain().focus().toggleOrderedList().run()">1. Liste</button>
      <button type="button" :class="{ active: editor.isActive('taskList') }" title="Checkbox-Liste" @click="editor.chain().focus().toggleTaskList().run()">☑ Liste</button>

      <span class="toolbar-separator"></span>

      <button type="button" title="Rückgängig" @click="editor.chain().focus().undo().run()">↶</button>
      <button type="button" title="Wiederholen" @click="editor.chain().focus().redo().run()">↷</button>
    </div>

    <EditorContent :editor="editor" />
  </div>
</template>
