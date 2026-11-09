import { describe, expect, it } from 'vitest'
import { VIEW_MODE_STORAGE_KEY } from '@/shared/constants'
import { readStoredViewMode, writeStoredViewMode, type StorageLike } from './preferences.store'

function fakeStorage(initial: Record<string, string> = {}): StorageLike & {
  data: Record<string, string>
} {
  const data = { ...initial }
  return {
    data,
    getItem: (key) => data[key] ?? null,
    setItem: (key, value) => {
      data[key] = value
    }
  }
}

describe('readStoredViewMode', () => {
  it('没有存储时回落默认视图', () => {
    expect(readStoredViewMode(null)).toBe('grid')
  })

  it('读取已保存的视图', () => {
    expect(readStoredViewMode(fakeStorage({ [VIEW_MODE_STORAGE_KEY]: 'list' }))).toBe('list')
  })

  it('存储里是脏数据时回落默认视图', () => {
    expect(readStoredViewMode(fakeStorage({ [VIEW_MODE_STORAGE_KEY]: 'weird' }))).toBe('grid')
  })

  it('读取抛异常时不向外传播', () => {
    const broken: StorageLike = {
      getItem: () => {
        throw new Error('denied')
      },
      setItem: () => undefined
    }
    expect(readStoredViewMode(broken)).toBe('grid')
  })
})

describe('writeStoredViewMode', () => {
  it('写入到约定 key', () => {
    const storage = fakeStorage()
    writeStoredViewMode('list', storage)
    expect(storage.data[VIEW_MODE_STORAGE_KEY]).toBe('list')
  })

  it('没有存储或写入抛异常时静默忽略', () => {
    expect(() => writeStoredViewMode('list', null)).not.toThrow()
    const broken: StorageLike = {
      getItem: () => null,
      setItem: () => {
        throw new Error('quota')
      }
    }
    expect(() => writeStoredViewMode('list', broken)).not.toThrow()
  })
})
