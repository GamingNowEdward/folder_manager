import { describe, expect, it } from 'vitest'
import { resolveDialogKey } from './dialog-key.model'
import type { KeyLike } from '@/shared/utils/keyboard'

function key(overrides: Partial<KeyLike>): KeyLike {
  return { key: '', ...overrides }
}

describe('resolveDialogKey', () => {
  it('Enter 确认', () => {
    expect(resolveDialogKey(key({ key: 'Enter' }), false)).toBe('confirm')
  })

  it('Escape 取消', () => {
    expect(resolveDialogKey(key({ key: 'Escape' }), false)).toBe('cancel')
  })

  it('关闭快捷确认时忽略 Y / N', () => {
    expect(resolveDialogKey(key({ key: 'y' }), false)).toBeNull()
    expect(resolveDialogKey(key({ key: 'n' }), false)).toBeNull()
  })

  it('开启快捷确认时 Y 确认、N 取消，且不区分大小写', () => {
    expect(resolveDialogKey(key({ key: 'y' }), true)).toBe('confirm')
    expect(resolveDialogKey(key({ key: 'Y' }), true)).toBe('confirm')
    expect(resolveDialogKey(key({ key: 'n' }), true)).toBe('cancel')
    expect(resolveDialogKey(key({ key: 'N' }), true)).toBe('cancel')
  })

  it('带 Ctrl / Alt / Meta 的字母键不触发快捷确认', () => {
    expect(resolveDialogKey(key({ key: 'y', ctrlKey: true }), true)).toBeNull()
    expect(resolveDialogKey(key({ key: 'n', altKey: true }), true)).toBeNull()
    expect(resolveDialogKey(key({ key: 'y', metaKey: true }), true)).toBeNull()
  })

  it('输入法组合中的 Enter 不确认（中文选词不误提交）', () => {
    expect(resolveDialogKey(key({ key: 'Enter', isComposing: true }), false)).toBeNull()
    expect(resolveDialogKey(key({ key: 'Enter', keyCode: 229 }), false)).toBeNull()
    expect(resolveDialogKey(key({ key: 'y', isComposing: true }), true)).toBeNull()
  })

  it('其他按键不产生动作', () => {
    expect(resolveDialogKey(key({ key: 'a' }), true)).toBeNull()
    expect(resolveDialogKey(key({ key: 'Tab' }), true)).toBeNull()
  })
})
