import { useState, type FormEvent } from 'react'
import QRCode from 'qrcode'
import { login, verifyTotp, type Admin, type LoginResponse } from '@/lib/auth'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

type Step = 'credentials' | 'totp' | 'recovery'

export function LoginWizard({ onLoggedIn }: { onLoggedIn: (admin: Admin) => void }) {
  const [step, setStep] = useState<Step>('credentials')
  const [loginCode, setLoginCode] = useState('')
  const [password, setPassword] = useState('')
  const [totp, setTotp] = useState('')
  const [resp, setResp] = useState<LoginResponse | null>(null)
  const [qr, setQr] = useState('')
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([])
  const [admin, setAdmin] = useState<Admin | null>(null)
  const [error, setError] = useState('')
  const [busy, setBusy] = useState(false)

  async function submitCredentials(e: FormEvent) {
    e.preventDefault()
    setError('')
    setBusy(true)
    try {
      const r = await login(loginCode, password)
      setResp(r)
      if (r.otpauth_url) {
        setQr(await QRCode.toDataURL(r.otpauth_url))
      }
      setStep('totp')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Login failed')
    } finally {
      setBusy(false)
    }
  }

  async function submitTotp(e: FormEvent) {
    e.preventDefault()
    if (!resp) return
    setError('')
    setBusy(true)
    try {
      const r = await verifyTotp(resp.temp_token, totp)
      setAdmin(r.admin)
      if (r.recovery_codes) {
        setRecoveryCodes(r.recovery_codes)
        setStep('recovery')
      } else {
        onLoggedIn(r.admin)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Invalid code')
    } finally {
      setBusy(false)
    }
  }

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
              ? 'Scan with any authenticator app, then enter the code.'
              : 'Enter the 6-digit code from your authenticator.'}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {qr && <img src={qr} alt="TOTP QR" className="mx-auto rounded-md border" />}
          <form onSubmit={submitTotp} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="totp">6-digit code</Label>
              <Input
                id="totp"
                inputMode="numeric"
                pattern="[0-9]{6}"
                value={totp}
                onChange={(e) => setTotp(e.target.value)}
                placeholder="000000"
                autoFocus
              />
            </div>
            {error && <p className="text-sm text-destructive">{error}</p>}
            <Button type="submit" className="w-full" disabled={busy || totp.length !== 6}>
              Verify
            </Button>
          </form>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="w-full max-w-sm">
      <CardHeader>
        <CardTitle>Sign in</CardTitle>
        <CardDescription>Login code + password.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <form onSubmit={submitCredentials} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="code">Login code</Label>
            <Input
              id="code"
              value={loginCode}
              onChange={(e) => setLoginCode(e.target.value)}
              placeholder="32-character code"
              autoFocus
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="pw">Password</Label>
            <Input id="pw" type="password" value={password} onChange={(e) => setPassword(e.target.value)} />
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
          <Button type="submit" className="w-full" disabled={busy || !loginCode || !password}>
            Continue
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}
