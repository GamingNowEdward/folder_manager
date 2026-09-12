import { describe, expect, it } from 'vitest'
import {
  createProject,
  findProjectById,
  isProjectNameTaken,
  removeProject,
  renameProject
} from './project.model'
import type { Project } from '@/types'

function project(id: string, name: string): Project {
  return { id, name, folders: [] }
}

describe('project.model', () => {
  it('creates a project with a generated id', () => {
    const created = createProject('工作')
    expect(created.id).toBeTruthy()
    expect(created.name).toBe('工作')
    expect(created.folders).toEqual([])
  })

  it('detects duplicate names while excluding the edited project', () => {
    const projects = [project('p1', '工作'), project('p2', '私人')]
    expect(isProjectNameTaken(projects, '工作')).toBe(true)
    expect(isProjectNameTaken(projects, '工作', 'p1')).toBe(false)
    expect(isProjectNameTaken(projects, '新项目')).toBe(false)
  })

  it('renames only the targeted project', () => {
    const projects = [project('p1', '工作'), project('p2', '私人')]
    const next = renameProject(projects, 'p2', '娱乐')
    expect(next.map((item) => item.name)).toEqual(['工作', '娱乐'])
    expect(projects[1].name).toBe('私人')
  })

  it('ignores rename when the target is missing or the name is taken', () => {
    const projects = [project('p1', '工作'), project('p2', '私人')]
    expect(renameProject(projects, 'missing', 'X')).toBe(projects)
    expect(renameProject(projects, 'p2', '工作')).toBe(projects)
  })

  it('finds and removes projects by id', () => {
    const projects = [project('p1', '工作'), project('p2', '私人')]
    expect(findProjectById(projects, 'p2')?.name).toBe('私人')
    expect(findProjectById(projects, 'nope')).toBeNull()
    expect(removeProject(projects, 'p1').map((item) => item.id)).toEqual(['p2'])
  })
})
