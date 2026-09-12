import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

vi.mock('@/infrastructure/tauri/config', () => ({
  loadConfig: vi.fn(),
  scheduleSave: vi.fn()
}))

import { scheduleSave } from '@/infrastructure/tauri/config'
import { useProjectStore } from '@/features/projects'
import { useFolderStore } from './folder.store'

const scheduleSaveMock = vi.mocked(scheduleSave)

function setup(): void {
  useProjectStore().hydrate({
    currentProjectId: 'p1',
    projects: [{ id: 'p1', name: 'A', folders: [] }]
  })
}

beforeEach(() => {
  setActivePinia(createPinia())
  scheduleSaveMock.mockClear()
})

describe('folder.store', () => {
  it('adds folders to the current project and persists', () => {
    setup()
    const projectStore = useProjectStore()
    const folderStore = useFolderStore()
    folderStore.addFolder('X', 'C:\\X')
    expect(projectStore.currentFolders.map((folder) => folder.name)).toEqual(['X'])
    expect(scheduleSaveMock).toHaveBeenCalled()
  })

  it('ignores duplicate names', () => {
    setup()
    const projectStore = useProjectStore()
    const folderStore = useFolderStore()
    folderStore.addFolder('X', 'C:\\X')
    folderStore.addFolder('X', 'D:\\X')
    expect(projectStore.currentFolders).toHaveLength(1)
  })

  it('updates a folder by id', () => {
    setup()
    const projectStore = useProjectStore()
    const folderStore = useFolderStore()
    folderStore.addFolder('X', 'C:\\X')
    const id = projectStore.currentFolders[0].id
    folderStore.updateFolder(id, 'Y', 'C:\\Y')
    expect(projectStore.currentFolders[0]).toMatchObject({ id, name: 'Y', path: 'C:\\Y' })
  })

  it('removes folders by id', () => {
    setup()
    const projectStore = useProjectStore()
    const folderStore = useFolderStore()
    folderStore.addFolders([
      { name: 'A', path: 'C:\\A' },
      { name: 'B', path: 'C:\\B' }
    ])
    folderStore.removeFolders([projectStore.currentFolders[0].id])
    expect(projectStore.currentFolders.map((folder) => folder.name)).toEqual(['B'])
  })

  it('reorders folders by id', () => {
    setup()
    const projectStore = useProjectStore()
    const folderStore = useFolderStore()
    folderStore.addFolders([
      { name: 'A', path: 'C:\\A' },
      { name: 'B', path: 'C:\\B' }
    ])
    const firstId = projectStore.currentFolders[0].id
    folderStore.reorderFolder(firstId, 2)
    expect(projectStore.currentFolders.map((folder) => folder.name)).toEqual(['B', 'A'])
  })
})
