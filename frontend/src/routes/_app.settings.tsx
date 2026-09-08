import { createFileRoute } from '@tanstack/react-router'
import { PageStub } from '@/components/page-stub'

export const Route = createFileRoute('/_app/settings')({ component: () => <PageStub title="Settings" phase="P8" /> })
