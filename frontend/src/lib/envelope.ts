export type Envelope<T> =
  | { ok: true; data: T }
  | { ok: false; error: { code: string; msg: string; details: unknown } }

export class ApiError extends Error {
  constructor(
    readonly code: string,
    msg: string,
    readonly status: number,
    readonly details: unknown = null,
  ) {
    super(msg)
    this.name = 'ApiError'
  }
}

export function parseEnvelope<T>(body: unknown, status: number): T {
  const e = body as Envelope<T> | null
  if (e && typeof e === 'object' && 'ok' in e) {
    if (e.ok) return e.data
    throw new ApiError(e.error.code, e.error.msg, status, e.error.details)
  }
  throw new ApiError('INTERNAL', 'malformed response envelope', status)
}
