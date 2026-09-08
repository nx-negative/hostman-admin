import { Activity, Bell, Radio, ScrollText, Server, Settings, ShieldCheck, Users, UsersRound } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

export interface NavItem {
  to: string
  label: string
  icon: LucideIcon
}

export const nav: NavItem[] = [
  { to: '/', label: 'Dashboard', icon: Activity },
  { to: '/nodes', label: 'Nodes', icon: Server },
  { to: '/resellers', label: 'Resellers', icon: Users },
  { to: '/direct-users', label: 'Direct Users', icon: Radio },
  { to: '/services', label: 'Services', icon: ShieldCheck },
  { to: '/notifications', label: 'Notifications', icon: Bell },
  { to: '/audit', label: 'Audit', icon: ScrollText },
  { to: '/admins', label: 'Admins', icon: UsersRound },
  { to: '/settings', label: 'Settings', icon: Settings },
]
