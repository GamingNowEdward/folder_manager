import { describe, expect, it } from 'vitest'
import type { Folder } from '@/types'
import {
  findFoldersByInitial,
  folderInitial,
  resolveEnterAction,
  resolveQuickNavKey,
  resolveQuickNavTarget
} from './quick-nav.model'

function folder(id: string, name: string, path = `C:\\${name}`): Folder {
  return { id, name, path, createdAt: 0 }
}

describe('resolveQuickNavKey', () => {
  it('接受单个英文字母或数字并归一为小写', () => {
    expect(resolveQuickNavKey({ key: 'a' })).toBe('a')
    expect(resolveQuickNavKey({ key: 'R' })).toBe('r')
    expect(resolveQuickNavKey({ key: '7' })).toBe('7')
  })

  it('允许 Shift（大写字母仍按同一首字母匹配）', () => {
    expect(resolveQuickNavKey({ key: 'R', shiftKey: true })).toBe('r')
  })

  it('拒绝 Ctrl / Alt / Meta 组合，避免抢占系统与编辑器快捷键', () => {
    expect(resolveQuickNavKey({ key: 'a', ctrlKey: true })).toBeNull()
    expect(resolveQuickNavKey({ key: 'a', altKey: true })).toBeNull()
    expect(resolveQuickNavKey({ key: 'a', metaKey: true })).toBeNull()
  })

  it('拒绝输入法组合中的按键', () => {
    expect(resolveQuickNavKey({ key: 'r', isComposing: true })).toBeNull()
    expect(resolveQuickNavKey({ key: 'r', keyCode: 229 })).toBeNull()
  })

  it('拒绝功能键、中文、重音字母、空白与多字符键', () => {
    for (const key of ['Enter', 'F1', 'Escape', '中', 'é', 'É', ' ', '']) {
      expect(resolveQuickNavKey({ key })).toBeNull()
    }
  })
})

describe('folderInitial / findFoldersByInitial', () => {
  it('取显示名 trim 后的首字符并小写', () => {
    expect(folderInitial(folder('f1', '  Red Dead'))).toBe('r')
    expect(folderInitial(folder('f2', 'RDR2'))).toBe('r')
    expect(folderInitial(folder('f3', '3D Assets'))).toBe('3')
    expect(folderInitial(folder('f4', '工作资料'))).toBe('工')
    expect(folderInitial(folder('f5', ''))).toBe('')
    expect(folderInitial(folder('f6', '   '))).toBe('')
  })

  it('只匹配显示名（备注），不匹配路径末段', () => {
    const list = [
      folder('f1', '工作资料', 'C:\\work\\RDR2'),
      folder('f2', 'RDR2 备注', 'C:\\work\\other'),
      folder('f3', 'Resident Evil')
    ]

    expect(findFoldersByInitial(list, 'r').map((it) => it.id)).toEqual(['f2', 'f3'])
    // 备注是中文时，目录名首字母不参与匹配
    expect(findFoldersByInitial(list, 'g')).toEqual([])
  })

  it('保持传入顺序（即当前显示顺序）', () => {
    const list = [folder('f3', 'R3'), folder('f1', 'R1'), folder('f2', 'R2')]
    expect(findFoldersByInitial(list, 'r').map((it) => it.id)).toEqual(['f3', 'f1', 'f2'])
  })
})

const folders = [
  folder('b', 'Beta'),
  folder('s', 'Sierra'),
  folder('r1', 'Red Dead Redemption 2'),
  folder('r2', 'Resident Evil'),
  folder('d', '3D Assets')
]

describe('resolveQuickNavTarget', () => {
  it('首次按键定位到第一个匹配项', () => {
    expect(resolveQuickNavTarget(folders, 'r', null, false)?.id).toBe('r1')
  })

  it('重复按同一个键定位到下一个匹配项', () => {
    expect(resolveQuickNavTarget(folders, 'r', 'r1', true)?.id).toBe('r2')
  })

  it('到末尾后回绕到该字符的第一个', () => {
    expect(resolveQuickNavTarget(folders, 'r', 'r2', true)?.id).toBe('r1')
  })

  it('只有一个候选时重复按保持该项', () => {
    expect(resolveQuickNavTarget(folders, '3', 'd', true)?.id).toBe('d')
  })

  it('当前选中项不在候选中时从第一个重新开始', () => {
    expect(resolveQuickNavTarget(folders, 'r', 's', true)?.id).toBe('r1')
  })

  it('没有匹配项时返回 null', () => {
    expect(resolveQuickNavTarget(folders, 'z', null, false)).toBeNull()
    expect(resolveQuickNavTarget([], 'r', null, false)).toBeNull()
  })

  it('大小写不敏感', () => {
    const list = [folder('x', 'Alpha'), folder('y', 'apple')]
    expect(resolveQuickNavTarget(list, 'A', null, false)?.id).toBe('x')
    expect(resolveQuickNavTarget(list, 'A', 'x', true)?.id).toBe('y')
  })
})

describe('resolveEnterAction', () => {
  it('Enter 打开、Shift+Enter 复制', () => {
    expect(resolveEnterAction({ key: 'Enter' })).toBe('open')
    expect(resolveEnterAction({ key: 'Enter', shiftKey: true })).toBe('copy')
  })

  it('拒绝带 Ctrl / Alt / Meta 的组合与输入法组合', () => {
    expect(resolveEnterAction({ key: 'Enter', ctrlKey: true })).toBeNull()
    expect(resolveEnterAction({ key: 'Enter', altKey: true })).toBeNull()
    expect(resolveEnterAction({ key: 'Enter', metaKey: true })).toBeNull()
    expect(resolveEnterAction({ key: 'Enter', isComposing: true })).toBeNull()
    expect(resolveEnterAction({ key: 'Enter', keyCode: 229 })).toBeNull()
  })

  it('其他键不响应', () => {
    expect(resolveEnterAction({ key: 'a' })).toBeNull()
    expect(resolveEnterAction({ key: 'Escape' })).toBeNull()
  })
})
