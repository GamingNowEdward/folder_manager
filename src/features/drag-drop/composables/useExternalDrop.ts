import { onMounted, onUnmounted } from 'vue'
import { useFolderStore } from '@/features/folders'
import { useProjectStore } from '@/features/projects'
import { filterDroppedDirectories } from '@/infrastructure/tauri/system'
import { onWindowFileDrop } from '@/infrastructure/tauri/window'
import { useStatusStore } from '@/shared/stores/status.store'
import { describeExternalDrop, planExternalDrop } from '../model/external-drop'

export function useExternalDrop(): void {
  const projectStore = useProjectStore()
  const folderStore = useFolderStore()
  const statusStore = useStatusStore()
  let unlisten: (() => void) | null = null
  let requestId = 0

  async function handleDrop(rawPaths: string[]): Promise<void> {
    if (!projectStore.currentProject) {
      statusStore.setStatus('请先创建或选择一个项目')
      return
    }

    const currentRequest = ++requestId
    let filtered
    try {
      // 文件 / 目录判定只能由 Rust 完成：WebView 没有文件系统访问能力。
      filtered = await filterDroppedDirectories(rawPaths)
    } catch (error) {
      console.error('读取拖入内容失败:', error)
      statusStore.setStatus('✕ 无法读取拖入的内容')
      return
    }

    // 连续拖入时只让最后一次生效；等待期间项目也可能已被切换或删除。
    if (currentRequest !== requestId || !projectStore.currentProject) return

    const existing = projectStore.currentFolders.map((folder) => folder.name)
    const { accepted, skipped } = planExternalDrop(filtered.directories, existing)
    if (accepted.length > 0) folderStore.addFolders(accepted)

    const message = describeExternalDrop({
      added: accepted.length,
      duplicated: skipped,
      files: filtered.files,
      invalid: filtered.invalid
    })
    if (message) statusStore.setStatus(message)
  }

  onMounted(async () => {
    unlisten = await onWindowFileDrop((paths) => {
      void handleDrop(paths)
    })
  })

  onUnmounted(() => {
    unlisten?.()
    unlisten = null
  })
}
