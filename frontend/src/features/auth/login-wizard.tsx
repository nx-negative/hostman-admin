import { useState } from 'react'
import { useForm } from '@tanstack/react-form'
import { z } from 'zod'
import { toast } from '@/components/ui/toast'
import { login, markSession, verifyTotp, type Admin, type LoginResponse } from '@/lib/auth'
import { ApiError } from '@/lib/envelope'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

type Step = 'credentials' | 'totp' | 'recovery'

const credentialsSchema = z.object({
  loginCode: z.string().min(1, 'Login code is required'),
  password: z.string().min(1, 'Password is required'),
})
const totpSchema = z.string().length(6, 'Enter the 6-digit code from your authenticator')

function formError(err: unknown): string {
  if (err instanceof Error) return err.message
  return 'Something went wrong, please try again'
}

/** Single-session (§7.4) conflict vs. general login/TOTP failure. */
function describeError(err: unknown): { title?: string; description: string } {
  if (err instanceof ApiError && err.code === 'CONFLICT') {
    return {
      title: 'Session already active',
      description: err.message,
    }
  }
  return { title: undefined, description: formError(err) }
}

export function LoginWizard({ onLoggedIn }: { onLoggedIn: (admin: Admin) => void }) {
  const [step, setStep] = useState<Step>('credentials')
  const [resp, setResp] = useState<LoginResponse | null>(null)
  const [qr, setQr] = useState('')
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([])
  const [admin, setAdmin] = useState<Admin | null>(null)
  const [error, setError] = useState('')
  const [busy, setBusy] = useState(false)

  const credentialsForm = useForm({
    defaultValues: { loginCode: '', password: '' },
    onSubmit: async ({ value }) => {
      setError('')
      const parsed = credentialsSchema.safeParse(value)
      if (!parsed.success) {
        const msg = parsed.error.issues[0].message
        setError(msg)
        toast.add({ title: 'Invalid input', description: msg, type: 'error' })
        return
      }
      setBusy(true)
      try {
        const r = await login(parsed.data.loginCode, parsed.data.password)
        markSession()
        setResp(r)
        if (r.otpauth_url) {
          const QRCode = (await import('qrcode')).default
          setQr(await QRCode.toDataURL(r.otpauth_url))
        }
        setStep('totp')
      } catch (err) {
        const { title, description } = describeError(err)
        setError(description)
        toast.add({ title: title ?? 'Login failed', description })
      } finally {
        setBusy(false)
      }
    },
  })

  const totpForm = useForm({
    defaultValues: { totp: '' },
    onSubmit: async ({ value }) => {
      setError('')
      const msg = totpSchema.safeParse(value.totp).error?.issues[0]?.message
      if (msg) {
        setError(msg)
        toast.add({ title: 'Invalid code', description: msg, type: 'error' })
        return
      }
      if (!resp) return
      setBusy(true)
      try {
        const r = await verifyTotp(resp.temp_token, value.totp)
        setAdmin(r.admin)
        if (r.recovery_codes) {
          setRecoveryCodes(r.recovery_codes)
          setStep('recovery')
          toast.add({ title: 'TOTP enabled', description: 'Save your recovery codes.', type: 'success' })
        } else {
          toast.add({ title: 'Welcome back', description: 'You are signed in.', type: 'success' })
          onLoggedIn(r.admin)
        }
      } catch (err) {
        const { title, description } = describeError(err)
        setError(description)
        toast.add({ title: title ?? 'Verification failed', description })
      } finally {
        setBusy(false)
      }
    },
  })

  if (step === 'recovery') {
    return (
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Save your recovery codes</CardTitle>
          <CardDescription>Shown once. Each works one time if you lose your authenticator.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-2 rounded-md bg-muted p-3 font-mono text-sm">
            {recoveryCodes.map((c) => (
              <span key={c}>{c}</span>
            ))}
          </div>
          <Button className="w-full" onClick={() => admin && onLoggedIn(admin)}>
            I've saved them — continue
          </Button>
        </CardContent>
      </Card>
    )
  }

    if (step === 'totp') {
    const isFirst = !!resp?.otpauth_url
    return (
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>{isFirst ? 'Scan the QR code' : 'Enter your TOTP code'}</CardTitle>
          <CardDescription>
            {isFirst
              ? 'Scan with your authenticator app, then enter the 6-digit code below.'
              : 'Enter the 6-digit code from your authenticator app.'}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {qr && <img src={qr} alt="TOTP QR" className="mx-auto rounded-md border" />}
          <form onSubmit={(e) => { e.preventDefault(); totpForm.handleSubmit() }} className="space-y-4">
            <totpForm.Field name="totp">
              {(field) => (
                <>
                  <Label htmlFor="totp" className="text-sm font-medium">6-digit code</Label>
                  <Input
                    id="totp"
                    inputMode="numeric"
                    pattern="[0-9]{6}"
                    autoFocus
                    className="h-10 text-base"
                    value={field.state.value}
                    onChange={(e) => field.handleChange(e.target.value)}
                  />
                </>
              )}
            </totpForm.Field>
            {error && <p className="text-sm text-destructive">{error}</p>}
            <Button type="submit" className="w-full h-10 text-base" disabled={busy}>
              {busy ? 'Verifying…' : 'Verify'}
            </Button>
          </form>
        </CardContent>
      </Card>
    )
  }

    return (
    <Card className="w-full max-w-sm">
      <CardHeader>
        <CardTitle>Sign in to Hostman Admin</CardTitle>
        <CardDescription>
          Login code and password are crypto-verified. Enter your credentials to continue.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <form onSubmit={(e) => { e.preventDefault(); credentialsForm.handleSubmit() }} className="space-y-4">
          <credentialsForm.Field name="loginCode">
            {(field) => (
              <>
                <Label htmlFor="code" className="text-sm font-medium">Login code</Label>
                <Input
                  id="code"
                  placeholder="32-character code"
                  autoFocus
                  className="h-8 text-base"
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                />
              </>
            )}
          </credentialsForm.Field>
          <credentialsForm.Field name="password">
            {(field) => (
              <>
                <Label htmlFor="pw" className="text-sm font-medium">Password</Label>
                <Input
                  id="pw"
                  type="password"
                  className="h-8 text-base"
                  value={field.state.value}
                  onChange={(e) => field.handleChange(e.target.value)}
                />
              </>
            )}
          </credentialsForm.Field>
          {error && <p className="text-sm text-destructive">{error}</p>}
          <Button type="submit" className="w-full h-8 text-sm" disabled={busy}>
            {busy ? 'Signing in…' : 'Continue'}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}
