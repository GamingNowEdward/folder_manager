import { ref } from 'vue'
import { defineStore } from 'pinia'
import { FOLDER_SORT_STORAGE_KEY, VIEW_MODE_STORAGE_KEY } from '@/shared/constants'
import {
  DEFAULT_FOLDER_SORT,
  parseFolderSort,
  serializeFolderSort,
  type FolderSort,
  type FolderSortField
} from '@/shared/model/folder-sort'
import {
  DEFAULT_FOLDER_VIEW_MODE,
  parseViewMode,
  type FolderViewMode
} from '@/shared/model/view-mode'

export interface StorageLike {
  getItem(key: string): string | null
  setItem(key: string, value: string): void
}

/** WebView2 在受限隐私策略下访问 localStorage 可能直接抛异常，统一在这里兜底。 */
export function resolveStorage(): StorageLike | null {
  try {
    return typeof localStorage === 'undefined' ? null : localStorage
  } catch {
    return null
  }
}

export function readStoredViewMode(storage: StorageLike | null): FolderViewMode {
  if (!storage) return DEFAULT_FOLDER_VIEW_MODE
  try {
    return parseViewMode(storage.getItem(VIEW_MODE_STORAGE_KEY))
  } catch {
    return DEFAULT_FOLDER_VIEW_MODE
  }
}

export function writeStoredViewMode(mode: FolderViewMode, storage: StorageLike | null): void {
  if (!storage) return
  try {
    storage.setItem(VIEW_MODE_STORAGE_KEY, mode)
  } catch {
    // 视图偏好不是关键数据，写失败直接忽略。
  }
}

export function readStoredFolderSort(storage: StorageLike | null): FolderSort {
  if (!storage) return { ...DEFAULT_FOLDER_SORT }
  try {
    return parseFolderSort(storage.getItem(FOLDER_SORT_STORAGE_KEY))
  } catch {
    return { ...DEFAULT_FOLDER_SORT }
  }
}

export function writeStoredFolderSort(sort: FolderSort, storage: StorageLike | null): void {
  if (!storage) return
  try {
    storage.setItem(FOLDER_SORT_STORAGE_KEY, serializeFolderSort(sort))
  } catch {
    // 排序偏好不是关键数据，写失败直接忽略。
  }
}

function isSameSort(a: FolderSort, b: FolderSort): boolean {
  return a.field === b.field && a.direction === b.direction
}

export const usePreferencesStore = defineStore('preferences', () => {
  const storage = resolveStorage()
  const viewMode = ref<FolderViewMode>(readStoredViewMode(storage))
  const folderSort = ref<FolderSort>(readStoredFolderSort(storage))

  function setViewMode(mode: FolderViewMode): void {
    if (viewMode.value === mode) return
    viewMode.value = mode
    writeStoredViewMode(mode, storage)
  }

  function toggleViewMode(): void {
    setViewMode(viewMode.value === 'grid' ? 'list' : 'grid')
  }

  function setFolderSort(sort: FolderSort): void {
    const next = parseFolderSort(sort)
    if (isSameSort(folderSort.value, next)) return
    folderSort.value = next
    writeStoredFolderSort(next, storage)
  }

  function setSortField(field: FolderSortField): void {
    setFolderSort({ field, direction: folderSort.value.direction })
  }

  /** 自定义排序的方向由拖拽决定，因此不参与正/反切换。 */
  function toggleSortDirection(): void {
    if (folderSort.value.field === 'custom') return
    setFolderSort({
      field: folderSort.value.field,
      direction: folderSort.value.direction === 'asc' ? 'desc' : 'asc'
    })
  }

  return {
    viewMode,
    folderSort,
    setViewMode,
    toggleViewMode,
    setFolderSort,
    setSortField,
    toggleSortDirection
  }
})
