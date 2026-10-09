export const MIN_PORT = 1
export const MAX_PORT = 65535

/**
 * 把端口输入解析为合法端口号；非法返回 `null`。
 *
 * 必须同时接受 `number` 与 `string`：`<input type="number">` 的 `v-model`
 * 会在用户编辑后把值转成 number（见 `port.spec.ts` 的回归用例），
 * 若只按字符串处理（如 `raw.trim()`）会抛异常。
 */
export function parsePort(raw: string | number): number | null {
  const port = typeof raw === 'number' ? raw : Number(raw.trim())
  if (!Number.isInteger(port) || port < MIN_PORT || port > MAX_PORT) return null
  return port
}
