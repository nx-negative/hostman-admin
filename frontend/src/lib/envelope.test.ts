import { describe, expect, it } from 'vitest'
import { ApiError, parseEnvelope } from './envelope'

describe('parseEnvelope', () => {
  it('returns data on ok envelope', () => {
    expect(parseEnvelope<{ a: number }>({ ok: true, data: { a: 1 } }, 200)).toEqual({ a: 1 })
  })

  it('throws ApiError with code/status on error envelope', () => {
    const body = { ok: false, error: { code: 'VALIDATION', msg: 'bad input', details: {} } }
    expect(() => parseEnvelope(body, 400)).toThrowError(ApiError)
    try {
      parseEnvelope(body, 400)
    } catch (e) {
      const err = e as ApiError
      expect(err.code).toBe('VALIDATION')
      expect(err.status).toBe(400)
      expect(err.message).toBe('bad input')
    }
  })

  it('throws on malformed body', () => {
    expect(() => parseEnvelope(null, 200)).toThrowError(ApiError)
    expect(() => parseEnvelope({ unexpected: true }, 200)).toThrowError(ApiError)
  })
})
