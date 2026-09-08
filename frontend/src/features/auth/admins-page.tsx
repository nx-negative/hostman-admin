import { useState, type FormEvent } from 'react'
import { useAuth } from '@/app/auth'
import {
  createAdmin,
  deleteAdmin,
  listAdmins,
  type Admin,
  type CreatedAdmin,
} from '@/lib/auth'
import { Button } from '@/components/ui/button'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'

export function AdminsPage() {
  const { admin } = useAuth()
  const [admins, setAdmins] = useState<Admin[]>([])
  const [loading, setLoading] = useState(false)
  const [role, setRole] = useState('admin')
  const [created, setCreated] = useState<CreatedAdmin | null>(null)
  const [open, setOpen] = useState(false)
  const [error, setError] = useState('')

  async function load() {
    setLoading(true)
    try {
      setAdmins(await listAdmins())
    } finally {
      setLoading(false)
    }
  }

  async function submit(e: FormEvent) {
    e.preventDefault()
    setError('')
    try {
      const a = await createAdmin(role)
      setCreated(a)
      await load()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed')
    }
  }

  async function remove(id: string) {
    await deleteAdmin(id)
    await load()
  }

  if (admin?.role !== 'mother_admin') {
    return (
      <div className="space-y-4">
        <h1 className="text-2xl font-semibold">Admins</h1>
        <p className="text-sm text-muted-foreground">Only mother_admin can manage admins.</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Admins</h1>
        <Dialog open={open} onOpenChange={setOpen}>
          <DialogTrigger
            className="inline-flex h-10 items-center justify-center gap-2 whitespace-nowrap rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground"
            onClick={() => { setCreated(null); setRole('admin') }}
          >
            Add admin
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>{created ? 'Admin created' : 'Add admin'}</DialogTitle>
              <DialogDescription>
                {created
                  ? 'Copy these credentials — shown once.'
                  : 'A login code and one-time password will be generated.'}
              </DialogDescription>
            </DialogHeader>
            {created ? (
              <div className="space-y-3">
                <div className="space-y-1">
                  <Label>Login code</Label>
                  <div className="rounded-md bg-muted p-2 font-mono text-sm">{created.login_code}</div>
                </div>
                <div className="space-y-1">
                  <Label>One-time password</Label>
                  <div className="rounded-md bg-muted p-2 font-mono text-sm">{created.password}</div>
                </div>
                <Button className="w-full" onClick={() => setOpen(false)}>
                  Done
                </Button>
              </div>
            ) : (
              <form onSubmit={submit} className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="role">Role</Label>
                  <select
                    id="role"
                    className="flex h-10 w-full rounded-md border border-input bg-background px-3"
                    value={role}
                    onChange={(e) => setRole(e.target.value)}
                  >
                    <option value="admin">admin</option>
                    <option value="mother_admin">mother_admin</option>
                  </select>
                </div>
                {error && <p className="text-sm text-destructive">{error}</p>}
                <Button type="submit" className="w-full">
                  Create
                </Button>
              </form>
            )}
          </DialogContent>
        </Dialog>
      </div>

      <Button variant="outline" onClick={load} disabled={loading}>
        {loading ? 'Loading…' : 'Refresh'}
      </Button>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Prefix</TableHead>
            <TableHead>Role</TableHead>
            <TableHead>TOTP</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {admins.map((a) => (
            <TableRow key={a.id}>
              <TableCell className="font-mono">{a.code_prefix}</TableCell>
              <TableCell>
                <Badge variant={a.role === 'mother_admin' ? 'default' : 'secondary'}>{a.role}</Badge>
              </TableCell>
              <TableCell>{a.totp_enabled ? '✓' : '—'}</TableCell>
              <TableCell className="text-right">
                {a.id !== admin.id && a.role !== 'mother_admin' && (
                  <Button variant="destructive" size="sm" onClick={() => remove(a.id)}>
                    Remove
                  </Button>
                )}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}
