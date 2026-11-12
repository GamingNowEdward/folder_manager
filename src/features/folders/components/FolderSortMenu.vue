<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import type { FolderSort, FolderSortField } from '@/shared/model/folder-sort'

const props = defineProps<{
  x: number
  y: number
  sort: FolderSort
}>()

const emit = defineEmits<{
  'select-field': [field: FolderSortField]
  'toggle-direction': []
  close: []
}>()

const MENU_WIDTH = 216
const MENU_HEIGHT = 224

const rootRef = ref<HTMLElement | null>(null)

const options: { field: FolderSortField; label: string }[] = [
  { field: 'custom', label: '自定义排序' },
  { field: 'name', label: '字母排序' },
  { field: 'created', label: '添加日期排序' }
]

const canReverse = computed(() => props.sort.field !== 'custom')

/** 右键点靠近窗口边缘时把菜单收进可视区。 */
const menuStyle = computed(() => {
  const maxLeft = window.innerWidth - MENU_WIDTH - 8
  const maxTop = window.innerHeight - MENU_HEIGHT - 8
  return {
    left: `${Math.max(8, Math.min(props.x, maxLeft))}px`,
    top: `${Math.max(8, Math.min(props.y, maxTop))}px`
  }
})

function close(): void {
  emit('close')
}

function onDocumentMouseDown(event: MouseEvent): void {
  // 右键不应关闭菜单：否则在同一空白处连续右键会先关后开，产生闪烁。
  if (event.button !== 0) return
  if (rootRef.value?.contains(event.target as Node)) return
  close()
}

function onDocumentKeyDown(event: KeyboardEvent): void {
  if (event.key !== 'Escape') return
  event.preventDefault()
  close()
}

function onWindowBlur(): void {
  close()
}

onMounted(() => {
  document.addEventListener('mousedown', onDocumentMouseDown)
  document.addEventListener('keydown', onDocumentKeyDown)
  window.addEventListener('blur', onWindowBlur)
})

onUnmounted(() => {
  document.removeEventListener('mousedown', onDocumentMouseDown)
  document.removeEventListener('keydown', onDocumentKeyDown)
  window.removeEventListener('blur', onWindowBlur)
})
</script>

<template>
  <div ref="rootRef" class="sort-menu" role="menu" :style="menuStyle" @contextmenu.prevent>
    <div class="menu-title">排序方式</div>
    <button
      v-for="option in options"
      :key="option.field"
      type="button"
      class="menu-item"
      :class="{ active: sort.field === option.field }"
      role="menuitemradio"
      :aria-checked="sort.field === option.field"
      @click="emit('select-field', option.field)"
    >
      <span class="menu-check">{{ sort.field === option.field ? '✓' : '' }}</span>
      <span>{{ option.label }}</span>
    </button>

    <div class="menu-divider" />

    <button
      type="button"
      class="menu-item"
      :class="{ disabled: !canReverse }"
      role="menuitemcheckbox"
      :aria-checked="canReverse && sort.direction === 'desc'"
      :disabled="!canReverse"
      :title="canReverse ? '' : '自定义排序的顺序由拖拽决定'"
      @click="emit('toggle-direction')"
    >
      <span class="menu-check">{{ canReverse && sort.direction === 'desc' ? '✓' : '' }}</span>
      <span>反向排列</span>
    </button>
  </div>
</template>

<style scoped>
.sort-menu {
  position: fixed;
  z-index: 900;
  width: 216px;
  padding: 6px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  background: rgba(81, 81, 102, 0.5);
  -webkit-backdrop-filter: blur(28px) saturate(150%);
  backdrop-filter: blur(28px) saturate(150%);
  box-shadow:
    0 14px 40px rgba(0, 0, 0, 0.35),
    inset 0 1px 0 rgba(255, 255, 255, 0.04);
  display: flex;
  flex-direction: column;
  gap: 1px;
  user-select: none;
  animation: sort-menu-in 0.09s ease-out;
}

@keyframes sort-menu-in {
  from {
    opacity: 0;
    transform: translateY(-2px) scale(0.99);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

.menu-title {
  padding: 6px 10px 4px;
  font-size: 11px;
  letter-spacing: 0.3px;
  color: rgba(255, 255, 255, 0.38);
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 10px;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: rgba(255, 255, 255, 0.72);
  font-size: 12.5px;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition:
    background 0.1s,
    color 0.1s;
}

.menu-item:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.07);
  color: rgba(255, 255, 255, 0.92);
}

.menu-item.active {
  background: rgba(96, 205, 255, 0.1);
  color: #60cdff;
}

.menu-item.active:hover:not(:disabled) {
  background: rgba(96, 205, 255, 0.14);
}

.menu-item.disabled {
  opacity: 0.3;
  cursor: default;
}

.menu-check {
  width: 14px;
  flex-shrink: 0;
  font-size: 11px;
  color: #60cdff;
}

.menu-divider {
  height: 1px;
  margin: 5px 6px;
  background: rgba(255, 255, 255, 0.07);
}
</style>
