export interface DropCard {
  id: string
  index: number
  left: number
  top: number
  right: number
  bottom: number
}

const ROW_THRESHOLD_PX = 5

export function resolveDropTargetIndex(
  cards: DropCard[],
  pointerX: number,
  pointerY: number,
  fallbackIndex: number
): number {
  if (cards.length === 0) return 0

  const rows: number[][] = []
  const rowTops: number[] = []

  for (let i = 0; i < cards.length; i++) {
    const card = cards[i]
    const top = Math.round(card.top)
    let rowIndex = rowTops.findIndex((rowTop) => Math.abs(rowTop - top) < ROW_THRESHOLD_PX)
    if (rowIndex < 0) {
      rowIndex = rows.length
      rows.push([])
      rowTops.push(card.top)
    }
    rows[rowIndex].push(i)
  }

  for (const row of rows) {
    row.sort((a, b) => cards[a].left - cards[b].left)
  }

  const visualOrder = rows.flat()

  for (const cardIndex of visualOrder) {
    const card = cards[cardIndex]
    if (pointerY >= card.top && pointerY <= card.bottom) {
      if (pointerX < card.left + (card.right - card.left) / 2) {
        return card.index
      }
      continue
    }
    if (pointerY < card.top) {
      return card.index
    }
  }

  return fallbackIndex
}
