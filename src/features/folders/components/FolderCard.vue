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
    class="folder-card"
    :class="{ selected }"
    :data-folder-id="folder.id"
    @mousedown="onMouseDown"
    @dblclick="emit('open', folder.path)"
    @contextmenu.prevent="emit('copy', folder.path)"
  >
    <div class="card-top">
      <div class="card-icon">📂</div>
      <div class="card-info">
        <div class="card-name">{{ folder.name }}</div>
        <div class="card-path">{{ folder.path }}</div>
      </div>
    </div>
    <FolderItemActions
      class="card-actions"
      :folder="folder"
      @open="emit('open', $event)"
      @copy="emit('copy', $event)"
      @edit="emit('edit', $event)"
      @delete="emit('delete', $event)"
    />
  </div>
</template>

<style scoped>
.folder-card {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  cursor: grab;
  user-select: none;
  transition:
    transform 0.15s,
    box-shadow 0.15s,
    border-color 0.15s,
    background 0.15s;
}

.folder-card:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.12);
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.25);
}

.card-top {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.card-icon {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: rgba(96, 205, 255, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 17px;
  flex-shrink: 0;
}

.card-info {
  flex: 1;
  min-width: 0;
}

.card-name {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  margin-bottom: 4px;
}

.card-path {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-actions {
  opacity: 0;
  transition: opacity 0.12s;
}

.folder-card:hover .card-actions {
  opacity: 1;
}
</style>
