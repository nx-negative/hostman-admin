import { createFileRoute } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { api } from '@/lib/api'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

export const Route = createFileRoute('/_app/')({ component: Dashboard })

type Ready = { status: string; db: string }
type DbState = 'ok' | 'down' | 'pending'

function Dashboard() {
  const q = useQuery({
    queryKey: ['readyz'],
    queryFn: () => api<Ready>('/readyz'),
    refetchInterval: 15_000,
  })

  const state: DbState = q.isPending ? 'pending' : q.data?.db === 'ok' ? 'ok' : 'down'

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">Dashboard</h1>
      <Card className="max-w-sm">
        <CardHeader>
          <CardTitle className="text-base">System</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center justify-between text-sm">
            <span>Cloud database</span>
            <ReadyBadge state={state} />
          </div>
          <div className="flex items-center justify-between text-sm text-muted-foreground">
            <span>Phase</span>
            <span>P0 scaffold</span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

const dot = { ok: 'bg-emerald-500', down: 'bg-red-500', pending: 'bg-amber-400' } as const
const label = { ok: 'db ok', down: 'db down', pending: 'checking…' } as const

function ReadyBadge({ state }: { state: DbState }) {
  return (
    <Badge variant="outline" className="gap-1.5">
      <span className={`size-2 rounded-full ${dot[state]}`} /> {label[state]}
    </Badge>
  )
}
