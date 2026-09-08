import { createFileRoute } from '@tanstack/react-router'
import { PageStub } from '@/components/page-stub'

export const Route = createFileRoute('/_app/resellers')({ component: () => <PageStub title="Resellers" phase="P3" /> })
