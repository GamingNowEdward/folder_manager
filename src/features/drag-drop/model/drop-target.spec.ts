import { describe, expect, it } from 'vitest'
import { resolveDropTargetIndex, resolveListDropTargetIndex, type DropCard } from './drop-target'

function card(
  id: string,
  index: number,
  left: number,
  top: number,
  width = 100,
  height = 50
): DropCard {
  return { id, index, left, top, right: left + width, bottom: top + height }
}

describe('resolveDropTargetIndex', () => {
  it('returns 0 when no cards remain', () => {
    expect(resolveDropTargetIndex([], 0, 0, 1)).toBe(0)
  })

  it('returns the card index when the pointer is over its left half', () => {
    const cards = [card('a', 0, 0, 0), card('b', 1, 110, 0), card('c', 2, 220, 0)]
    expect(resolveDropTargetIndex(cards, 120, 25, 3)).toBe(1)
  })

  it('falls back when the pointer is over the right half of the last card', () => {
    const cards = [card('a', 0, 0, 0), card('b', 1, 110, 0)]
    expect(resolveDropTargetIndex(cards, 200, 25, 3)).toBe(3)
  })

  it('returns the first card index when the pointer is above it', () => {
    const cards = [card('a', 0, 0, 100), card('b', 1, 110, 100)]
    expect(resolveDropTargetIndex(cards, 50, 50, 3)).toBe(0)
  })

  it('handles wrapped rows', () => {
    const cards = [card('a', 0, 0, 0), card('b', 1, 110, 0), card('c', 2, 0, 60)]
    expect(resolveDropTargetIndex(cards, 10, 70, 3)).toBe(2)
  })

  it('returns the original index of the card in the full array', () => {
    const cards = [card('a', 0, 0, 0), card('c', 2, 110, 0)]
    expect(resolveDropTargetIndex(cards, 120, 25, 3)).toBe(2)
  })
})

describe('resolveListDropTargetIndex', () => {
  // 列表行：全宽、逐行向下排列
  function row(id: string, index: number, top: number): DropCard {
    return { id, index, left: 0, top, right: 800, bottom: top + 38 }
  }

  it('returns 0 when no rows remain', () => {
    expect(resolveListDropTargetIndex([], 100, 5)).toBe(0)
  })

  it('inserts before the row when the pointer is in its upper half', () => {
    const rows = [row('a', 0, 0), row('b', 1, 40), row('c', 2, 80)]
    expect(resolveListDropTargetIndex(rows, 10, 3)).toBe(0)
    expect(resolveListDropTargetIndex(rows, 50, 3)).toBe(1)
  })

  it('inserts after the row when the pointer is in its lower half', () => {
    const rows = [row('a', 0, 0), row('b', 1, 40)]
    // 第一行的下半区 → 落到第二行之前，即 index 1
    expect(resolveListDropTargetIndex(rows, 30, 3)).toBe(1)
  })

  it('falls back past the last row', () => {
    const rows = [row('a', 0, 0), row('b', 1, 40)]
    expect(resolveListDropTargetIndex(rows, 200, 3)).toBe(3)
  })

  it('uses visual order even when rows are passed out of order', () => {
    const rows = [row('b', 1, 40), row('a', 0, 0)]
    expect(resolveListDropTargetIndex(rows, 10, 2)).toBe(0)
  })
})
