import { createFileRoute } from '@tanstack/react-router'
import { PageStub } from '@/components/page-stub'

export const Route = createFileRoute('/_app/services')({ component: () => <PageStub title="Services" phase="P4" /> })
