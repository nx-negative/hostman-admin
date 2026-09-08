import { createFileRoute } from '@tanstack/react-router'
import { Shell } from '@/app/shell'

export const Route = createFileRoute('/_app')({ component: Shell })
