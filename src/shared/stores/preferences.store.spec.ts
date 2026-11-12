import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { FOLDER_SORT_STORAGE_KEY, VIEW_MODE_STORAGE_KEY } from '@/shared/constants'
import {
  readStoredFolderSort,
  readStoredViewMode,
  usePreferencesStore,
  writeStoredFolderSort,
  writeStoredViewMode,
  type StorageLike
} from './preferences.store'

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

describe('readStoredFolderSort', () => {
  it('没有存储时回落默认排序', () => {
    expect(readStoredFolderSort(null)).toEqual({ field: 'custom', direction: 'asc' })
  })

  it('读取已保存的排序', () => {
    const storage = fakeStorage({
      [FOLDER_SORT_STORAGE_KEY]: '{"field":"created","direction":"desc"}'
    })
    expect(readStoredFolderSort(storage)).toEqual({ field: 'created', direction: 'desc' })
  })

  it('脏数据与读取异常都回落默认排序', () => {
    expect(readStoredFolderSort(fakeStorage({ [FOLDER_SORT_STORAGE_KEY]: 'oops' }))).toEqual({
      field: 'custom',
      direction: 'asc'
    })
    const broken: StorageLike = {
      getItem: () => {
        throw new Error('denied')
      },
      setItem: () => undefined
    }
    expect(readStoredFolderSort(broken)).toEqual({ field: 'custom', direction: 'asc' })
  })
})

describe('writeStoredFolderSort', () => {
  it('写入到约定 key 且可解析回来', () => {
    const storage = fakeStorage()
    writeStoredFolderSort({ field: 'name', direction: 'desc' }, storage)
    expect(storage.data[FOLDER_SORT_STORAGE_KEY]).toBe('{"field":"name","direction":"desc"}')
  })

  it('没有存储或写入异常时静默忽略', () => {
    expect(() => writeStoredFolderSort({ field: 'name', direction: 'asc' }, null)).not.toThrow()
    const broken: StorageLike = {
      getItem: () => null,
      setItem: () => {
        throw new Error('quota')
      }
    }
    expect(() => writeStoredFolderSort({ field: 'name', direction: 'asc' }, broken)).not.toThrow()
  })
})

describe('usePreferencesStore 排序操作', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('切换排序字段时保留方向', () => {
    const store = usePreferencesStore()
    store.setFolderSort({ field: 'created', direction: 'desc' })

    store.setSortField('name')

    expect(store.folderSort).toEqual({ field: 'name', direction: 'desc' })
  })

  it('在自动排序下切换方向', () => {
    const store = usePreferencesStore()
    store.setSortField('name')

    store.toggleSortDirection()
    expect(store.folderSort.direction).toBe('desc')
    store.toggleSortDirection()
    expect(store.folderSort.direction).toBe('asc')
  })

  it('自定义排序不响应方向切换', () => {
    const store = usePreferencesStore()
    expect(store.folderSort.field).toBe('custom')

    store.toggleSortDirection()

    expect(store.folderSort).toEqual({ field: 'custom', direction: 'asc' })
  })

  it('写入非法排序时被归一到默认值', () => {
    const store = usePreferencesStore()

    store.setFolderSort({ field: 'size' as never, direction: 'sideways' as never })

    expect(store.folderSort).toEqual({ field: 'custom', direction: 'asc' })
  })
})
