<script setup lang="ts">
import type { Project } from '../types'

defineProps<{
  projects: Project[]
  currentName: string
}>()

const emit = defineEmits<{
  select: [name: string]
  add: []
  edit: []
  delete: []
}>()
</script>

<template>
  <div class="sidebar">
    <div class="sidebar-label">项目</div>
    <div
      v-for="p in projects"
      :key="p.name"
      class="sidebar-item"
      :class="{ active: p.name === currentName }"
      @click="emit('select', p.name)"
    >
      <span class="item-dot" :class="{ active: p.name === currentName }"></span>
      {{ p.name }}
    </div>
    <div v-if="projects.length === 0" class="sidebar-empty">暂无项目</div>

    <div class="sidebar-actions">
      <div class="sidebar-item" @click="emit('add')"><span class="action-icon">＋</span>新建项目</div>
      <div class="sidebar-item" :class="{ disabled: !currentName }" @click="currentName && emit('edit')"><span class="action-icon">✎</span>编辑</div>
      <div class="sidebar-item danger" :class="{ disabled: !currentName }" @click="currentName && emit('delete')"><span class="action-icon">✕</span>删除</div>
    </div>
  </div>
</template>

<style scoped>
.item-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  flex-shrink: 0;
}

.item-dot.active {
  background: #60cdff;
}

.sidebar-empty {
  padding: 12px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.25);
}

.sidebar-item.disabled {
  opacity: 0.3;
  pointer-events: none;
}

.action-icon {
  width: 18px;
  text-align: center;
  flex-shrink: 0;
}

.sidebar-item.danger:hover {
  background: rgba(255, 153, 164, 0.1);
  color: #ff99a4;
}
</style>
