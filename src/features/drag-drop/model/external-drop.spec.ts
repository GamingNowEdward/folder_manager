import { describe, expect, it } from 'vitest'
import { describeExternalDrop, planExternalDrop } from './external-drop'

describe('planExternalDrop', () => {
  it('accepts new folders and derives names from paths', () => {
    const plan = planExternalDrop(['C:\\Work\\Alpha', 'C:\\Work\\Beta\\'], new Set())
    expect(plan.accepted).toEqual([
      { name: 'Alpha', path: 'C:\\Work\\Alpha' },
      { name: 'Beta', path: 'C:\\Work\\Beta\\' }
    ])
    expect(plan.skipped).toBe(0)
  })

  it('skips duplicates in existing names and within the batch', () => {
    const plan = planExternalDrop(['C:\\Alpha', 'D:\\Alpha', 'C:\\Beta'], new Set(['Beta']))
    expect(plan.accepted.map((item) => item.name)).toEqual(['Alpha'])
    expect(plan.skipped).toBe(2)
  })

  it('ignores empty names without counting them as skipped', () => {
    const plan = planExternalDrop(['', '///'], new Set())
    expect(plan.accepted).toEqual([])
    expect(plan.skipped).toBe(0)
  })
})

describe('describeExternalDrop', () => {
  it('reports added folders only', () => {
    expect(describeExternalDrop({ added: 2, duplicated: 0, files: 0, invalid: 0 })).toBe(
      '▶ 已添加 2 个文件夹'
    )
  })

  it('appends the duplicated count', () => {
    expect(describeExternalDrop({ added: 2, duplicated: 1, files: 0, invalid: 0 })).toBe(
      '▶ 已添加 2 个文件夹，1 个因重名跳过'
    )
  })

  it('appends ignored files and invalid paths', () => {
    expect(describeExternalDrop({ added: 1, duplicated: 0, files: 2, invalid: 3 })).toBe(
      '▶ 已添加 1 个文件夹，忽略 2 个文件，3 项无法访问'
    )
  })

  it('reports duplicates when nothing was added', () => {
    expect(describeExternalDrop({ added: 0, duplicated: 3, files: 0, invalid: 0 })).toBe(
      '✕ 3 个文件夹因重名被跳过'
    )
  })

  it('reports only files', () => {
    expect(describeExternalDrop({ added: 0, duplicated: 0, files: 1, invalid: 0 })).toBe(
      '✕ 已忽略 1 个文件（只支持文件夹）'
    )
  })

  it('reports only invalid paths', () => {
    expect(describeExternalDrop({ added: 0, duplicated: 0, files: 0, invalid: 2 })).toBe(
      '✕ 2 项无法访问（不存在或没有权限）'
    )
  })

  it('reports files and invalid paths together', () => {
    expect(describeExternalDrop({ added: 0, duplicated: 0, files: 1, invalid: 1 })).toBe(
      '✕ 已忽略 1 个文件，1 项无法访问'
    )
  })

  it('returns an empty message when nothing happened', () => {
    expect(describeExternalDrop({ added: 0, duplicated: 0, files: 0, invalid: 0 })).toBe('')
  })
})
