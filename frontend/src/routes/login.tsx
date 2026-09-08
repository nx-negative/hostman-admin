import { createFileRoute } from '@tanstack/react-router'
import { LoginWizard } from '@/features/auth/login-wizard'
import { useAuth } from '@/app/auth'
import type { Admin } from '@/lib/auth'

export const Route = createFileRoute('/login')({ component: LoginPage })

function LoginPage() {
  const { admin, login } = useAuth()
  if (admin) {
    if (typeof window !== 'undefined') window.location.href = '/'
    return null
  }

  const onLoggedIn = (a: Admin) => {
    login(a)
    if (typeof window !== 'undefined') window.location.href = '/'
  }

  return (
    <div className="flex min-h-svh items-center justify-center bg-background p-4">
      <LoginWizard onLoggedIn={onLoggedIn} />
    </div>
  )
}
