import { defineStore } from 'pinia'
import type { Folder, FolderId } from '@/types'
import { useProjectStore } from '@/features/projects'
import {
  addFolder as addFolderModel,
  moveFolderToIndex,
  removeFolders as removeFoldersModel,
  updateFolder as updateFolderModel
} from '../model/folder.model'

export interface FolderEntry {
  name: string
  path: string
}

export const useFolderStore = defineStore('folders', () => {
  const projectStore = useProjectStore()

  function replace(folders: Folder[]): void {
    projectStore.replaceCurrentFolders(folders)
  }

  function addFolder(name: string, path: string): void {
    replace(addFolderModel(projectStore.currentFolders, name, path))
  }

  function addFolders(entries: FolderEntry[]): void {
    let folders = projectStore.currentFolders
    for (const entry of entries) {
      folders = addFolderModel(folders, entry.name, entry.path)
    }
    replace(folders)
  }

  function updateFolder(id: FolderId, name: string, path: string): void {
    replace(updateFolderModel(projectStore.currentFolders, id, name, path))
  }

  function removeFolder(id: FolderId): void {
    replace(removeFoldersModel(projectStore.currentFolders, [id]))
  }

  function removeFolders(ids: FolderId[]): void {
    replace(removeFoldersModel(projectStore.currentFolders, ids))
  }

  function reorderFolder(id: FolderId, targetIndex: number): void {
    replace(moveFolderToIndex(projectStore.currentFolders, id, targetIndex))
  }

  return { addFolder, addFolders, updateFolder, removeFolder, removeFolders, reorderFolder }
})
