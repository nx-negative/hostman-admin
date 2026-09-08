import { createFileRoute } from '@tanstack/react-router'
import { Shell } from '@/app/shell'
import { useAuth } from '@/app/auth'

export const Route = createFileRoute('/_app')({ component: AppLayout })

function AppLayout() {
  const { admin, loading } = useAuth()
  if (loading) {
    return (
      <div className="flex h-svh items-center justify-center bg-background">
        <div className="text-sm text-muted-foreground">Loading…</div>
      </div>
    )
  }
  if (!admin) {
    if (typeof window !== 'undefined' && window.location.pathname !== '/login') {
      window.location.href = '/login'
    }
    return null
  }
  return <Shell />
}
