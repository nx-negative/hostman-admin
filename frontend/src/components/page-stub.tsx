import { Badge } from '@/components/ui/badge'

export function PageStub({ title, phase }: { title: string; phase: string }) {
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <h1 className="text-2xl font-semibold">{title}</h1>
        <Badge variant="outline">{phase}</Badge>
      </div>
      <p className="text-sm text-muted-foreground">Scheduled for {phase}.</p>
    </div>
  )
}
