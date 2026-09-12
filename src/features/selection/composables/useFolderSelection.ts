import { computed, ref } from 'vue'
import type { FolderId } from '@/types'
import { selectOnly, toggleId } from '../model/selection.model'

export function useFolderSelection() {
  const selectedIds = ref<FolderId[]>([])
  const selectedIdSet = computed(() => new Set(selectedIds.value))

  function clear(): void {
    selectedIds.value = []
  }

  function selectOne(id: FolderId): void {
    selectedIds.value = selectOnly(id)
  }

  function toggle(id: FolderId): void {
    selectedIds.value = toggleId(selectedIds.value, id)
  }

  function setIds(ids: FolderId[]): void {
    selectedIds.value = ids
  }

  function isSelected(id: FolderId): boolean {
    return selectedIdSet.value.has(id)
  }

  return { selectedIds, selectedIdSet, clear, selectOne, toggle, setIds, isSelected }
}

export type FolderSelection = ReturnType<typeof useFolderSelection>
