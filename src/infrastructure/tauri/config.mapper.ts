import type { Project, WorkspaceData } from '@/types'
import { CONFIG_VERSION } from '@/shared/constants'
import { createId } from '@/shared/utils/id'

export interface FolderDto {
  id?: string | null
  name?: string | null
  path?: string | null
  created_at?: number | null
}

export interface ProjectDto {
  id?: string | null
  name?: string | null
  folders?: FolderDto[] | null
}

export interface ConfigDto {
  version?: number | null
  current_project?: string | null
  projects?: ProjectDto[] | null
}

export function toWorkspace(dto: ConfigDto | null | undefined): WorkspaceData {
  const rawProjects = Array.isArray(dto?.projects) ? dto.projects : []
  const projects: Project[] = rawProjects.map((rawProject) => ({
    id: rawProject?.id || createId(),
    name: rawProject?.name ?? '',
    folders: (Array.isArray(rawProject?.folders) ? rawProject.folders : []).map((rawFolder) => ({
      id: rawFolder?.id || createId(),
      name: rawFolder?.name ?? '',
      path: rawFolder?.path ?? '',
      // Rust 侧迁移会补齐时间戳；这里只做防御性回落，避免脏数据产生 NaN 排序。
      createdAt:
        typeof rawFolder?.created_at === 'number' && Number.isFinite(rawFolder.created_at)
          ? Math.max(0, Math.trunc(rawFolder.created_at))
          : 0
    }))
  }))
  const rawCurrent = dto?.current_project ?? ''
  const currentProjectId = projects.some((project) => project.id === rawCurrent) ? rawCurrent : ''
  return { currentProjectId, projects }
}

export function toConfigDto(workspace: WorkspaceData): ConfigDto {
  return {
    version: CONFIG_VERSION,
    current_project: workspace.currentProjectId,
    projects: workspace.projects.map((project) => ({
      id: project.id,
      name: project.name,
      folders: project.folders.map((folder) => ({
        id: folder.id,
        name: folder.name,
        path: folder.path,
        created_at: folder.createdAt
      }))
    }))
  }
}
