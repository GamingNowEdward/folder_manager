<script setup lang="ts">
import type { Folder } from '../types'

const props = defineProps<{
  folder: Folder
  index: number
  selected: boolean
}>()

const emit = defineEmits<{
  open: [path: string]
  copy: [path: string]
  edit: [name: string]
  delete: [name: string]
  mousedown: [index: number, e: MouseEvent]
}>()

function onMouseDown(e: MouseEvent): void {
  if ((e.target as HTMLElement).closest('button')) return
  e.preventDefault()
  emit('mousedown', props.index, e)
}
</script>

<template>
  <div
    class="folder-card"
    :class="{ selected }"
    :data-folder-index="index"
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
    <div class="card-actions">
      <button class="action-btn" title="打开" @click="emit('open', folder.path)">▶</button>
      <button class="action-btn" title="复制路径" @click="emit('copy', folder.path)">■</button>
      <button class="action-btn" title="编辑" @click="emit('edit', folder.name)">✎</button>
      <button class="action-btn action-danger" title="删除" @click="emit('delete', folder.name)">✕</button>
    </div>
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
  transition: transform 0.15s, box-shadow 0.15s, border-color 0.15s, background 0.15s;
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
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.12s;
}

.folder-card:hover .card-actions {
  opacity: 1;
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
