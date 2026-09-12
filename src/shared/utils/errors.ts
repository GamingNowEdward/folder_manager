export function toErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  if (error === null || error === undefined) return '未知错误'
  return String(error)
}
