<script setup lang="ts">
import { ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useProjectStore } from '@/features/projects'
import { useExternalDrop, useFolderDragDrop } from '@/features/drag-drop'
import { useBoxSelection, useFolderSelection } from '@/features/selection'
import { useFolderActions, useFolderShortcuts, useFolderStore } from '@/features/folders'
import { usePreferencesStore } from '@/shared/stores/preferences.store'
import { useStatusStore } from '@/shared/stores/status.store'
import AddFolderCard from './AddFolderCard.vue'
import AddFolderRow from './AddFolderRow.vue'
import FolderCard from './FolderCard.vue'
import FolderListItem from './FolderListItem.vue'

const projectStore = useProjectStore()
const folderStore = useFolderStore()
const folderActions = useFolderActions()
const statusStore = useStatusStore()
const preferences = usePreferencesStore()
const { currentProject, currentFolders, currentProjectId } = storeToRefs(projectStore)
const { viewMode } = storeToRefs(preferences)

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
  layout: () => viewMode.value,
  onReorder: (id, targetIndex) => folderStore.reorderFolder(id, targetIndex),
  onSelect: (id, additive) => {
    if (additive) selection.toggle(id)
    else selection.selectOne(id)
  },
  onDragFinished: () => statusStore.setStatus('已重新排序')
})

useFolderShortcuts({
  selection,
  // 拖拽 / 框选进行中不响应快捷键，避免把进行中的手势打断。
  isBusy: () => isBoxSelecting.value || isDragging.value
})
useExternalDrop()

// 选择集是当前项目的局部状态，切换项目后旧 id 已无意义，必须清空。
watch(currentProjectId, () => selection.clear())
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
        <div
          v-if="currentProject"
          :key="currentProject.name"
          :class="viewMode === 'grid' ? 'cards-grid' : 'folder-list'"
        >
          <TransitionGroup v-if="viewMode === 'grid'" name="flip-list">
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
          <TransitionGroup v-else name="flip-list">
            <FolderListItem
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
            v-if="viewMode === 'grid' && currentFolders.length === 0"
            @click="folderActions.openAddDialog()"
          />
          <AddFolderRow
            v-else-if="viewMode === 'list' && currentFolders.length === 0"
            @click="folderActions.openAddDialog()"
          />
        </div>
      </Transition>
      <div
        v-if="isDragging && draggedFolder"
        class="drag-ghost"
        :class="{ 'is-row': viewMode === 'list' }"
        :style="{
          left: ghost.x + 'px',
          top: ghost.y + 'px',
          width: ghost.width + 'px',
          height: ghost.height + 'px'
        }"
      >
        <template v-if="viewMode === 'list'">
          <span class="ghost-row-icon">📂</span>
          <span class="ghost-row-name">{{ draggedFolder.name }}</span>
          <span class="ghost-row-path">{{ draggedFolder.path }}</span>
        </template>
        <template v-else>
          <div class="ghost-icon">📂</div>
          <div class="ghost-name">{{ draggedFolder.name }}</div>
          <div class="ghost-path">{{ draggedFolder.path }}</div>
        </template>
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

/* 列表视图的 ghost 是单行条：宽高取自被拖拽行，内容改为横排 */
.drag-ghost.is-row {
  flex-direction: row;
  align-items: center;
  gap: 10px;
  padding: 0 12px;
  transform: none;
  max-width: 460px;
}

.ghost-row-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.ghost-row-name {
  font-size: 13px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ghost-row-path {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
