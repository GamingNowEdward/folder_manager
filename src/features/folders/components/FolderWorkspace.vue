<script setup lang="ts">
import { ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useProjectStore } from '@/features/projects'
import { useExternalDrop, useFolderDragDrop } from '@/features/drag-drop'
import { useBoxSelection, useFolderSelection } from '@/features/selection'
import { useDeleteShortcut, useFolderActions, useFolderStore } from '@/features/folders'
import { useStatusStore } from '@/shared/stores/status.store'
import AddFolderCard from './AddFolderCard.vue'
import FolderCard from './FolderCard.vue'

const projectStore = useProjectStore()
const folderStore = useFolderStore()
const folderActions = useFolderActions()
const statusStore = useStatusStore()
const { currentProject, currentFolders } = storeToRefs(projectStore)

const selection = useFolderSelection()
const { isSelected } = selection

const cardsAreaRef = ref<HTMLElement | null>(null)

const { isBoxSelecting, boxRect, boxOverlayStyle, onAreaMouseDown } = useBoxSelection({
  areaRef: cardsAreaRef,
  onSelect: (ids) => selection.setIds(ids)
})

const { isDragging, dragId, ghost, draggedFolder, onCardMouseDown } = useFolderDragDrop({
  areaRef: cardsAreaRef,
  folders: currentFolders,
  onReorder: (id, targetIndex) => folderStore.reorderFolder(id, targetIndex),
  onSelect: (id, additive) => {
    if (additive) selection.toggle(id)
    else selection.selectOne(id)
  },
  onDragFinished: () => statusStore.setStatus('已重新排序')
})

useDeleteShortcut(selection)
useExternalDrop()
</script>

<template>
  <div class="main-area">
    <div v-if="currentProject" class="main-header">
      <span class="main-title">{{ currentProject.name }}</span>
      <div class="header-actions">
        <button class="btn btn-accent" @click="folderActions.openAddDialog()">＋ 添加文件夹</button>
      </div>
    </div>
    <div ref="cardsAreaRef" class="cards-area" @mousedown="onAreaMouseDown">
      <div v-if="isBoxSelecting && boxRect" class="box-select-overlay" :style="boxOverlayStyle" />
      <Transition name="project-swap" mode="out-in">
        <div v-if="currentProject" :key="currentProject.name" class="cards-grid">
          <TransitionGroup name="flip-list">
            <FolderCard
              v-for="folder in currentFolders"
              :key="folder.id"
              :class="{ 'is-drag-source': isDragging && folder.id === dragId }"
              :folder="folder"
              :selected="isSelected(folder.id)"
              @mousedown="onCardMouseDown"
              @open="folderActions.openInSystem"
              @copy="folderActions.copyFolderPath"
              @edit="folderActions.openEditDialog"
              @delete="folderActions.requestDelete"
            />
          </TransitionGroup>
          <AddFolderCard
            v-if="currentFolders.length === 0"
            @click="folderActions.openAddDialog()"
          />
        </div>
      </Transition>
      <div
        v-if="isDragging && draggedFolder"
        class="drag-ghost"
        :style="{
          left: ghost.x + 'px',
          top: ghost.y + 'px',
          width: ghost.width + 'px',
          height: ghost.height + 'px'
        }"
      >
        <div class="ghost-icon">📂</div>
        <div class="ghost-name">{{ draggedFolder.name }}</div>
        <div class="ghost-path">{{ draggedFolder.path }}</div>
      </div>
      <div v-if="!currentProject" class="empty-hint">
        <div class="empty-icon">📁</div>
        <div class="empty-text">选择或新建一个项目开始</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.flip-list-move {
  transition: transform 0.25s cubic-bezier(0.2, 0, 0, 1);
}

.project-swap-enter-active,
.project-swap-leave-active {
  transition:
    opacity 0.09s cubic-bezier(0.2, 0, 0, 1),
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
