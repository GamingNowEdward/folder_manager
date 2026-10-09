import { invoke } from '@tauri-apps/api/core'

export const COMMANDS = {
  loadConfig: 'load_config',
  saveConfig: 'save_config',
  openFolder: 'open_folder',
  getApiSettings: 'get_api_settings',
  setApiPort: 'set_api_port',
  apiDocumentation: 'api_documentation'
} as const

export function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args)
}
