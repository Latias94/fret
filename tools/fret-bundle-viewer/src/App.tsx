import { Suspense, lazy } from 'react'

import { useBundleStore } from '@/store/use-bundle-store'
import { HeaderBar } from '@/components/header-bar'
import { EmptyState } from '@/components/empty-state'
import { useTranslation } from '@/hooks/use-i18n'
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable'
import { Spinner } from '@/components/ui/spinner'

const LazySemanticsTreePanel = lazy(() =>
  import('@/components/semantics-tree-panel').then((m) => ({ default: m.SemanticsTreePanel }))
)
const LazySnapshotsPanel = lazy(() =>
  import('@/components/snapshots-panel').then((m) => ({ default: m.SnapshotsPanel }))
)
const LazyDetailsPanel = lazy(() =>
  import('@/components/details-panel').then((m) => ({ default: m.DetailsPanel }))
)

function PanelFallback({ label }: { label: string }) {
  return (
    <div className="flex h-full items-center justify-center gap-2 text-sm text-muted-foreground">
      <Spinner />
      <span>{label}</span>
    </div>
  )
}

export default function App() {
  const bundle = useBundleStore((s) => s.bundle)
  const parseError = useBundleStore((s) => s.parseError)
  const { t } = useTranslation()

  return (
    <div className="flex h-screen flex-col bg-background">
      <HeaderBar />

      {!bundle && !parseError ? (
        <EmptyState />
      ) : parseError ? (
        <EmptyState />
      ) : (
        <div className="flex-1 overflow-hidden">
          <ResizablePanelGroup direction="horizontal" className="h-full">
            <ResizablePanel defaultSize={30} minSize={20} maxSize={50}>
              <div className="h-full border-r border-border bg-card">
                <Suspense fallback={<PanelFallback label={t('common.loadingTree')} />}>
                  <LazySemanticsTreePanel />
                </Suspense>
              </div>
            </ResizablePanel>

            <ResizableHandle withHandle />

            <ResizablePanel defaultSize={35} minSize={25} maxSize={50}>
              <div className="h-full border-r border-border bg-card">
                <Suspense fallback={<PanelFallback label={t('common.loadingSnapshots')} />}>
                  <LazySnapshotsPanel />
                </Suspense>
              </div>
            </ResizablePanel>

            <ResizableHandle withHandle />

            <ResizablePanel defaultSize={35} minSize={25}>
              <div className="h-full bg-card">
                <Suspense fallback={<PanelFallback label={t('common.loadingDetails')} />}>
                  <LazyDetailsPanel />
                </Suspense>
              </div>
            </ResizablePanel>
          </ResizablePanelGroup>
        </div>
      )}

      <footer className="flex items-center justify-center border-t border-border bg-muted/30 px-4 py-2">
        <p className="text-xs text-muted-foreground">{t('app.offline')}</p>
      </footer>
    </div>
  )
}
