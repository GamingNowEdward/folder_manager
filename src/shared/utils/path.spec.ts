import { describe, expect, it } from 'vitest'
import { folderNameFromPath } from './path'

describe('folderNameFromPath', () => {
  it('returns the last segment for windows paths', () => {
    expect(folderNameFromPath('C:\\Work\\Alpha')).toBe('Alpha')
    expect(folderNameFromPath('C:\\Work\\Alpha\\')).toBe('Alpha')
  })

  it('handles forward slashes', () => {
    expect(folderNameFromPath('/home/user/Alpha/')).toBe('Alpha')
  })

  it('returns empty string for empty or separator-only input', () => {
    expect(folderNameFromPath('')).toBe('')
    expect(folderNameFromPath('///')).toBe('')
  })
})
