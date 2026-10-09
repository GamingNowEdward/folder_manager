import { describe, expect, it } from 'vitest'
import { parsePort } from './port'

describe('parsePort', () => {
  it('accepts numbers (v-model on input type=number)', () => {
    // 回归：编辑端口输入框后 v-model 会给出 number
    expect(parsePort(18000)).toBe(18000)
    expect(parsePort(1)).toBe(1)
    expect(parsePort(65535)).toBe(65535)
  })

  it('accepts numeric strings', () => {
    expect(parsePort('18000')).toBe(18000)
    expect(parsePort('  8080 ')).toBe(8080)
  })

  it('rejects out-of-range, non-integer and empty input', () => {
    expect(parsePort(0)).toBeNull()
    expect(parsePort(-1)).toBeNull()
    expect(parsePort(65536)).toBeNull()
    expect(parsePort(1.5)).toBeNull()
    expect(parsePort('abc')).toBeNull()
    expect(parsePort('')).toBeNull()
    expect(parsePort('   ')).toBeNull()
  })
})
