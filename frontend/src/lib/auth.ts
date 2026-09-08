import { api } from './api'

export interface Admin {
  id: string
  role: 'mother_admin' | 'admin'
  code_prefix: string
  totp_enabled: boolean
}

export interface LoginResponse {
  state: 'totp_enroll' | 'totp_required'
  temp_token: string
  otpauth_url?: string
}

export interface VerifyResponse {
  admin: Admin
  recovery_codes: string[] | null
}

const SESSION_MARK = 'hm_session'

/** Per-tab marker: survives reload, dies with the tab/browser (§7.4 session-scoped). */
export const markSession = () => sessionStorage.setItem(SESSION_MARK, '1')
export const clearSessionMark = () => sessionStorage.removeItem(SESSION_MARK)
export const hasSessionMark = () => sessionStorage.getItem(SESSION_MARK) === '1'

export async function login(loginCode: string, password: string): Promise<LoginResponse> {
  return api('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ login_code: loginCode, password }),
  })
}

export async function verifyTotp(tempToken: string, code: string): Promise<VerifyResponse> {
  return api('/auth/totp/verify', {
    method: 'POST',
    body: JSON.stringify({ temp_token: tempToken, code }),
  })
}

export async function fetchMe(): Promise<Admin> {
  const res = await api<{ admin: Admin }>('/auth/me')
  return res.admin
}

export async function logout(): Promise<void> {
  await api('/auth/logout', { method: 'POST' })
}

export async function listAdmins(): Promise<Admin[]> {
  const res = await api<{ admins: Admin[] }>('/admins')
  return res.admins
}

export interface CreatedAdmin {
  id: string
  login_code: string
  password: string
}

export async function createAdmin(role: string, email?: string): Promise<CreatedAdmin> {
  return api('/admins', {
    method: 'POST',
    body: JSON.stringify({ role, email }),
  })
}

export async function deleteAdmin(id: string): Promise<void> {
  await api(`/admins/${id}`, { method: 'DELETE' })
}
