import { createFileRoute } from '@tanstack/react-router'
import { LoginWizard } from '@/features/auth/login-wizard'
import { useAuth } from '@/app/auth'

export const Route = createFileRoute('/login')({ component: LoginPage })

function LoginPage() {
  const { admin, login } = useAuth()
  if (admin) {
    if (typeof window !== 'undefined') window.location.href = '/'
    return null
  }
  return (
    <div className="flex min-h-svh items-center justify-center bg-background p-4">
      <LoginWizard
        onLoggedIn={(isFirst) => {
          if (isFirst) {
            login({ id: '', role: 'mother_admin', code_prefix: '', totp_enabled: true })
          }
          window.location.href = '/'
        }}
      />
    </div>
  )
}
