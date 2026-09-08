import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import { fetchMe, logout as logoutApi, type Admin } from '../lib/auth'

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

  const refresh = async () => {
    try {
      setAdmin(await fetchMe())
    } catch {
      setAdmin(null)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    refresh()
  }, [])

  const login = (a: Admin) => setAdmin(a)

  const logout = async () => {
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
