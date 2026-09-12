import { describe, expect, it } from 'vitest'
import { planExternalDrop } from './external-drop'

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
