<script setup lang="ts">
import type { Folder, FolderId } from '@/types'

const props = defineProps<{
  folder: Folder
}>()

const emit = defineEmits<{
  open: [path: string]
  copy: [path: string]
  edit: [id: FolderId]
  delete: [id: FolderId]
}>()

function onOpen(): void {
  emit('open', props.folder.path)
}

function onCopy(): void {
  emit('copy', props.folder.path)
}

function onEdit(): void {
  emit('edit', props.folder.id)
}

function onDelete(): void {
  emit('delete', props.folder.id)
}
</script>

<template>
  <div class="item-actions">
    <button class="action-btn" title="打开" @click="onOpen">▶</button>
    <button class="action-btn" title="复制路径" @click="onCopy">■</button>
    <button class="action-btn" title="编辑" @click="onEdit">✎</button>
    <button class="action-btn action-danger" title="删除" @click="onDelete">✕</button>
  </div>
</template>

<style scoped>
.item-actions {
  display: flex;
  gap: 4px;
}

.action-btn {
  width: 28px;
  height: 28px;
  background: rgba(255, 255, 255, 0.06);
  border: none;
  border-radius: 7px;
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.1s;
  font-family: inherit;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.9);
}

.action-danger:hover {
  background: rgba(255, 153, 164, 0.15);
  color: #ff99a4;
}
</style>
