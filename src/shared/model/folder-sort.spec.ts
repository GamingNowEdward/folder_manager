import { describe, expect, it } from 'vitest'
import { DEFAULT_FOLDER_SORT, parseFolderSort, serializeFolderSort } from './folder-sort'

describe('parseFolderSort', () => {
  it('没有存储时回落默认排序', () => {
    expect(parseFolderSort(null)).toEqual(DEFAULT_FOLDER_SORT)
    expect(parseFolderSort(undefined)).toEqual(DEFAULT_FOLDER_SORT)
  })

  it('解析合法的 JSON 字符串', () => {
    expect(parseFolderSort('{"field":"name","direction":"desc"}')).toEqual({
      field: 'name',
      direction: 'desc'
    })
  })

  it('接受已经解析过的对象', () => {
    expect(parseFolderSort({ field: 'created', direction: 'asc' })).toEqual({
      field: 'created',
      direction: 'asc'
    })
  })

  it('只回落缺失或非法的那一项', () => {
    expect(parseFolderSort('{"field":"name"}')).toEqual({ field: 'name', direction: 'asc' })
    expect(parseFolderSort('{"direction":"desc"}')).toEqual({ field: 'custom', direction: 'desc' })
  })

  it('未知字段、非 JSON、非对象都回落默认', () => {
    for (const raw of ['{"field":"size","direction":"asc"}', 'not json', '[]', '"name"', 42]) {
      expect(parseFolderSort(raw)).toEqual(DEFAULT_FOLDER_SORT)
    }
  })
})

describe('serializeFolderSort', () => {
  it('序列化后可以原样解析回来', () => {
    const sort = { field: 'created' as const, direction: 'desc' as const }
    expect(parseFolderSort(serializeFolderSort(sort))).toEqual(sort)
  })
})
