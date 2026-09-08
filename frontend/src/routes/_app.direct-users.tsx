import { createFileRoute } from '@tanstack/react-router'
import { PageStub } from '@/components/page-stub'

export const Route = createFileRoute('/_app/direct-users')({ component: () => <PageStub title="Direct Users" phase="P3" /> })
