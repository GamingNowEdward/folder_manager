import type { Project, ProjectId } from '@/types'
import { createId } from '@/shared/utils/id'

export function createProject(name: string): Project {
  return { id: createId(), name, folders: [] }
}

export function findProjectById(projects: Project[], id: ProjectId): Project | null {
  return projects.find((project) => project.id === id) ?? null
}

export function isProjectNameTaken(
  projects: Project[],
  name: string,
  exceptId?: ProjectId
): boolean {
  return projects.some((project) => project.name === name && project.id !== exceptId)
}

export function renameProject(projects: Project[], id: ProjectId, name: string): Project[] {
  const project = findProjectById(projects, id)
  if (!project || isProjectNameTaken(projects, name, id)) return projects
  return projects.map((item) => (item.id === id ? { ...item, name } : item))
}

export function removeProject(projects: Project[], id: ProjectId): Project[] {
  return projects.filter((project) => project.id !== id)
}
