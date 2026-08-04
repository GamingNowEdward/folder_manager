<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue'
import { useAppStore } from './stores/app'
import { invoke } from '@tauri-apps/api/core'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { getCurrentWindow } from '@tauri-apps/api/window'
import TitleBar from './components/TitleBar.vue'
import StatusBar from './components/StatusBar.vue'
import FolderCard from './components/FolderCard.vue'
import AddFolderCard from './components/AddFolderCard.vue'
import Sidebar from './components/Sidebar.vue'
import ProjectDialog from './dialogs/ProjectDialog.vue'
import FolderDialog from './dialogs/FolderDialog.vue'
import ConfirmDialog from './dialogs/ConfirmDialog.vue'

const store = useAppStore()
const appWindow = getCurrentWindow()

const showProjectDialog = ref(false)
const projectDialogMode = ref<'add' | 'edit'>('add')
const showFolderDialog = ref(false)
const folderDialogMode = ref<'add' | 'edit'>('add')
const editFolderName = ref('')
const editFolderPath = ref('')
const dropIndicatorIndex = ref(-1)
const selectedFolderNames = ref<string[]>([])
const dragFromIndex = ref(-1)
const dragToIndex = ref(-1)
const isDraggingCard = ref(false)
let dragStartPos = { x: 0, y: 0 }
const dragFolderName = ref('')
const dragGhostX = ref(0)
const dragGhostY = ref(0)
const dragGhostW = ref(0)
const dragGhostH = ref(0)
const dragOffsetX = ref(0)
const dragOffsetY = ref(0)

const dragFolder = computed(() =>
  store.currentFolders.find((f) => f.name === dragFolderName.value) ?? null
)
const isBoxSelecting = ref(false)
const boxStartX = ref(0)
const boxStartY = ref(0)
const boxRect = ref<{ left: number; top: number; width: number; height: number } | null>(null)
const cardsAreaRef = ref<HTMLElement | null>(null)

const showConfirmDialog = ref(false)
const confirmDialogTitle = ref('')
const confirmDialogMessage = ref('')
let confirmCallback: (() => void) | null = null

const boxOverlayStyle = computed(() => {
  if (!boxRect.value || !cardsAreaRef.value) return { display: 'none' }
  const areaRect = cardsAreaRef.value.getBoundingClientRect()
  return {
    left: `${boxRect.value.left - areaRect.left + cardsAreaRef.value.scrollLeft}px`,
    top: `${boxRect.value.top - areaRect.top + cardsAreaRef.value.scrollTop}px`,
    width: `${boxRect.value.width}px`,
    height: `${boxRect.value.height}px`
  }
})

onMounted(() => {
  document.addEventListener('contextmenu', (e) => {
    if (!(e.target as HTMLElement).closest('input, textarea')) {
      e.preventDefault()
    }
  })
  store.loadFromDisk()
  document.addEventListener('mousemove', onWindowMouseMove)
  document.addEventListener('mouseup', onWindowMouseUp)
  document.addEventListener('keydown', onWindowKeyDown)
  window.addEventListener('blur', onWindowBlur)
  appWindow.onDragDropEvent((event) => {
    if (event.payload.type === 'drop' && event.payload.paths?.length > 0) {
      onExternalDrop(event.payload.paths)
    }
  }).then((fn) => { unlistenDragDrop = fn })
})

let unlistenDragDrop: (() => void) | null = null

onUnmounted(() => {
  document.removeEventListener('mousemove', onWindowMouseMove)
  document.removeEventListener('mouseup', onWindowMouseUp)
  document.removeEventListener('keydown', onWindowKeyDown)
  window.removeEventListener('blur', onWindowBlur)
  unlistenDragDrop?.()
})

function onAddProject(): void {
  projectDialogMode.value = 'add'
  showProjectDialog.value = true
}

function onEditProject(): void {
  projectDialogMode.value = 'edit'
  showProjectDialog.value = true
}

function onDeleteProject(): void {
  const name = store.currentProjectName
  if (!name) return
  confirmDialogTitle.value = '删除项目'
  confirmDialogMessage.value = `确定要删除项目「${name}」及其所有文件夹配置吗？\n（不会删除实际文件）`
  confirmCallback = () => {
    store.deleteProject(name)
    store.setStatus(`已删除项目「${name}」`)
  }
  showConfirmDialog.value = true
}

function onProjectDialogConfirm(name: string): void {
  if (projectDialogMode.value === 'add') {
    if (store.projects.some((p) => p.name === name)) {
      store.setStatus(`✕ 项目「${name}」已存在`)
      return
    }
    store.addProject(name)
    store.setStatus(`已创建项目「${name}」`)
  } else {
    const oldName = store.currentProjectName
    if (!oldName) return
    if (name !== oldName && store.projects.some((p) => p.name === name)) {
      store.setStatus(`✕ 项目「${name}」已存在`)
      return
    }
    store.renameProject(oldName, name)
    store.setStatus(`已重命名为「${name}」`)
  }
  showProjectDialog.value = false
}

function onAddFolder(): void {
  folderDialogMode.value = 'add'
  editFolderName.value = ''
  editFolderPath.value = ''
  showFolderDialog.value = true
}

function onEditFolder(name: string, path: string): void {
  folderDialogMode.value = 'edit'
  editFolderName.value = name
  editFolderPath.value = path
  showFolderDialog.value = true
}

function onFolderDialogConfirm(result: { name: string; path: string }): void {
  if (folderDialogMode.value === 'add') {
    if (store.currentFolders.some((f) => f.name === result.name)) {
      store.setStatus(`✕ 文件夹「${result.name}」在此项目中已存在`)
      return
    }
    store.addFolder(result.name, result.path)
    store.setStatus(`已添加文件夹「${result.name}」`)
  } else {
    const oldName = editFolderName.value
    if (result.name !== oldName && store.currentFolders.some((f) => f.name === result.name)) {
      store.setStatus(`✕ 文件夹「${result.name}」在此项目中已存在`)
      return
    }
    store.renameFolder(oldName, result.name, result.path)
    store.setStatus(`已更新文件夹「${result.name}」`)
  }
  showFolderDialog.value = false
}

function onOpenFolder(path: string): void {
  invoke('open_folder', { path }).then(() => {
    store.setStatus(`▶ 已打开文件夹：${path}`)
  }).catch((err) => {
    store.setStatus(`✕ ${err}`)
  })
}

function onCopyPath(path: string): void {
  writeText(path).then(() => {
    store.setStatus(`■ 已复制路径到剪贴板：${path}`)
  }).catch(() => {
    store.setStatus(`✕ 复制路径失败`)
  })
}

function onDeleteFolder(name: string): void {
  confirmDialogTitle.value = '删除文件夹'
  confirmDialogMessage.value = `确定要从此项目中移除文件夹「${name}」吗？\n（不会删除实际文件）`
  confirmCallback = () => {
    store.deleteFolder(name)
    store.setStatus(`已移除文件夹「${name}」`)
  }
  showConfirmDialog.value = true
}

function onExternalDrop(paths: string[]): void {
  if (!store.currentProject) {
    store.setStatus('请先创建或选择一个项目')
    return
  }
  const existing = new Set(store.currentFolders.map((f) => f.name))
  let added = 0
  let skipped = 0
  for (const p of paths) {
    const name = p.replace(/[/\\]+$/, '').split(/[/\\]/).pop() ?? ''
    if (!name || existing.has(name)) {
      if (name) skipped++
      continue
    }
    store.addFolder(name, p)
    existing.add(name)
    added++
  }
  if (added > 0 && skipped > 0) {
    store.setStatus(`▶ 已添加 ${added} 个文件夹，${skipped} 个因重名跳过`)
  } else if (added > 0) {
    store.setStatus(`▶ 已添加 ${added} 个文件夹`)
  } else if (skipped > 0) {
    store.setStatus(`✕ ${skipped} 个文件夹因重名被跳过`)
  }
}

function getDropTargetIndex(e: { clientX: number; clientY: number }): number {
  const allCards = document.querySelectorAll('[data-folder-index]')
  const cards: Element[] = []
  allCards.forEach((card) => {
    const n = card.getAttribute('data-folder-name')
    if (n !== dragFolderName.value) cards.push(card)
  })
  if (cards.length === 0) return 0

  const mx = e.clientX
  const my = e.clientY
  const threshold = 5

  const rows: number[][] = []
  const rowTops: number[] = []
  const rowBottoms: number[] = []

  for (let i = 0; i < cards.length; i++) {
    const r = cards[i].getBoundingClientRect()
    const top = Math.round(r.top)
    let rowIdx = rowTops.findIndex((t) => Math.abs(t - top) < threshold)
    if (rowIdx < 0) {
      rowIdx = rows.length
      rows.push([])
      rowTops.push(r.top)
      rowBottoms.push(r.bottom)
    }
    rows[rowIdx].push(i)
    rowBottoms[rowIdx] = Math.max(rowBottoms[rowIdx], r.bottom)
  }

  for (const row of rows) {
    row.sort((a, b) => cards[a].getBoundingClientRect().left - cards[b].getBoundingClientRect().left)
  }

  const visualOrder = rows.flat()

  for (let vi = 0; vi < visualOrder.length; vi++) {
    const idx = visualOrder[vi]
    const rect = cards[idx].getBoundingClientRect()
    if (my >= rect.top && my <= rect.bottom) {
      if (mx < rect.left + rect.width / 2) {
        return parseInt(cards[idx].getAttribute('data-folder-index') ?? '0')
      }
      continue
    }
    if (my < rect.top) {
      return parseInt(cards[idx].getAttribute('data-folder-index') ?? '0')
    }
  }

  return store.currentFolders.length
}

function liveReorder(fromIndex: number, toIndex: number): boolean {
  const proj = store.currentProject
  if (!proj || !dragFolderName.value) return false
  const folders = proj.folders
  const curIdx = folders.findIndex((f) => f.name === dragFolderName.value)
  if (curIdx < 0) return false
  const adjTo = toIndex > curIdx ? toIndex - 1 : toIndex
  if (adjTo === curIdx) return false
  const [moved] = folders.splice(curIdx, 1)
  folders.splice(adjTo, 0, moved)
  return true
}


function onConfirmOk(): void {
  confirmCallback?.()
  confirmCallback = null
  showConfirmDialog.value = false
}

function onConfirmCancel(): void {
  confirmCallback = null
  showConfirmDialog.value = false
}

function onCardsAreaMouseDown(e: MouseEvent): void {
  if (e.button !== 0) return
  const target = e.target as HTMLElement
  if (target.closest('[data-folder-index]') || target.closest('.add-card')) return
  e.preventDefault()
  selectedFolderNames.value = []
  isBoxSelecting.value = true
  boxStartX.value = e.clientX
  boxStartY.value = e.clientY
}

function onCardSelect(name: string, ctrlKey: boolean): void {
  if (ctrlKey) {
    const idx = selectedFolderNames.value.indexOf(name)
    if (idx >= 0) {
      selectedFolderNames.value = selectedFolderNames.value.filter((n) => n !== name)
    } else {
      selectedFolderNames.value = [...selectedFolderNames.value, name]
    }
  } else {
    selectedFolderNames.value = [name]
  }
}

function onWindowMouseMove(e: MouseEvent): void {
  if (isBoxSelecting.value) {
    const x1 = Math.min(boxStartX.value, e.clientX)
    const y1 = Math.min(boxStartY.value, e.clientY)
    const x2 = Math.max(boxStartX.value, e.clientX)
    const y2 = Math.max(boxStartY.value, e.clientY)

    boxRect.value = { left: x1, top: y1, width: x2 - x1, height: y2 - y1 }

    const cards = document.querySelectorAll('[data-folder-index]')
    const selected: string[] = []
    for (const card of cards) {
      const r = card.getBoundingClientRect()
      if (r.left < x2 && r.right > x1 && r.top < y2 && r.bottom > y1) {
        const idx = card.getAttribute('data-folder-index')
        if (idx !== null) {
          const folder = store.currentFolders[parseInt(idx)]
          if (folder) selected.push(folder.name)
        }
      }
    }
    selectedFolderNames.value = selected
    return
  }

  if (dragFromIndex.value >= 0) {
    const dx = Math.abs(e.clientX - dragStartPos.x)
    const dy = Math.abs(e.clientY - dragStartPos.y)
    if (dx > 5 || dy > 5) {
      if (!isDraggingCard.value) {
        isDraggingCard.value = true
        document.body.style.cursor = 'grabbing'
        const el = document.querySelector(
          `[data-folder-name="${dragFolderName.value}"]`
        )
        if (el) {
          const rect = el.getBoundingClientRect()
          dragGhostW.value = rect.width
          dragGhostH.value = rect.height
          dragOffsetX.value = e.clientX - rect.left
          dragOffsetY.value = e.clientY - rect.top
        }
      }
      dragGhostX.value = e.clientX - dragOffsetX.value
      dragGhostY.value = e.clientY - dragOffsetY.value
      const target = getDropTargetIndex(e)
      if (target !== dragToIndex.value) {
        dragToIndex.value = target
        liveReorder(dragFromIndex.value, dragToIndex.value)
      }
    }
  }
}

function onWindowMouseUp(e: MouseEvent): void {
  if (isBoxSelecting.value) {
    isBoxSelecting.value = false
    boxRect.value = null
    return
  }

  if (dragFromIndex.value >= 0) {
    if (isDraggingCard.value) {
      store._save()
      store.setStatus('已重新排序')
    } else {
      const folder = store.currentFolders[dragFromIndex.value]
      if (folder) onCardSelect(folder.name, e.ctrlKey || e.metaKey)
    }
    dragFromIndex.value = -1
    dragToIndex.value = -1
    isDraggingCard.value = false
    dropIndicatorIndex.value = -1
    dragFolderName.value = ''
    document.body.style.cursor = ''
  }
}

function onWindowBlur(): void {
  if (isDraggingCard.value || dragFromIndex.value >= 0) {
    dragFromIndex.value = -1
    dragToIndex.value = -1
    isDraggingCard.value = false
    dropIndicatorIndex.value = -1
    dragFolderName.value = ''
    document.body.style.cursor = ''
  }
  if (isBoxSelecting.value) {
    isBoxSelecting.value = false
    boxRect.value = null
  }
}

function onWindowKeyDown(e: KeyboardEvent): void {
  if (document.activeElement?.tagName === 'INPUT') return
  if (e.key !== 'Delete') return
  if (showConfirmDialog.value || showProjectDialog.value || showFolderDialog.value) return
  if (document.querySelector('.project-dropdown')) return
  if (selectedFolderNames.value.length === 0) return
  e.preventDefault()
  confirmDialogTitle.value = '删除文件夹'
  confirmDialogMessage.value = `确定要删除选中的 ${selectedFolderNames.value.length} 个文件夹吗？\n（不会删除实际文件）`
  confirmCallback = () => {
    const names = [...selectedFolderNames.value]
    store.deleteFolders(names)
    selectedFolderNames.value = []
    store.setStatus(`已删除 ${names.length} 个文件夹`)
  }
  showConfirmDialog.value = true
}

function onCardMouseDown(index: number, _e: MouseEvent): void {
  dragFromIndex.value = index
  dragStartPos = { x: _e.clientX, y: _e.clientY }
  isDraggingCard.value = false
  dragFolderName.value = store.currentFolders[index]?.name ?? ''
}
</script>

<template>
  <div class="app-root">
    <TitleBar />
    <div class="app-body">
      <Sidebar
        :projects="store.projects"
        :current-name="store.currentProjectName"
        @select="store.selectProject"
        @add="onAddProject"
        @edit="onEditProject"
        @delete="onDeleteProject"
      />
      <div class="main-area">
        <div class="main-header" v-if="store.currentProject">
          <span class="main-title">{{ store.currentProjectName }}</span>
          <div class="header-actions">
            <button class="btn btn-accent" @click="onAddFolder">＋ 添加文件夹</button>
          </div>
        </div>
        <div ref="cardsAreaRef" class="cards-area" @mousedown="onCardsAreaMouseDown">
          <div v-if="isBoxSelecting && boxRect" class="box-select-overlay" :style="boxOverlayStyle" />
          <Transition name="project-swap" mode="out-in">
            <div v-if="store.currentProject" :key="store.currentProjectName" class="cards-grid">
              <TransitionGroup name="flip-list">
                <FolderCard
                  v-for="(folder, index) in store.currentFolders"
                  :key="folder.name"
                  :class="{ 'is-drag-source': isDraggingCard && folder.name === dragFolderName }"
                  :data-folder-name="folder.name"
                  :folder="folder"
                  :index="index"
                  :selected="selectedFolderNames.includes(folder.name)"
                  @mousedown="onCardMouseDown"
                  @open="onOpenFolder"
                  @copy="onCopyPath"
                  @edit="onEditFolder(folder.name, folder.path)"
                  @delete="onDeleteFolder"
                />
              </TransitionGroup>
              <AddFolderCard v-if="store.currentFolders.length === 0" @click="onAddFolder" />
            </div>
          </Transition>
          <div
            v-if="isDraggingCard && dragFolder"
            class="drag-ghost"
            :style="{
              left: dragGhostX + 'px',
              top: dragGhostY + 'px',
              width: dragGhostW + 'px',
              height: dragGhostH + 'px'
            }"
          >
            <div class="ghost-icon">📂</div>
            <div class="ghost-name">{{ dragFolder.name }}</div>
            <div class="ghost-path">{{ dragFolder.path }}</div>
          </div>
          <div v-if="!store.currentProject" class="empty-hint">
            <div class="empty-icon">📁</div>
            <div class="empty-text">选择或新建一个项目开始</div>
          </div>
        </div>
      </div>
    </div>
    <StatusBar />
    <ProjectDialog
      v-if="showProjectDialog"
      :mode="projectDialogMode"
      :default-name="projectDialogMode === 'edit' ? store.currentProjectName : ''"
      @confirm="onProjectDialogConfirm"
      @cancel="showProjectDialog = false"
    />
    <FolderDialog
      v-if="showFolderDialog"
      :mode="folderDialogMode"
      :default-name="editFolderName"
      :default-path="editFolderPath"
      @confirm="onFolderDialogConfirm"
      @cancel="showFolderDialog = false"
    />
    <ConfirmDialog
      v-if="showConfirmDialog"
      :title="confirmDialogTitle"
      :message="confirmDialogMessage"
      @confirm="onConfirmOk"
      @cancel="onConfirmCancel"
    />
  </div>
</template>

<style scoped>
.flip-list-move {
  transition: transform 0.25s cubic-bezier(0.2, 0, 0, 1);
}

.project-swap-enter-active,
.project-swap-leave-active {
  transition: opacity 0.09s cubic-bezier(0.2, 0, 0, 1),
              transform 0.09s cubic-bezier(0.2, 0, 0, 1);
}

.project-swap-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.project-swap-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

.drag-ghost {
  position: fixed;
  z-index: 9999;
  pointer-events: none;
  background: rgba(40, 40, 46, 0.72);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(96, 205, 255, 0.35);
  border-radius: 14px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.45);
  transform: scale(1.04);
}

.ghost-icon {
  font-size: 17px;
  margin-bottom: 2px;
}

.ghost-name {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
}

.ghost-path {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
