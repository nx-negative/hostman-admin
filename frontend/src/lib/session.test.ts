import { describe, expect, it, beforeEach } from 'vitest'
import { clearSessionMark, hasSessionMark, markSession } from './auth'

function shimSessionStorage() {
  const store = new Map<string, string>()
  Object.defineProperty(globalThis, 'sessionStorage', {
    value: {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => {
        store.set(k, v)
      },
      removeItem: (k: string) => {
        store.delete(k)
      },
    },
    configurable: true,
  })
}

describe('session marker (§7.4 tab-scoped session)', () => {
  beforeEach(() => shimSessionStorage())

  it('is absent by default', () => {
    expect(hasSessionMark()).toBe(false)
  })

  it('mark → present, clear → absent', () => {
    markSession()
    expect(hasSessionMark()).toBe(true)
    clearSessionMark()
    expect(hasSessionMark()).toBe(false)
  })
})