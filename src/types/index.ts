export interface Folder {
  name: string
  path: string
}

export interface Project {
  name: string
  folders: Folder[]
}

export interface AppData {
  current_project: string
  projects: Project[]
}
