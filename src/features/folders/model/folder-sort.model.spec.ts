import { describe, expect, it } from 'vitest'
import type { Folder } from '@/types'
import { sortFolders } from './folder-sort.model'

function folder(id: string, name: string, createdAt: number): Folder {
  return { id, name, path: `C:\\${name}`, createdAt }
}

const byCreated = [folder('f1', 'B', 100), folder('f2', 'A', 200), folder('f3', 'C', 300)]

describe('sortFolders', () => {
  it('custom 保持底层顺序，并忽略方向', () => {
    expect(
      sortFolders(byCreated, { field: 'custom', direction: 'asc' }).map((it) => it.id)
    ).toEqual(['f1', 'f2', 'f3'])
    expect(
      sortFolders(byCreated, { field: 'custom', direction: 'desc' }).map((it) => it.id)
    ).toEqual(['f1', 'f2', 'f3'])
  })

  it('不修改入参数组', () => {
    const source = [...byCreated]
    sortFolders(source, { field: 'name', direction: 'asc' })
    expect(source).toEqual(byCreated)
  })

  it('name 忽略大小写', () => {
    const list = [folder('f1', 'beta', 1), folder('f2', 'Alpha', 2), folder('f3', 'ALPHA2', 3)]
    expect(sortFolders(list, { field: 'name', direction: 'asc' }).map((it) => it.name)).toEqual([
      'Alpha',
      'ALPHA2',
      'beta'
    ])
  })

  it('name 按拼音排列中文', () => {
    const list = [folder('f1', '长城', 1), folder('f2', '百度', 2), folder('f3', '阿里', 3)]
    expect(sortFolders(list, { field: 'name', direction: 'asc' }).map((it) => it.name)).toEqual([
      '阿里',
      '百度',
      '长城'
    ])
  })

  it('name 对数字做自然排序', () => {
    const list = [folder('f1', 'RDR10', 1), folder('f2', 'RDR2', 2), folder('f3', 'RDR1', 3)]
    expect(sortFolders(list, { field: 'name', direction: 'asc' }).map((it) => it.name)).toEqual([
      'RDR1',
      'RDR2',
      'RDR10'
    ])
  })

  it('name 同值时按添加时间稳定排序', () => {
    const list = [folder('f3', 'same', 300), folder('f1', 'same', 100), folder('f2', 'same', 200)]
    expect(sortFolders(list, { field: 'name', direction: 'asc' }).map((it) => it.id)).toEqual([
      'f1',
      'f2',
      'f3'
    ])
  })

  it('name desc 为反序', () => {
    const list = [folder('f1', 'A', 1), folder('f2', 'B', 2), folder('f3', 'C', 3)]
    expect(sortFolders(list, { field: 'name', direction: 'desc' }).map((it) => it.name)).toEqual([
      'C',
      'B',
      'A'
    ])
  })

  it('created 按添加时间正序 / 倒序', () => {
    expect(
      sortFolders(byCreated, { field: 'created', direction: 'asc' }).map((it) => it.id)
    ).toEqual(['f1', 'f2', 'f3'])
    expect(
      sortFolders(byCreated, { field: 'created', direction: 'desc' }).map((it) => it.id)
    ).toEqual(['f3', 'f2', 'f1'])
  })

  it('created 同时间时按名称、再按 id 兜底', () => {
    const list = [folder('b', 'Beta', 100), folder('a', 'Alpha', 100)]
    expect(sortFolders(list, { field: 'created', direction: 'asc' }).map((it) => it.id)).toEqual([
      'a',
      'b'
    ])
  })
})
