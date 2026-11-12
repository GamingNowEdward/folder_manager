import type { Folder } from '@/types'
import type { FolderSort } from '@/shared/model/folder-sort'

/**
 * 名称排序使用中文排序规则（ICU）：中文按拼音、英文按字母、数字按数值。
 * `sensitivity: 'base'` 忽略大小写与重音差异，`numeric: true` 让 `RDR2` 排在 `RDR10` 前。
 */
const NAME_COLLATOR = new Intl.Collator('zh-CN', { numeric: true, sensitivity: 'base' })

function compareByName(a: Folder, b: Folder): number {
  const byName = NAME_COLLATOR.compare(a.name, b.name)
  if (byName !== 0) return byName
  const byCreated = a.createdAt - b.createdAt
  if (byCreated !== 0) return byCreated
  return a.id.localeCompare(b.id)
}

function compareByCreated(a: Folder, b: Folder): number {
  const byCreated = a.createdAt - b.createdAt
  if (byCreated !== 0) return byCreated
  const byName = NAME_COLLATOR.compare(a.name, b.name)
  if (byName !== 0) return byName
  return a.id.localeCompare(b.id)
}

/**
 * 计算显示顺序，返回新数组（不修改入参）。
 *
 * - `custom`：保持底层数组顺序，忽略方向。
 * - `name` / `created`：按字段比较；`desc` 对比较结果取反，而不是反转结果数组，
 *   这样同类项之间的兜底顺序保持一致（稳定排序语义）。
 */
export function sortFolders(folders: Folder[], sort: FolderSort): Folder[] {
  const next = [...folders]
  if (sort.field === 'custom') return next

  const compare = sort.field === 'name' ? compareByName : compareByCreated
  const descending = sort.direction === 'desc'
  next.sort((a, b) => (descending ? -compare(a, b) : compare(a, b)))
  return next
}
