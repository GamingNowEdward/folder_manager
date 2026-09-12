import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

vi.mock('@/infrastructure/tauri/config', () => ({
  loadConfig: vi.fn(),
  scheduleSave: vi.fn()
}))

import { scheduleSave } from '@/infrastructure/tauri/config'
import { useProjectStore } from './project.store'

const scheduleSaveMock = vi.mocked(scheduleSave)

beforeEach(() => {
  setActivePinia(createPinia())
  scheduleSaveMock.mockClear()
})

describe('project.store', () => {
  it('hydrates projects and current id', () => {
    const store = useProjectStore()
    store.hydrate({
      currentProjectId: 'p1',
      projects: [{ id: 'p1', name: '工作', folders: [] }]
    })
    expect(store.currentProject?.name).toBe('工作')
  })

  it('adds a project, selects it and persists', () => {
    const store = useProjectStore()
    store.addProject('工作')
    expect(store.projects).toHaveLength(1)
    expect(store.currentProjectId).toBe(store.projects[0].id)
    expect(scheduleSaveMock).toHaveBeenCalledTimes(1)
  })

  it('renames a project without changing identity', () => {
    const store = useProjectStore()
    store.addProject('工作')
    const id = store.currentProjectId
    store.renameProjectById(id, '工作2')
    expect(store.currentProjectId).toBe(id)
    expect(store.currentProject?.name).toBe('工作2')
  })

  it('selects the first remaining project after deleting the current one', () => {
    const store = useProjectStore()
    store.hydrate({
      currentProjectId: 'p2',
      projects: [
        { id: 'p1', name: 'A', folders: [] },
        { id: 'p2', name: 'B', folders: [] }
      ]
    })
    store.deleteProject('p2')
    expect(store.currentProjectId).toBe('p1')
    expect(store.projects.map((project) => project.id)).toEqual(['p1'])
  })

  it('clears current project when the last project is deleted', () => {
    const store = useProjectStore()
    store.hydrate({
      currentProjectId: 'p1',
      projects: [{ id: 'p1', name: 'A', folders: [] }]
    })
    store.deleteProject('p1')
    expect(store.currentProjectId).toBe('')
  })

  it('replaces folders of the current project', () => {
    const store = useProjectStore()
    store.hydrate({
      currentProjectId: 'p1',
      projects: [{ id: 'p1', name: 'A', folders: [] }]
    })
    store.replaceCurrentFolders([{ id: 'f1', name: 'X', path: 'C:\\X' }])
    expect(store.currentFolders.map((folder) => folder.name)).toEqual(['X'])
  })
})
