import { Link, Outlet } from '@tanstack/react-router'
import { motion } from 'framer-motion'
import { Moon, ShieldCheck, Sun } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useTheme } from '@/app/theme'
import { nav } from '@/lib/nav'

export function Shell() {
  const { theme, toggle } = useTheme()
  return (
    <div className="flex h-svh bg-background text-foreground">
      <aside className="flex w-60 shrink-0 flex-col border-r">
        <div className="flex h-14 items-center gap-2 border-b px-4 font-semibold">
          <ShieldCheck className="size-5" /> Hostman Admin
        </div>
        <nav className="flex-1 space-y-1 overflow-y-auto p-2">
          {nav.map((i) => (
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
        <div className="border-t p-3 text-xs text-muted-foreground">v0.1.0 · P0</div>
      </aside>
      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-14 items-center justify-between border-b px-6">
          <div className="text-sm text-muted-foreground">Admin control plane</div>
          <Button variant="ghost" size="icon" onClick={toggle} aria-label="Toggle theme">
            {theme === 'dark' ? <Sun className="size-4" /> : <Moon className="size-4" />}
          </Button>
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
