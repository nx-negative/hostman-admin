import { createFileRoute } from '@tanstack/react-router'
import { AdminsPage } from '@/features/auth/admins-page'

export const Route = createFileRoute('/_app/admins')({ component: AdminsPage })
