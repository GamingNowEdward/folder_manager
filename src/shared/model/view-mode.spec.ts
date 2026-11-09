import { describe, expect, it } from 'vitest'
import { DEFAULT_FOLDER_VIEW_MODE, parseViewMode } from './view-mode'

describe('parseViewMode', () => {
  it('识别 list（不区分大小写）', () => {
    expect(parseViewMode('list')).toBe('list')
    expect(parseViewMode('LIST')).toBe('list')
    expect(parseViewMode('List')).toBe('list')
  })

  it('识别 grid', () => {
    expect(parseViewMode('grid')).toBe('grid')
  })

  it('缺失或非法值回落默认视图', () => {
    expect(parseViewMode(null)).toBe(DEFAULT_FOLDER_VIEW_MODE)
    expect(parseViewMode(undefined)).toBe(DEFAULT_FOLDER_VIEW_MODE)
    expect(parseViewMode('')).toBe(DEFAULT_FOLDER_VIEW_MODE)
    expect(parseViewMode('cards')).toBe(DEFAULT_FOLDER_VIEW_MODE)
    expect(parseViewMode(123)).toBe(DEFAULT_FOLDER_VIEW_MODE)
    expect(parseViewMode({ mode: 'list' })).toBe(DEFAULT_FOLDER_VIEW_MODE)
  })
})
