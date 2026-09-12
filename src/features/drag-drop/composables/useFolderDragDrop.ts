import { computed, onMounted, onUnmounted, reactive, ref, type Ref } from 'vue'
import type { Folder, FolderId } from '@/types'
import { DRAG_THRESHOLD_PX } from '@/shared/constants'
import { resolveDropTargetIndex, type DropCard } from '../model/drop-target'

interface DragDropOptions {
  areaRef: Ref<HTMLElement | null>
  folders: Ref<Folder[]>
  onReorder: (id: FolderId, targetIndex: number) => void
  onSelect: (id: FolderId, additive: boolean) => void
  onDragFinished: () => void
}

export function useFolderDragDrop(options: DragDropOptions) {
  const isDragging = ref(false)
  const dragId = ref<FolderId | ''>('')
  const ghost = reactive({ x: 0, y: 0, width: 0, height: 0 })
  let startX = 0
  let startY = 0
  let offsetX = 0
  let offsetY = 0
  let lastTargetIndex = -1

  const draggedFolder = computed(
    () => options.folders.value.find((folder) => folder.id === dragId.value) ?? null
  )

  function onCardMouseDown(id: FolderId, event: MouseEvent): void {
    dragId.value = id
    startX = event.clientX
    startY = event.clientY
    isDragging.value = false
    lastTargetIndex = -1
  }

  function collectDropCards(): DropCard[] {
    const area = options.areaRef.value
    if (!area) return []
    const cards: DropCard[] = []
    area.querySelectorAll<HTMLElement>('[data-folder-id]').forEach((element) => {
      const id = element.dataset.folderId
      if (!id || id === dragId.value) return
      const index = options.folders.value.findIndex((folder) => folder.id === id)
      if (index < 0) return
      const rect = element.getBoundingClientRect()
      cards.push({
        id,
        index,
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom
      })
    })
    return cards
  }

  function onMouseMove(event: MouseEvent): void {
    if (!dragId.value) return
    const dx = Math.abs(event.clientX - startX)
    const dy = Math.abs(event.clientY - startY)
    if (dx <= DRAG_THRESHOLD_PX && dy <= DRAG_THRESHOLD_PX) return

    if (!isDragging.value) {
      isDragging.value = true
      document.body.style.cursor = 'grabbing'
      const element = options.areaRef.value?.querySelector<HTMLElement>(
        `[data-folder-id="${CSS.escape(dragId.value)}"]`
      )
      if (element) {
        const rect = element.getBoundingClientRect()
        ghost.width = rect.width
        ghost.height = rect.height
        offsetX = event.clientX - rect.left
        offsetY = event.clientY - rect.top
      }
    }

    ghost.x = event.clientX - offsetX
    ghost.y = event.clientY - offsetY

    const targetIndex = resolveDropTargetIndex(
      collectDropCards(),
      event.clientX,
      event.clientY,
      options.folders.value.length
    )
    if (targetIndex !== lastTargetIndex) {
      lastTargetIndex = targetIndex
      options.onReorder(dragId.value, targetIndex)
    }
  }

  function onMouseUp(event: MouseEvent): void {
    if (!dragId.value) return
    const wasDragging = isDragging.value
    const id = dragId.value
    reset()
    if (wasDragging) {
      options.onDragFinished()
    } else {
      options.onSelect(id, event.ctrlKey || event.metaKey)
    }
  }

  function reset(): void {
    dragId.value = ''
    isDragging.value = false
    lastTargetIndex = -1
    document.body.style.cursor = ''
  }

  function onWindowBlur(): void {
    if (dragId.value) reset()
  }

  onMounted(() => {
    document.addEventListener('mousemove', onMouseMove)
    document.addEventListener('mouseup', onMouseUp)
    window.addEventListener('blur', onWindowBlur)
  })

  onUnmounted(() => {
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
    window.removeEventListener('blur', onWindowBlur)
  })

  return { isDragging, dragId, ghost, draggedFolder, onCardMouseDown }
}
