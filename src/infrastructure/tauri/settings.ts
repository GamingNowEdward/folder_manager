import { COMMANDS, invokeCommand } from './commands'

export interface ApiSettings {
  /** 用户已保存的端口；从未保存过时为 null。 */
  savedPort: number | null
  /** 本进程启动时请求的端口；与实际监听不同即发生了自动顺延。API 未启动时为 null。 */
  startupPort: number | null
  /** 当前实际监听端口；API 未启动时为 null。 */
  actualPort: number | null
  defaultPort: number
  /** 当前可用的 Base URL；API 未启动时为 null。 */
  url: string | null
  /** 是否由环境变量 FOLDER_MANAGER_API_PORT 覆盖。 */
  envOverride: boolean
  /** 已保存端口与当前实际端口不一致（需重启应用生效）。 */
  restartRequired: boolean
}

export async function getApiSettings(): Promise<ApiSettings> {
  return invokeCommand<ApiSettings>(COMMANDS.getApiSettings)
}

export async function setApiPort(port: number): Promise<ApiSettings> {
  return invokeCommand<ApiSettings>(COMMANDS.setApiPort, { port })
}

export async function getApiDocumentation(): Promise<string> {
  return invokeCommand<string>(COMMANDS.apiDocumentation)
}
