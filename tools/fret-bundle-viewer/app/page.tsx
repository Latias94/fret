'use client'

import { useBundleStore } from '@/store/use-bundle-store'
import { HeaderBar } from '@/components/header-bar'
import { SemanticsTreePanel } from '@/components/semantics-tree-panel'
import { SnapshotsPanel } from '@/components/snapshots-panel'
import { DetailsPanel } from '@/components/details-panel'
import { EmptyState } from '@/components/empty-state'
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from '@/components/ui/resizable'

export default function FretBundleViewer() {
  const bundle = useBundleStore((s) => s.bundle)
  const parseError = useBundleStore((s) => s.parseError)

  return (
    <div className="flex flex-col h-screen bg-background">
      <HeaderBar />
      
      {!bundle && !parseError ? (
        <EmptyState />
      ) : parseError ? (
        <EmptyState />
      ) : (
        <div className="flex-1 overflow-hidden">
          <ResizablePanelGroup direction="horizontal" className="h-full">
            {/* Left Panel: Semantics Tree */}
            <ResizablePanel defaultSize={30} minSize={20} maxSize={50}>
              <div className="h-full border-r border-border bg-card">
                <SemanticsTreePanel />
              </div>
            </ResizablePanel>
            
            <ResizableHandle withHandle />
            
            {/* Middle Panel: Windows & Snapshots */}
            <ResizablePanel defaultSize={35} minSize={25} maxSize={50}>
              <div className="h-full border-r border-border bg-card">
                <SnapshotsPanel />
              </div>
            </ResizablePanel>
            
            <ResizableHandle withHandle />
            
            {/* Right Panel: Details */}
            <ResizablePanel defaultSize={35} minSize={25}>
              <div className="h-full bg-card">
                <DetailsPanel />
              </div>
            </ResizablePanel>
          </ResizablePanelGroup>
        </div>
      )}

      {/* Footer */}
      <footer className="flex items-center justify-center px-4 py-2 border-t border-border bg-muted/30">
        <p className="text-xs text-muted-foreground">
          Offline — no data leaves your machine.
        </p>
      </footer>
    </div>
  )
}
