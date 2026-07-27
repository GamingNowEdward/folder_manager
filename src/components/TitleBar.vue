<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'

const appWindow = getCurrentWindow()

function onMinimize(): void { appWindow.minimize() }
function onMaximize(): void { appWindow.toggleMaximize() }
function onClose(): void { appWindow.close() }

function onDragMouseDown(e: MouseEvent): void {
  if ((e.target as HTMLElement).closest('.title-bar-buttons')) return
  const startX = e.clientX
  const startY = e.clientY
  let dragged = false

  const onMove = (me: MouseEvent) => {
    if (!dragged && (Math.abs(me.clientX - startX) > 5 || Math.abs(me.clientY - startY) > 5)) {
      dragged = true
      appWindow.startDragging()
    }
  }

  const onUp = () => {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
  }

  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}
</script>

<template>
  <div class="title-bar" @mousedown="onDragMouseDown" @dblclick="onMaximize">
    <div class="title-bar-left">
      <span class="title-icon">📁</span>
      <span class="title-text">Folder Manager</span>
    </div>
    <div class="title-bar-buttons" @dblclick.stop>
      <button class="win-btn" @click="onMinimize" title="最小化">
        <svg width="16" height="16" viewBox="0 0 16 16">
          <line x1="3" y1="8" x2="13" y2="8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
        </svg>
      </button>
      <button class="win-btn" @click="onMaximize" title="最大化">
        <svg width="16" height="16" viewBox="0 0 16 16">
          <rect x="3.5" y="3.5" width="9" height="9" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.2"/>
        </svg>
      </button>
      <button class="win-btn win-close" @click="onClose" title="关闭">
        <svg width="16" height="16" viewBox="0 0 16 16">
          <line x1="4" y1="4" x2="12" y2="12" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
          <line x1="12" y1="4" x2="4" y2="12" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.title-bar {
  display: flex;
  align-items: center;
  height: 38px;
  user-select: none;
  flex-shrink: 0;
  padding: 0 8px 0 16px;
}

.title-bar-left {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-icon {
  font-size: 15px;
}

.title-text {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
}

.title-bar-buttons {
  display: flex;
  gap: 2px;
}

.win-btn {
  width: 36px;
  height: 28px;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.1s;
}

.win-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.85);
}

.win-close:hover {
  background: rgba(196, 43, 28, 0.9);
  color: #fff;
}
</style>
