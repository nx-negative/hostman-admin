import { createFileRoute } from '@tanstack/react-router'
import { PageStub } from '@/components/page-stub'

export const Route = createFileRoute('/_app/audit')({ component: () => <PageStub title="Audit" phase="P6" /> })
