export type FolderViewMode = 'grid' | 'list'

export const DEFAULT_FOLDER_VIEW_MODE: FolderViewMode = 'grid'

/**
 * 解析持久化的视图偏好。
 * 任何非法 / 缺失 / 脏数据都回落默认值，避免 UI 进入空白布局。
 */
export function parseViewMode(raw: unknown): FolderViewMode {
  return typeof raw === 'string' && raw.toLowerCase() === 'list' ? 'list' : 'grid'
}
