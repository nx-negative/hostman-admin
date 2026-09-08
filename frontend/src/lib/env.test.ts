import { describe, expect, it } from 'vitest'
import { envSchema } from './env'

describe('envSchema', () => {
  it('accepts empty env', () => {
    expect(envSchema.parse({})).toEqual({})
  })

  it('accepts valid url', () => {
    expect(envSchema.parse({ VITE_API_URL: 'https://api.example.com/v1' }).VITE_API_URL).toBe(
      'https://api.example.com/v1',
    )
  })

  it('rejects invalid url', () => {
    expect(() => envSchema.parse({ VITE_API_URL: 'not-a-url' })).toThrowError()
  })
})
