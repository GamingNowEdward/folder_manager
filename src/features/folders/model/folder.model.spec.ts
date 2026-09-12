import { describe, expect, it } from 'vitest'
import {
  addFolder,
  createFolder,
  findFolderById,
  isFolderNameTaken,
  moveFolderToIndex,
  removeFolders,
  updateFolder
} from './folder.model'
import type { Folder } from '@/types'

function folder(id: string, name: string, path = `C:\\${name}`): Folder {
  return { id, name, path }
}

const folders = [folder('f1', 'A'), folder('f2', 'B'), folder('f3', 'C')]

describe('folder.model', () => {
  it('creates a folder with a generated id', () => {
    const created = createFolder('A', 'C:\\A')
    expect(created.id).toBeTruthy()
    expect(created).toMatchObject({ name: 'A', path: 'C:\\A' })
  })

  it('adds folders and rejects duplicate names', () => {
    expect(addFolder(folders, 'A', 'X')).toBe(folders)
    const next = addFolder(folders, 'D', 'C:\\D')
    expect(next).toHaveLength(4)
    expect(next[3].name).toBe('D')
  })

  it('detects duplicate names while excluding the edited folder', () => {
    expect(isFolderNameTaken(folders, 'A')).toBe(true)
    expect(isFolderNameTaken(folders, 'A', 'f1')).toBe(false)
  })

  it('finds and updates folders by id', () => {
    expect(findFolderById(folders, 'f2')?.name).toBe('B')
    const next = updateFolder(folders, 'f2', 'B2', 'C:\\B2')
    expect(next[1]).toMatchObject({ id: 'f2', name: 'B2', path: 'C:\\B2' })
    expect(folders[1].name).toBe('B')
  })

  it('rejects update when the name is taken by another folder', () => {
    expect(updateFolder(folders, 'f2', 'A', 'C:\\A')).toBe(folders)
    expect(updateFolder(folders, 'missing', 'X', 'C:\\X')).toBe(folders)
  })

  it('removes folders by ids', () => {
    expect(removeFolders(folders, ['f1', 'f3']).map((item) => item.id)).toEqual(['f2'])
    expect(removeFolders(folders, [])).toBe(folders)
  })

  describe('moveFolderToIndex', () => {
    it('returns the same array when nothing changes', () => {
      expect(moveFolderToIndex(folders, 'f1', 0)).toBe(folders)
      expect(moveFolderToIndex(folders, 'f2', 2)).toBe(folders)
      expect(moveFolderToIndex(folders, 'missing', 1)).toBe(folders)
      expect(moveFolderToIndex(folders, 'f1', 99)).toBe(folders)
    })

    it('moves down with index adjustment', () => {
      expect(moveFolderToIndex(folders, 'f1', 2).map((item) => item.id)).toEqual(['f2', 'f1', 'f3'])
    })

    it('moves up', () => {
      expect(moveFolderToIndex(folders, 'f3', 0).map((item) => item.id)).toEqual(['f3', 'f1', 'f2'])
    })

    it('moves to the end', () => {
      expect(moveFolderToIndex(folders, 'f1', 3).map((item) => item.id)).toEqual(['f2', 'f3', 'f1'])
    })
  })
})
