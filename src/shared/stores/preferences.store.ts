import { ref } from 'vue'
import { defineStore } from 'pinia'
import { VIEW_MODE_STORAGE_KEY } from '@/shared/constants'
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

export const usePreferencesStore = defineStore('preferences', () => {
  const storage = resolveStorage()
  const viewMode = ref<FolderViewMode>(readStoredViewMode(storage))

  function setViewMode(mode: FolderViewMode): void {
    if (viewMode.value === mode) return
    viewMode.value = mode
    writeStoredViewMode(mode, storage)
  }

  function toggleViewMode(): void {
    setViewMode(viewMode.value === 'grid' ? 'list' : 'grid')
  }

  return { viewMode, setViewMode, toggleViewMode }
})
