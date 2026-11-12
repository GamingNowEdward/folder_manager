import { describe, expect, it } from 'vitest'
import { toConfigDto, toWorkspace } from './config.mapper'

describe('config.mapper', () => {
  it('maps dto to domain and keeps valid ids', () => {
    const workspace = toWorkspace({
      version: 3,
      current_project: 'p1',
      projects: [
        {
          id: 'p1',
          name: '工作',
          folders: [{ id: 'f1', name: 'A', path: 'C:\\A', created_at: 1234 }]
        }
      ]
    })
    expect(workspace.currentProjectId).toBe('p1')
    expect(workspace.projects[0].id).toBe('p1')
    expect(workspace.projects[0].folders[0].id).toBe('f1')
    expect(workspace.projects[0].folders[0].createdAt).toBe(1234)
  })

  it('generates ids when missing and clears unknown current project', () => {
    const workspace = toWorkspace({
      current_project: '工作',
      projects: [{ name: '工作', folders: [{ name: 'A', path: 'C:\\A' }] }]
    })
    expect(workspace.currentProjectId).toBe('')
    expect(workspace.projects[0].id).toBeTruthy()
    expect(workspace.projects[0].folders[0].id).toBeTruthy()
  })

  it('falls back to zero when created_at is missing or invalid', () => {
    const workspace = toWorkspace({
      version: 3,
      projects: [
        {
          id: 'p1',
          name: 'P',
          folders: [
            { id: 'f1', name: 'A', path: 'C:\\A' },
            { id: 'f2', name: 'B', path: 'C:\\B', created_at: -5 },
            { id: 'f3', name: 'C', path: 'C:\\C', created_at: Number.NaN }
          ]
        }
      ]
    })
    expect(workspace.projects[0].folders.map((folder) => folder.createdAt)).toEqual([0, 0, 0])
  })

  it('tolerates null input', () => {
    expect(toWorkspace(null)).toEqual({ currentProjectId: '', projects: [] })
    expect(toWorkspace(undefined)).toEqual({ currentProjectId: '', projects: [] })
  })

  it('maps domain to versioned dto', () => {
    const dto = toConfigDto({
      currentProjectId: 'p1',
      projects: [
        {
          id: 'p1',
          name: '工作',
          folders: [{ id: 'f1', name: 'A', path: 'C:\\A', createdAt: 1234 }]
        }
      ]
    })
    expect(dto).toEqual({
      version: 3,
      current_project: 'p1',
      projects: [
        {
          id: 'p1',
          name: '工作',
          folders: [{ id: 'f1', name: 'A', path: 'C:\\A', created_at: 1234 }]
        }
      ]
    })
  })
})
