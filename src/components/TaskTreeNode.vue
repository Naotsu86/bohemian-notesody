<script setup>
import { computed, ref } from 'vue'
const props = defineProps({ topic:{type:Object,required:true}, items:{type:Array,required:true}, taskMap:{type:Object,required:true}, selectedId:{type:String,default:null} })
const emit = defineEmits(['select','add-task','delete','reorder'])
const expanded = ref(false)
const dragOverPosition = ref(null)
function sortItems(a,b){ const ao=Number(a.sort_order)||999999, bo=Number(b.sort_order)||999999; return ao!==bo?ao-bo:new Date(a.created_at)-new Date(b.created_at) }
const children = computed(()=>props.items.filter(i=>i.parent_id===props.topic.id).sort(sortItems))
function dragStart(e,item){e.dataTransfer.effectAllowed='move';e.dataTransfer.setData('text/plain',item.id)}
function dragOver(e){e.preventDefault();const r=e.currentTarget.getBoundingClientRect();dragOverPosition.value=e.clientY<r.top+r.height/2?'before':'after'}
function drop(e,target){e.preventDefault();const draggedId=e.dataTransfer.getData('text/plain');if(draggedId&&draggedId!==target.id)emit('reorder',{draggedId,targetId:target.id,position:dragOverPosition.value||'before'});dragOverPosition.value=null}
</script>
<template>
  <div class="task-topic-wrap">
    <div class="tree-node root-tree-node" :class="{'drag-before':dragOverPosition==='before','drag-after':dragOverPosition==='after'}" draggable="true" @dragstart="dragStart($event,topic)" @dragover="dragOver" @dragleave="dragOverPosition=null" @drop="drop($event,topic)">
      <button class="tree-toggle" type="button" @click.stop="expanded=!expanded">{{ expanded ? '⌄' : '›' }}</button>
      <span class="tree-root-color" :style="{backgroundColor:topic.color||'#0284c7'}"></span>
      <span class="tree-note-title" @click="expanded=!expanded">{{ topic.title }}</span>
      <button class="tree-add-child" type="button" title="Aufgabe anlegen" @click.stop="emit('add-task',topic)">+</button>
      <button class="tree-delete" type="button" title="Hauptthema löschen" @click.stop="emit('delete',topic)">×</button>
    </div>
    <div v-if="expanded" class="tree-children task-tree-children">
      <div v-if="!children.length" class="task-tree-empty">Noch keine Aufgaben.</div>
      <div v-for="item in children" :key="item.id" class="tree-node task-tree-node" :class="{selected:selectedId===item.id}" @click="emit('select',item)">
        <span class="task-status-dot" :class="taskMap[item.id]?.status || 'open'">{{ taskMap[item.id]?.status === 'done' ? '✓' : taskMap[item.id]?.status === 'in_progress' ? '◐' : '○' }}</span>
        <span class="tree-note-title">{{ item.title }}</span>
        <button class="tree-delete" type="button" title="Aufgabe löschen" @click.stop="emit('delete',item)">×</button>
      </div>
    </div>
  </div>
</template>
