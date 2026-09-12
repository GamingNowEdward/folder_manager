export type ProjectId = string
export type FolderId = string

export interface Folder {
  id: FolderId
  name: string
  path: string
}

export interface Project {
  id: ProjectId
  name: string
  folders: Folder[]
}

export interface WorkspaceData {
  currentProjectId: ProjectId
  projects: Project[]
}
