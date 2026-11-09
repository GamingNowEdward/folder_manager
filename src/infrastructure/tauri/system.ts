import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { COMMANDS, invokeCommand } from './commands'

export async function openFolderInSystem(path: string): Promise<void> {
  await invokeCommand<void>(COMMANDS.openFolder, { path })
}

export interface DroppedDirectories {
  directories: string[]
  files: number
  invalid: number
}

/**
 * 把拖入的路径交给 Rust 过滤：只保留真实存在的目录。
 * 判定与规范化由 Rust 完成，前端不引入任何平台相关的文件系统判断。
 */
export async function filterDroppedDirectories(paths: string[]): Promise<DroppedDirectories> {
  return invokeCommand<DroppedDirectories>(COMMANDS.filterDirectories, { paths })
}

export async function copyPath(path: string): Promise<void> {
  await writeText(path)
}

export async function copyText(text: string): Promise<void> {
  await writeText(text)
}
