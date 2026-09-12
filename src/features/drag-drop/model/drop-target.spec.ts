import { describe, expect, it } from 'vitest'
import { resolveDropTargetIndex, type DropCard } from './drop-target'

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
