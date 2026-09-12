import { computed, onMounted, onUnmounted, ref, type Ref } from 'vue'
import type { FolderId } from '@/types'
import { idsInRect, type CardRect } from '../model/selection.model'

interface BoxSelectionOptions {
  areaRef: Ref<HTMLElement | null>
  onSelect: (ids: FolderId[]) => void
}

export function useBoxSelection(options: BoxSelectionOptions) {
  const isBoxSelecting = ref(false)
  const boxRect = ref<{ left: number; top: number; width: number; height: number } | null>(null)
  let startX = 0
  let startY = 0

  const boxOverlayStyle = computed(() => {
    const rect = boxRect.value
    const area = options.areaRef.value
    if (!rect || !area) return { display: 'none' }
    const areaRect = area.getBoundingClientRect()
    return {
      left: `${rect.left - areaRect.left + area.scrollLeft}px`,
      top: `${rect.top - areaRect.top + area.scrollTop}px`,
      width: `${rect.width}px`,
      height: `${rect.height}px`
    }
  })

  function collectCards(): CardRect[] {
    const area = options.areaRef.value
    if (!area) return []
    const cards: CardRect[] = []
    area.querySelectorAll<HTMLElement>('[data-folder-id]').forEach((element) => {
      const id = element.dataset.folderId
      if (!id) return
      const rect = element.getBoundingClientRect()
      cards.push({
        id,
        rect: { left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom }
      })
    })
    return cards
  }

  function onAreaMouseDown(event: MouseEvent): void {
    if (event.button !== 0) return
    const target = event.target as HTMLElement
    if (target.closest('[data-folder-id]') || target.closest('.add-card')) return
    event.preventDefault()
    options.onSelect([])
    isBoxSelecting.value = true
    startX = event.clientX
    startY = event.clientY
  }

  function onMouseMove(event: MouseEvent): void {
    if (!isBoxSelecting.value) return
    const x1 = Math.min(startX, event.clientX)
    const y1 = Math.min(startY, event.clientY)
    const x2 = Math.max(startX, event.clientX)
    const y2 = Math.max(startY, event.clientY)
    boxRect.value = { left: x1, top: y1, width: x2 - x1, height: y2 - y1 }
    options.onSelect(idsInRect(collectCards(), { left: x1, top: y1, right: x2, bottom: y2 }))
  }

  function onMouseUp(): void {
    if (!isBoxSelecting.value) return
    isBoxSelecting.value = false
    boxRect.value = null
  }

  function cancel(): void {
    isBoxSelecting.value = false
    boxRect.value = null
  }

  onMounted(() => {
    document.addEventListener('mousemove', onMouseMove)
    document.addEventListener('mouseup', onMouseUp)
  })

  onUnmounted(() => {
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  })

  return { isBoxSelecting, boxRect, boxOverlayStyle, onAreaMouseDown, cancel }
}
