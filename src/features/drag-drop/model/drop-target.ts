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

/**
 * 列表视图的插入位置：按行垂直中心判定。
 * 指针在行上半 → 插到该行之前；下半 → 继续看下一行（即插到其后）。
 * 网格视图的左右半区判定对整行宽度的列表不适用，因此单独实现。
 */
export function resolveListDropTargetIndex(
  rows: DropCard[],
  pointerY: number,
  fallbackIndex: number
): number {
  if (rows.length === 0) return 0

  const sorted = [...rows].sort((a, b) => a.top - b.top)
  for (const row of sorted) {
    const middle = (row.top + row.bottom) / 2
    if (pointerY < middle) return row.index
  }

  return fallbackIndex
}
