import { ref } from 'vue'
import { useDialogs } from '@/features/dialogs'
import {
  getApiDocumentation,
  getApiSettings,
  setApiPort,
  type ApiSettings
} from '@/infrastructure/tauri/settings'
import { copyText } from '@/infrastructure/tauri/system'
import { useStatusStore } from '@/shared/stores/status.store'

// 模块级单例：设置弹窗状态与数据在 AppShell / DialogHost 之间共享（同 useDialogs 的写法）。
const settings = ref<ApiSettings | null>(null)
const errorMessage = ref('')
const loading = ref(false)
const saving = ref(false)

export function useSettings() {
  const dialogs = useDialogs()
  const statusStore = useStatusStore()

  async function load(): Promise<void> {
    loading.value = true
    errorMessage.value = ''
    try {
      settings.value = await getApiSettings()
    } catch (error) {
      errorMessage.value = '读取设置失败'
      console.error('读取 API 设置失败:', error)
    } finally {
      loading.value = false
    }
  }

  async function open(): Promise<void> {
    dialogs.openSettingsDialog()
    await load()
  }

  function close(): void {
    dialogs.closeSettingsDialog()
  }

  async function save(rawPort: string): Promise<void> {
    const port = Number(rawPort.trim())
    // 只做最基本的输入约束，业务规则以 Rust 端为准。
    if (!Number.isInteger(port) || port < 1 || port > 65535) {
      errorMessage.value = '端口必须是 1–65535 的整数'
      return
    }
    saving.value = true
    errorMessage.value = ''
    try {
      settings.value = await setApiPort(port)
      statusStore.setStatus(`已保存端口 ${port}，重启应用后生效`)
    } catch (error) {
      errorMessage.value = '保存失败，请重试'
      console.error('保存 API 端口失败:', error)
    } finally {
      saving.value = false
    }
  }

  async function copyDocumentation(): Promise<void> {
    try {
      const documentation = await getApiDocumentation()
      await copyText(documentation)
      statusStore.setStatus('已复制 Agent 接入文档')
    } catch (error) {
      statusStore.setStatus('复制 Agent 接入文档失败')
      console.error('复制接口文档失败:', error)
    }
  }

  return {
    settings,
    errorMessage,
    loading,
    saving,
    open,
    close,
    save,
    copyDocumentation
  }
}
