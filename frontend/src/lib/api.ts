import { ApiError, parseEnvelope } from './envelope'

const BASE = import.meta.env.VITE_API_URL ?? '/api/v1'

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  let res: Response
  try {
    res = await fetch(BASE + path, {
      ...init,
      headers: { 'content-type': 'application/json', ...init?.headers },
    })
  } catch {
    throw new ApiError('UPSTREAM', 'network error', 0)
  }
  const body = await res.json().catch(() => null)
  return parseEnvelope<T>(body, res.status)
}
