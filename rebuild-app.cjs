const fs = require('fs');
const orig = fs.readFileSync('c:/opencode/folder_manager_tauri/src/App.vue', 'utf8');

// Extract script section (before <template>)
const tplStart = orig.indexOf('<template>');
let script = orig.substring(0, tplStart);

// Apply script modifications
script = script
  .replace("import { onMounted, onUnmounted, ref, computed } from 'vue'",
           "import { onMounted, onUnmounted, ref, computed } from 'vue'")
  .replace("import ProjectSelect from './components/ProjectSelect.vue'",
           "import Sidebar from './components/Sidebar.vue'")
  .replace("import FlowLayout from './components/FlowLayout.vue'\n", '');

// Add drag state vars after dragStartPos
script = script.replace(
  'let dragStartPos = { x: 0, y: 0 }',
  `let dragStartPos = { x: 0, y: 0 }
const dragFolderName = ref('')
const dragGhostX = ref(0)
const dragGhostY = ref(0)
const dragGhostW = ref(0)
const dragGhostH = ref(0)
const dragOffsetX = ref(0)
const dragOffsetY = ref(0)

const dragFolder = computed(() =>
  store.currentFolders.find((f) => f.name === dragFolderName.value) ?? null
)`
);

// Replace getDropTargetIndex
const gStart = script.indexOf('function getDropTargetIndex');
const gEnd = script.indexOf('\n}', gStart) + 2;
script = script.substring(0, gStart) + `function getDropTargetIndex(e: { clientX: number; clientY: number }): number {
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
` + script.substring(gEnd);

// Replace onWindowMouseMove drag section
script = script.replace(
  `  if (dragFromIndex.value >= 0) {
    const dx = Math.abs(e.clientX - dragStartPos.x)
    const dy = Math.abs(e.clientY - dragStartPos.y)
    if (dx > 5 || dy > 5) {
      if (!isDraggingCard.value) {
        isDraggingCard.value = true
        document.body.style.cursor = 'grabbing'
      }
      dragToIndex.value = getDropTargetIndex(e)
      dropIndicatorIndex.value = dragToIndex.value
    }
  }`,
  `  if (dragFromIndex.value >= 0) {
    const dx = Math.abs(e.clientX - dragStartPos.x)
    const dy = Math.abs(e.clientY - dragStartPos.y)
    if (dx > 5 || dy > 5) {
      if (!isDraggingCard.value) {
        isDraggingCard.value = true
        document.body.style.cursor = 'grabbing'
        const el = document.querySelector(
          \`[data-folder-name="\${dragFolderName.value}"]\`
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
  }`
);

// Replace onWindowMouseUp
script = script.replace(
  `    if (isDraggingCard.value) {
      if (dragToIndex.value >= 0 && dragToIndex.value !== dragFromIndex.value) {
        store.reorderFolders(dragFromIndex.value, dragToIndex.value)
        store.setStatus('\u5DF2\u91CD\u65B0\u6392\u5E8F')
      }
    } else {`,
  `    if (isDraggingCard.value) {
      store._save()
      store.setStatus('\u5DF2\u91CD\u65B0\u6392\u5E8F')
    } else {`
);

script = script.replace(
  `    dropIndicatorIndex.value = -1
    document.body.style.cursor = ''`,
  `    dropIndicatorIndex.value = -1
    dragFolderName.value = ''
    document.body.style.cursor = ''`
);

// Add dragFolderName to onCardMouseDown
script = script.replace(
  `function onCardMouseDown(index: number, _e: MouseEvent): void {
  dragFromIndex.value = index
  dragStartPos = { x: _e.clientX, y: _e.clientY }
  isDraggingCard.value = false
}`,
  `function onCardMouseDown(index: number, _e: MouseEvent): void {
  dragFromIndex.value = index
  dragStartPos = { x: _e.clientX, y: _e.clientY }
  isDraggingCard.value = false
  dragFolderName.value = store.currentFolders[index]?.name ?? ''
}`
);

// Build new template
const template = `<template>
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
            <button class="btn btn-accent" @click="onAddFolder">\uFF0B \u6DFB\u52A0\u6587\u4EF6\u5939</button>
          </div>
        </div>
        <div ref="cardsAreaRef" class="cards-area" @mousedown="onCardsAreaMouseDown">
          <div v-if="isBoxSelecting && boxRect" class="box-select-overlay" :style="boxOverlayStyle" />
          <div v-if="store.currentProject" class="cards-grid">
            <TransitionGroup name="flip-list">
              <FolderCard
                v-for="(folder, index) in store.currentFolders"
                :key="folder.name"
                :class="{ 'is-drag-source': isDraggingCard && folder.name === dragFolderName }"
                :data-folder-name="folder.name"
                :folder="folder"
                :index="index"
                :total="store.currentFolders.length"
                :selected="selectedFolderNames.includes(folder.name)"
                @mousedown="onCardMouseDown"
                @open="onOpenFolder"
                @copy="onCopyPath"
                @edit="onEditFolder(folder.name, folder.path)"
                @move-up="store.moveFolder(folder.name, -1)"
                @move-down="store.moveFolder(folder.name, 1)"
                @delete="onDeleteFolder"
              />
            </TransitionGroup>
            <AddFolderCard @click="onAddFolder" />
          </div>
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
            <div class="ghost-icon">\uD83D\uDCC2</div>
            <div class="ghost-name">{{ dragFolder.name }}</div>
            <div class="ghost-path">{{ dragFolder.path }}</div>
          </div>
          <div v-if="!store.currentProject" class="empty-hint">
            <div class="empty-icon">\uD83D\uDCC1</div>
            <div class="empty-text">\u9009\u62E9\u6216\u65B0\u5EFA\u4E00\u4E2A\u9879\u76EE\u5F00\u59CB</div>
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

.drag-ghost {
  position: fixed;
  z-index: 9999;
  pointer-events: none;
  background: rgba(44, 44, 48, 0.95);
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
`;

fs.writeFileSync('c:/opencode/folder_manager_fluent/src/App.vue', script + template);
console.log('Rebuilt OK');