import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Project, Folder, AppData } from '../types'
import { invoke } from '@tauri-apps/api/core'

export const useAppStore = defineStore('app', () => {
  const projects = ref<Project[]>([])
  const currentProjectName = ref('')
  const statusMessage = ref('● 就绪')
  let _statusTimer: ReturnType<typeof setTimeout> | null = null
  let _saveTimer: ReturnType<typeof setTimeout> | null = null

  const currentProject = computed(() =>
    projects.value.find((p) => p.name === currentProjectName.value) ?? null
  )

  const currentFolders = computed(() => currentProject.value?.folders ?? [])

  function _save(): void {
    if (_saveTimer) clearTimeout(_saveTimer)
    _saveTimer = setTimeout(() => {
      const data: AppData = JSON.parse(JSON.stringify({
        current_project: currentProjectName.value,
        projects: projects.value
      }))
      invoke('save_config', { data }).catch((e) => {
        console.error('閰嶇疆淇濆瓨澶辫触:', e)
      })
    }, 150)
  }

  async function loadFromDisk(): Promise<void> {
    try {
      const data = await invoke<AppData>('load_config')
      projects.value = data.projects ?? []
      currentProjectName.value = data.current_project ?? ''
    } catch {
      projects.value = []
      currentProjectName.value = ''
    }
  }

  function selectProject(name: string): void {
    currentProjectName.value = name
    _save()
  }

  function addProject(name: string): void {
    if (projects.value.some((p) => p.name === name)) return
    projects.value.push({ name, folders: [] })
    currentProjectName.value = name
    _save()
  }

  function renameProject(oldName: string, newName: string): void {
    const proj = projects.value.find((p) => p.name === oldName)
    if (!proj) return
    if (projects.value.some((p) => p.name === newName)) return
    proj.name = newName
    if (currentProjectName.value === oldName) {
      currentProjectName.value = newName
    }
    _save()
  }

  function deleteProject(name: string): void {
    projects.value = projects.value.filter((p) => p.name !== name)
    if (currentProjectName.value === name) {
      currentProjectName.value = projects.value[0]?.name ?? ''
    }
    _save()
  }

  function addFolder(name: string, path: string): void {
    const proj = currentProject.value
    if (!proj) return
    if (proj.folders.some((f) => f.name === name)) return
    proj.folders.push({ name, path })
    _save()
  }

  function renameFolder(oldName: string, newName: string, newPath: string): void {
    const proj = currentProject.value
    if (!proj) return
    const folder = proj.folders.find((f) => f.name === oldName)
    if (!folder) return
    if (proj.folders.some((f) => f.name === newName && f.name !== oldName)) return
    folder.name = newName
    folder.path = newPath
    _save()
  }

  function deleteFolder(name: string): void {
    const proj = currentProject.value
    if (!proj) return
    proj.folders = proj.folders.filter((f) => f.name !== name)
    _save()
  }

  function deleteFolders(names: string[]): void {
    const proj = currentProject.value
    if (!proj) return
    proj.folders = proj.folders.filter((f) => !names.includes(f.name))
    _save()
  }

  function moveFolder(name: string, direction: number): void {
    const proj = currentProject.value
    if (!proj) return
    const idx = proj.folders.findIndex((f) => f.name === name)
    if (idx < 0) return
    const newIdx = idx + direction
    if (newIdx < 0 || newIdx >= proj.folders.length) return
    ;[proj.folders[idx], proj.folders[newIdx]] = [proj.folders[newIdx], proj.folders[idx]]
    _save()
  }

  function reorderFolders(fromIndex: number, toIndex: number): void {
    const proj = currentProject.value
    if (!proj) return
    const folders = proj.folders
    if (fromIndex < 0 || fromIndex >= folders.length) return
    if (toIndex < 0 || toIndex > folders.length) return
    const [moved] = folders.splice(fromIndex, 1)
    const adjustedTo = toIndex > fromIndex ? toIndex - 1 : toIndex
    folders.splice(adjustedTo, 0, moved)
    _save()
  }

  function setStatus(msg: string, timeout = 4000): void {
    if (_statusTimer) clearTimeout(_statusTimer)
    statusMessage.value = msg
    if (timeout > 0) {
      _statusTimer = setTimeout(() => { statusMessage.value = '● 就绪' }, timeout)
    }
  }

  return {
    projects,
    currentProjectName,
    currentProject,
    currentFolders,
    statusMessage,
    loadFromDisk,
    selectProject,
    addProject,
    renameProject,
    deleteProject,
    addFolder,
    renameFolder,
    deleteFolder,
    deleteFolders,
    moveFolder,
    reorderFolders,
    setStatus,
    _save
  }
})

