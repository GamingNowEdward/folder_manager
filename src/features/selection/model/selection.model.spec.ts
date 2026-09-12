import { describe, expect, it } from 'vitest'
import { idsInRect, rectsOverlap, selectOnly, toggleId } from './selection.model'

describe('selection.model', () => {
  it('selectOnly replaces the selection', () => {
    expect(selectOnly('a')).toEqual(['a'])
  })

  it('toggleId adds and removes ids', () => {
    expect(toggleId([], 'a')).toEqual(['a'])
    expect(toggleId(['a', 'b'], 'a')).toEqual(['b'])
  })

  it('rectsOverlap uses strict intersection', () => {
    const a = { left: 0, top: 0, right: 10, bottom: 10 }
    expect(rectsOverlap(a, { left: 5, top: 5, right: 15, bottom: 15 })).toBe(true)
    expect(rectsOverlap(a, { left: 10, top: 0, right: 20, bottom: 10 })).toBe(false)
    expect(rectsOverlap(a, { left: 0, top: 10, right: 10, bottom: 20 })).toBe(false)
  })

  it('idsInRect returns ids of intersecting cards', () => {
    const cards = [
      { id: 'a', rect: { left: 0, top: 0, right: 10, bottom: 10 } },
      { id: 'b', rect: { left: 20, top: 0, right: 30, bottom: 10 } },
      { id: 'c', rect: { left: 0, top: 20, right: 10, bottom: 30 } }
    ]
    expect(idsInRect(cards, { left: 5, top: 5, right: 25, bottom: 25 })).toEqual(['a', 'b', 'c'])
    expect(idsInRect(cards, { left: 11, top: 0, right: 19, bottom: 10 })).toEqual([])
  })
})
