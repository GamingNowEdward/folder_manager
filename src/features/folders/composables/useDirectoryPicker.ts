import { pickDirectory as pickDirectoryFromSystem } from '@/infrastructure/tauri/dialog'

export function useDirectoryPicker() {
  function pickDirectory(): Promise<string | null> {
    return pickDirectoryFromSystem()
  }

  return { pickDirectory }
}
