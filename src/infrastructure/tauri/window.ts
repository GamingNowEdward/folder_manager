import { getCurrentWindow } from '@tauri-apps/api/window'

const appWindow = getCurrentWindow()

export function minimizeWindow(): Promise<void> {
  return appWindow.minimize()
}

export function toggleMaximizeWindow(): Promise<void> {
  return appWindow.toggleMaximize()
}

export function closeWindow(): Promise<void> {
  return appWindow.close()
}

export function startWindowDrag(): Promise<void> {
  return appWindow.startDragging()
}

export async function onWindowFileDrop(handler: (paths: string[]) => void): Promise<() => void> {
  return appWindow.onDragDropEvent((event) => {
    if (event.payload.type === 'drop' && event.payload.paths.length > 0) {
      handler(event.payload.paths)
    }
  })
}
