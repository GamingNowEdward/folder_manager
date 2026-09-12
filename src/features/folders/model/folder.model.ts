import type { Folder, FolderId } from '@/types'
import { createId } from '@/shared/utils/id'

export function createFolder(name: string, path: string): Folder {
  return { id: createId(), name, path }
}

export function findFolderById(folders: Folder[], id: FolderId): Folder | null {
  return folders.find((folder) => folder.id === id) ?? null
}

export function isFolderNameTaken(folders: Folder[], name: string, exceptId?: FolderId): boolean {
  return folders.some((folder) => folder.name === name && folder.id !== exceptId)
}

export function addFolder(folders: Folder[], name: string, path: string): Folder[] {
  if (isFolderNameTaken(folders, name)) return folders
  return [...folders, createFolder(name, path)]
}

export function updateFolder(
  folders: Folder[],
  id: FolderId,
  name: string,
  path: string
): Folder[] {
  const folder = findFolderById(folders, id)
  if (!folder || isFolderNameTaken(folders, name, id)) return folders
  return folders.map((item) => (item.id === id ? { ...item, name, path } : item))
}

export function removeFolders(folders: Folder[], ids: FolderId[]): Folder[] {
  if (ids.length === 0) return folders
  const removing = new Set(ids)
  return folders.filter((folder) => !removing.has(folder.id))
}

export function moveFolderToIndex(folders: Folder[], id: FolderId, targetIndex: number): Folder[] {
  const currentIndex = folders.findIndex((folder) => folder.id === id)
  if (currentIndex < 0) return folders
  if (targetIndex < 0 || targetIndex > folders.length) return folders
  const adjusted = targetIndex > currentIndex ? targetIndex - 1 : targetIndex
  if (adjusted === currentIndex) return folders
  const next = [...folders]
  const [moved] = next.splice(currentIndex, 1)
  next.splice(adjusted, 0, moved)
  return next
}
