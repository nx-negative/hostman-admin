import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import {
  clearSessionMark,
  fetchMe,
  hasSessionMark,
  logout as logoutApi,
  markSession,
  type Admin,
} from '../lib/auth'

interface AuthState {
  admin: Admin | null
  loading: boolean
  login: (admin: Admin) => void
  logout: () => Promise<void>
  refresh: () => Promise<void>
}

const AuthCtx = createContext<AuthState | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [admin, setAdmin] = useState<Admin | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    const check = async () => {
      if (!hasSessionMark()) {
        // tab was closed/reopened (marker gone) but cookies may persist →
        // revoke the session so the token is removed, then require re-login.
        await logoutApi().catch(() => {})
        setAdmin(null)
        setLoading(false)
        return
      }
      try {
        setAdmin(await fetchMe())
      } catch {
        setAdmin(null)
      } finally {
        setLoading(false)
      }
    }
    check()
  }, [])

  const refresh = async () => {
    try {
      setAdmin(await fetchMe())
    } catch {
      setAdmin(null)
    } finally {
      setLoading(false)
    }
  }

  const login = (a: Admin) => {
    markSession()
    setAdmin(a)
  }

  const logout = async () => {
    clearSessionMark()
    await logoutApi().catch(() => {})
    setAdmin(null)
  }

  return (
    <AuthCtx.Provider value={{ admin, loading, login, logout, refresh }}>
      {children}
    </AuthCtx.Provider>
  )
}

export function useAuth() {
  const ctx = useContext(AuthCtx)
  if (!ctx) throw new Error('useAuth must be used within AuthProvider')
  return ctx
}
