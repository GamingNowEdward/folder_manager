import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { COMMANDS, invokeCommand } from './commands'

export async function openFolderInSystem(path: string): Promise<void> {
  await invokeCommand<void>(COMMANDS.openFolder, { path })
}

export async function copyPath(path: string): Promise<void> {
  await writeText(path)
}
