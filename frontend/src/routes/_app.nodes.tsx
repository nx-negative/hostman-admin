import { createFileRoute } from '@tanstack/react-router'
import { PageStub } from '@/components/page-stub'

export const Route = createFileRoute('/_app/nodes')({ component: () => <PageStub title="Nodes" phase="P2" /> })
