import { folderNameFromPath } from '@/shared/utils/path'

export interface ExternalFolder {
  name: string
  path: string
}

export interface ExternalDropPlan {
  accepted: ExternalFolder[]
  skipped: number
}

export function planExternalDrop(
  paths: string[],
  existingNames: Iterable<string>
): ExternalDropPlan {
  const accepted: ExternalFolder[] = []
  const known = new Set(existingNames)
  let skipped = 0

  for (const path of paths) {
    const name = folderNameFromPath(path)
    if (!name || known.has(name)) {
      if (name) skipped += 1
      continue
    }
    known.add(name)
    accepted.push({ name, path })
  }

  return { accepted, skipped }
}

export interface ExternalDropSummary {
  added: number
  /** 与当前项目内已有文件夹重名而跳过的数量。 */
  duplicated: number
  /** 拖入项中明确是文件的数量。 */
  files: number
  /** 无法解析的路径数量：不存在、无权限、相对路径等。 */
  invalid: number
}

/** 把一次拖入的结果汇总成状态栏文案；没有任何动静时返回空串。 */
export function describeExternalDrop(summary: ExternalDropSummary): string {
  const { added, duplicated, files, invalid } = summary

  if (added === 0 && duplicated === 0 && files === 0 && invalid === 0) return ''

  const details: string[] = []
  if (files > 0) details.push(`忽略 ${files} 个文件`)
  if (invalid > 0) details.push(`${invalid} 项无法访问`)

  if (added > 0) {
    const head = [`▶ 已添加 ${added} 个文件夹`]
    if (duplicated > 0) head.push(`${duplicated} 个因重名跳过`)
    return [...head, ...details].join('，')
  }

  if (duplicated > 0) {
    return [`✕ ${duplicated} 个文件夹因重名被跳过`, ...details].join('，')
  }

  if (files > 0 && invalid > 0) {
    return `✕ 已忽略 ${files} 个文件，${invalid} 项无法访问`
  }
  if (files > 0) return `✕ 已忽略 ${files} 个文件（只支持文件夹）`
  return `✕ ${invalid} 项无法访问（不存在或没有权限）`
}
