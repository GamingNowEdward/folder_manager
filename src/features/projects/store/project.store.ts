import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import type { Folder, Project, ProjectId, WorkspaceData } from '@/types'
import { loadConfig, scheduleSave } from '@/infrastructure/tauri/config'
import {
  createProject,
  findProjectById,
  removeProject,
  renameProject
} from '../model/project.model'

export const useProjectStore = defineStore('projects', () => {
  const projects = ref<Project[]>([])
  const currentProjectId = ref<ProjectId>('')

  const currentProject = computed(() => findProjectById(projects.value, currentProjectId.value))
  const currentFolders = computed(() => currentProject.value?.folders ?? [])

  function hydrate(workspace: WorkspaceData): void {
    projects.value = workspace.projects
    currentProjectId.value = workspace.currentProjectId
  }

  function snapshot(): WorkspaceData {
    return { currentProjectId: currentProjectId.value, projects: projects.value }
  }

  function persist(): void {
    scheduleSave(snapshot())
  }

  async function loadFromDisk(): Promise<void> {
    try {
      hydrate(await loadConfig())
    } catch (error) {
      console.error('配置加载失败:', error)
      projects.value = []
      currentProjectId.value = ''
    }
  }

  function selectProject(id: ProjectId): void {
    currentProjectId.value = id
    persist()
  }

  function addProject(name: string): void {
    const project = createProject(name)
    projects.value = [...projects.value, project]
    currentProjectId.value = project.id
    persist()
  }

  function renameProjectById(id: ProjectId, name: string): void {
    projects.value = renameProject(projects.value, id, name)
    persist()
  }

  function deleteProject(id: ProjectId): void {
    projects.value = removeProject(projects.value, id)
    if (currentProjectId.value === id) {
      currentProjectId.value = projects.value[0]?.id ?? ''
    }
    persist()
  }

  function replaceCurrentFolders(folders: Folder[]): void {
    const project = currentProject.value
    if (!project) return
    project.folders = folders
    persist()
  }

  return {
    projects,
    currentProjectId,
    currentProject,
    currentFolders,
    hydrate,
    snapshot,
    persist,
    loadFromDisk,
    selectProject,
    addProject,
    renameProjectById,
    deleteProject,
    replaceCurrentFolders
  }
})
