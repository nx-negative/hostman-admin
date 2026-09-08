import { createFileRoute } from '@tanstack/react-router'
import { PageStub } from '@/components/page-stub'

export const Route = createFileRoute('/_app/notifications')({ component: () => <PageStub title="Notifications" phase="P6" /> })
