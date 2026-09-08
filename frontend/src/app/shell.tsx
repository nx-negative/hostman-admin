import { Link, Outlet } from '@tanstack/react-router'
import { motion } from 'framer-motion'
import { LogOut, Moon, ShieldCheck, Sun } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useTheme } from '@/app/theme'
import { useAuth } from '@/app/auth'
import { nav } from '@/lib/nav'

export function Shell() {
  const { theme, toggle } = useTheme()
  const { admin, logout } = useAuth()
  const isMother = admin?.role === 'mother_admin'

  return (
    <div className="flex h-svh bg-background text-foreground">
      <aside className="flex w-60 shrink-0 flex-col border-r">
        <div className="flex h-14 items-center gap-2 border-b px-4 font-semibold">
          <ShieldCheck className="size-5" /> Hostman Admin
        </div>
        <nav className="flex-1 space-y-1 overflow-y-auto p-2">
          {nav
            .filter((i) => i.to !== '/admins' || isMother)
            .map((i) => (
              <Link
                key={i.to}
                to={i.to}
                className="flex items-center gap-3 rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-accent/50"
                activeOptions={{ exact: i.to === '/' }}
              >
                <i.icon className="size-4" /> {i.label}
              </Link>
            ))}
        </nav>
        <div className="border-t p-3 text-xs text-muted-foreground">v0.1.0 · P1</div>
      </aside>
      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-14 items-center justify-between border-b px-6">
          <div className="flex items-center gap-3 text-sm">
            <span className="text-muted-foreground">Admin control plane</span>
            {admin && (
              <>
                <span className="text-border">·</span>
                <span className="font-medium">{admin.code_prefix}</span>
                <span className="rounded bg-muted px-1.5 py-0.5 text-xs">{admin.role}</span>
              </>
            )}
          </div>
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="icon" onClick={toggle} aria-label="Toggle theme">
              {theme === 'dark' ? <Sun className="size-4" /> : <Moon className="size-4" />}
            </Button>
            <Button variant="ghost" size="icon" onClick={() => logout()} aria-label="Log out">
              <LogOut className="size-4" />
            </Button>
          </div>
        </header>
        <motion.main
          initial={{ opacity: 0, y: 4 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.15 }}
          className="flex-1 overflow-auto p-6"
        >
          <Outlet />
        </motion.main>
      </div>
    </div>
  )
}
