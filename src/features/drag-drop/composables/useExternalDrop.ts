import { onMounted, onUnmounted } from 'vue'
import { useFolderStore } from '@/features/folders'
import { useProjectStore } from '@/features/projects'
import { onWindowFileDrop } from '@/infrastructure/tauri/window'
import { useStatusStore } from '@/shared/stores/status.store'
import { planExternalDrop } from '../model/external-drop'

export function useExternalDrop(): void {
  const projectStore = useProjectStore()
  const folderStore = useFolderStore()
  const statusStore = useStatusStore()
  let unlisten: (() => void) | null = null

  function handleDrop(paths: string[]): void {
    if (!projectStore.currentProject) {
      statusStore.setStatus('请先创建或选择一个项目')
      return
    }
    const existing = projectStore.currentFolders.map((folder) => folder.name)
    const { accepted, skipped } = planExternalDrop(paths, existing)
    if (accepted.length > 0) folderStore.addFolders(accepted)
    const added = accepted.length
    if (added > 0 && skipped > 0) {
      statusStore.setStatus(`▶ 已添加 ${added} 个文件夹，${skipped} 个因重名跳过`)
    } else if (added > 0) {
      statusStore.setStatus(`▶ 已添加 ${added} 个文件夹`)
    } else if (skipped > 0) {
      statusStore.setStatus(`✕ ${skipped} 个文件夹因重名被跳过`)
    }
  }

  onMounted(async () => {
    unlisten = await onWindowFileDrop(handleDrop)
  })

  onUnmounted(() => {
    unlisten?.()
    unlisten = null
  })
}
