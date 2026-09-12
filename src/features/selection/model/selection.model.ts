export function selectOnly(id: string): string[] {
  return [id]
}

export function toggleId(ids: string[], id: string): string[] {
  return ids.includes(id) ? ids.filter((item) => item !== id) : [...ids, id]
}

export interface RectLike {
  left: number
  top: number
  right: number
  bottom: number
}

export interface CardRect {
  id: string
  rect: RectLike
}

export function rectsOverlap(a: RectLike, b: RectLike): boolean {
  return a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top
}

export function idsInRect(cards: CardRect[], box: RectLike): string[] {
  return cards.filter((card) => rectsOverlap(card.rect, box)).map((card) => card.id)
}
