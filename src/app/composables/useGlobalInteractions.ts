import { onMounted, onUnmounted } from 'vue'

export function useGlobalInteractions(): void {
  function onContextMenu(event: MouseEvent): void {
    if (!(event.target as HTMLElement).closest('input, textarea')) {
      event.preventDefault()
    }
  }

  onMounted(() => document.addEventListener('contextmenu', onContextMenu))
  onUnmounted(() => document.removeEventListener('contextmenu', onContextMenu))
}
