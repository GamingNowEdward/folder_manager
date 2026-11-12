export type ProjectId = string
export type FolderId = string

export interface Folder {
  id: FolderId
  name: string
  path: string
  /** 添加时间（Unix 毫秒）：用于「添加日期」排序，与显示名/路径无关。 */
  createdAt: number
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
