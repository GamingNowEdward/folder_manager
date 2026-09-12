export function folderNameFromPath(path: string): string {
  return (
    path
      .replace(/[/\\]+$/, '')
      .split(/[/\\]/)
      .pop() ?? ''
  )
}
