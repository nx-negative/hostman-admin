import { ApiError, parseEnvelope } from './envelope'

const BASE = import.meta.env.VITE_API_URL ?? '/api/v1'

/** Fired when a request is rejected as unauthorized (e.g. session revoked by a newer login). */
export const UNAUTHORIZED_EVENT = 'hm-unauthorized'

export function dispatchUnauthorized() {
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent(UNAUTHORIZED_EVENT))
  }
}

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
  if (res.status === 401) dispatchUnauthorized()
  const body = await res.json().catch(() => null)
  return parseEnvelope<T>(body, res.status)
}
