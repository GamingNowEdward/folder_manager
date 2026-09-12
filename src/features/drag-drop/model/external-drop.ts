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
