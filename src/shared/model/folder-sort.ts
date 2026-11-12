/**
 * 文件夹排序偏好：属于 UI 偏好（localStorage），不写进 config.json。
 *
 * - `custom`：用户拖拽决定的顺序，方向由拖拽决定，不使用 `direction`。
 * - `name`：按显示名（备注名）排序，中文按拼音、英文按字母、数字按数值。
 * - `created`：按添加时间（`Folder.createdAt`）排序。
 */
export type FolderSortField = 'custom' | 'name' | 'created'
export type FolderSortDirection = 'asc' | 'desc'

export interface FolderSort {
  field: FolderSortField
  direction: FolderSortDirection
}

export const DEFAULT_FOLDER_SORT: FolderSort = { field: 'custom', direction: 'asc' }

const SORT_FIELDS: FolderSortField[] = ['custom', 'name', 'created']
const SORT_DIRECTIONS: FolderSortDirection[] = ['asc', 'desc']

function isFolderSortField(value: unknown): value is FolderSortField {
  return typeof value === 'string' && (SORT_FIELDS as string[]).includes(value)
}

function isFolderSortDirection(value: unknown): value is FolderSortDirection {
  return typeof value === 'string' && (SORT_DIRECTIONS as string[]).includes(value)
}

/**
 * 解析持久化的排序偏好。
 * 任何非法 / 缺失 / 脏数据都回落默认值，避免 UI 进入不可预期的顺序。
 */
export function parseFolderSort(raw: unknown): FolderSort {
  let value: unknown = raw
  if (typeof raw === 'string') {
    try {
      value = JSON.parse(raw)
    } catch {
      return { ...DEFAULT_FOLDER_SORT }
    }
  }
  if (typeof value !== 'object' || value === null) return { ...DEFAULT_FOLDER_SORT }

  const record = value as Record<string, unknown>
  return {
    field: isFolderSortField(record.field) ? record.field : DEFAULT_FOLDER_SORT.field,
    direction: isFolderSortDirection(record.direction)
      ? record.direction
      : DEFAULT_FOLDER_SORT.direction
  }
}

export function serializeFolderSort(sort: FolderSort): string {
  return JSON.stringify(sort)
}
