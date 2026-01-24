import { useBundleStore } from '@/store/use-bundle-store'
import { DetailsPanel } from '@/components/details-panel'
import { EmptyState } from '@/components/empty-state'
import { HeaderBar } from '@/components/header-bar'
import { SemanticsTreePanel } from '@/components/semantics-tree-panel'
import { SnapshotsPanel } from '@/components/snapshots-panel'
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable'

export default function App() {
  const bundle = useBundleStore((s) => s.bundle)
  const parseError = useBundleStore((s) => s.parseError)

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
                <SemanticsTreePanel />
              </div>
            </ResizablePanel>

            <ResizableHandle withHandle />

            <ResizablePanel defaultSize={35} minSize={25} maxSize={50}>
              <div className="h-full border-r border-border bg-card">
                <SnapshotsPanel />
              </div>
            </ResizablePanel>

            <ResizableHandle withHandle />

            <ResizablePanel defaultSize={35} minSize={25}>
              <div className="h-full bg-card">
                <DetailsPanel />
              </div>
            </ResizablePanel>
          </ResizablePanelGroup>
        </div>
      )}

      <footer className="flex items-center justify-center border-t border-border bg-muted/30 px-4 py-2">
        <p className="text-xs text-muted-foreground">Offline — no data leaves your machine.</p>
      </footer>
    </div>
  )
}

