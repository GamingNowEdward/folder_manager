<script setup lang="ts">
import type { Folder, FolderId } from '@/types'
import FolderItemActions from './FolderItemActions.vue'

const props = defineProps<{
  folder: Folder
  selected: boolean
}>()

const emit = defineEmits<{
  open: [path: string]
  copy: [path: string]
  edit: [id: FolderId]
  delete: [id: FolderId]
  mousedown: [id: FolderId, event: MouseEvent]
}>()

function onMouseDown(event: MouseEvent): void {
  if ((event.target as HTMLElement).closest('button')) return
  event.preventDefault()
  emit('mousedown', props.folder.id, event)
}
</script>

<template>
  <div
    class="folder-row"
    :class="{ selected }"
    :data-folder-id="folder.id"
    @mousedown="onMouseDown"
    @dblclick="emit('open', folder.path)"
    @contextmenu.prevent="emit('copy', folder.path)"
  >
    <span class="row-icon">📂</span>
    <span class="row-name" :title="folder.name">{{ folder.name }}</span>
    <span class="row-path" :title="folder.path">{{ folder.path }}</span>
    <FolderItemActions
      class="row-actions"
      :folder="folder"
      @open="emit('open', $event)"
      @copy="emit('copy', $event)"
      @edit="emit('edit', $event)"
      @delete="emit('delete', $event)"
    />
  </div>
</template>

<style scoped>
.folder-row {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr) minmax(0, 1.6fr) auto;
  align-items: center;
  gap: 10px;
  height: 38px;
  padding: 0 10px;
  border-radius: 8px;
  cursor: grab;
  user-select: none;
  transition:
    background 0.12s,
    box-shadow 0.12s;
}

.folder-row:hover {
  background: rgba(255, 255, 255, 0.05);
}

.folder-row.selected {
  background: rgba(96, 205, 255, 0.1);
  box-shadow: inset 2px 0 0 #60cdff;
}

.row-icon {
  font-size: 13px;
  text-align: center;
}

.row-name {
  min-width: 0;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.85);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-path {
  min-width: 0;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.32);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-actions {
  opacity: 0;
  transition: opacity 0.12s;
}

.folder-row:hover .row-actions {
  opacity: 1;
}
</style>
