import { describe, expect, it } from 'vitest'
import { resolveFolderShortcut } from './shortcut.model'
import type { KeyLike } from '@/shared/utils/keyboard'

function key(overrides: Partial<KeyLike>): KeyLike {
  return { key: '', ...overrides }
}

describe('resolveFolderShortcut', () => {
  it('Delete 映射为删除', () => {
    expect(resolveFolderShortcut(key({ key: 'Delete' }))).toBe('delete')
  })

  it('Ctrl+A 与 Cmd+A 映射为全选', () => {
    expect(resolveFolderShortcut(key({ key: 'a', ctrlKey: true }))).toBe('select-all')
    expect(resolveFolderShortcut(key({ key: 'A', ctrlKey: true }))).toBe('select-all')
    expect(resolveFolderShortcut(key({ key: 'a', metaKey: true }))).toBe('select-all')
  })

  it('裸按 A 不触发全选', () => {
    expect(resolveFolderShortcut(key({ key: 'a' }))).toBeNull()
  })

  it('带 Alt / Shift 的组合键不触发全选', () => {
    expect(resolveFolderShortcut(key({ key: 'a', ctrlKey: true, shiftKey: true }))).toBeNull()
    expect(resolveFolderShortcut(key({ key: 'a', ctrlKey: true, altKey: true }))).toBeNull()
  })

  it('输入法组合中的按键不触发', () => {
    expect(resolveFolderShortcut(key({ key: 'a', ctrlKey: true, isComposing: true }))).toBeNull()
    expect(resolveFolderShortcut(key({ key: 'Delete', keyCode: 229 }))).toBeNull()
  })

  it('其他按键不触发', () => {
    expect(resolveFolderShortcut(key({ key: 'b', ctrlKey: true }))).toBeNull()
    expect(resolveFolderShortcut(key({ key: 'Backspace' }))).toBeNull()
  })
})
