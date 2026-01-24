'use client'

import { useMemo, useCallback, useState } from 'react'
import { useBundleStore } from '@/store/use-bundle-store'
import { bestSelector, nodePath, selectorToJson, generateScriptStep, scriptStepToJson, copyToClipboard } from '@/lib/selector'
import { RawJsonView } from '@/components/raw-json-view'
import { DiffView } from '@/components/diff-view'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'
import { cn } from '@/lib/utils'
import { useTranslation } from '@/hooks/use-i18n'
import {
  AlertTriangle,
  Copy,
  ExternalLink,
  Search,
  FileCode,
} from 'lucide-react'

function InfoRow({ label, value, mono = false, warning = false, missingText = 'Missing' }: { label: string; value?: string | number | null; mono?: boolean; warning?: boolean; missingText?: string }) {
  if (value === undefined || value === null || value === '') {
    if (warning) {
      return (
        <div className="flex items-center gap-2 py-1">
          <span className="text-xs text-muted-foreground w-28 shrink-0">{label}</span>
          <span className="text-xs text-amber-600 flex items-center gap-1">
            <AlertTriangle className="h-3 w-3" />
            {missingText}
          </span>
        </div>
      )
    }
    return null
  }
  return (
    <div className="flex items-start gap-2 py-1">
      <span className="text-xs text-muted-foreground w-28 shrink-0">{label}</span>
      <span className={cn('text-xs text-foreground break-all', mono && 'font-mono')}>{String(value)}</span>
    </div>
  )
}

function SummaryTab() {
  const window = useBundleStore((s) => s.getSelectedWindow())
  const snapshot = useBundleStore((s) => s.getSelectedSnapshotA())
  const redactText = useBundleStore((s) => s.redactText)
  const { t } = useTranslation()

  if (!snapshot) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground">{t('summary.noSnapshot')}</p>
      </div>
    )
  }

  return (
    <ScrollArea className="h-full">
      <div className="p-3 space-y-4">
        {/* Window Info */}
        <div>
          <h4 className="text-xs font-medium text-foreground mb-2">{t('summary.window')}</h4>
          <div className="bg-muted/30 rounded-md p-2">
            <InfoRow label={t('summary.windowId')} value={window?.windowId} mono />
            <InfoRow label={t('summary.scaleFactor')} value={snapshot.scaleFactor} />
            <InfoRow label={t('summary.logicalSize')} value={snapshot.windowSizeLogical ? `${snapshot.windowSizeLogical.w} x ${snapshot.windowSizeLogical.h}` : undefined} />
          </div>
        </div>

        {/* Snapshot Info */}
        <div>
          <h4 className="text-xs font-medium text-foreground mb-2">{t('summary.snapshot')}</h4>
          <div className="bg-muted/30 rounded-md p-2">
            <InfoRow label={t('summary.tickId')} value={snapshot.tickId} mono />
            <InfoRow label={t('summary.frameId')} value={snapshot.frameId} mono />
            <InfoRow label={t('summary.timestamp')} value={snapshot.timestampMonoNs} mono />
          </div>
        </div>

        {/* Focus */}
        <div>
          <h4 className="text-xs font-medium text-foreground mb-2">{t('summary.focus')}</h4>
          <div className="bg-muted/30 rounded-md p-2">
            <InfoRow label={t('summary.focusedNode')} value={snapshot.focus?.focusedNodeId} mono warning missingText={t('summary.missing')} />
            <InfoRow label={t('summary.activeDescendant')} value={snapshot.focus?.activeDescendantId} mono />
          </div>
        </div>

        {/* Overlay Routing */}
        <div>
          <h4 className="text-xs font-medium text-foreground mb-2">{t('summary.overlayRouting')}</h4>
          <div className="bg-muted/30 rounded-md p-2">
            <InfoRow label={t('summary.barrierRoot')} value={snapshot.overlay?.barrierRootId} mono />
            {snapshot.overlay?.layerRoots && snapshot.overlay.layerRoots.length > 0 && (
              <div className="mt-2">
                <span className="text-xs text-muted-foreground">{t('summary.topLayerRoots')}</span>
                <div className="mt-1 space-y-1">
                  {snapshot.overlay.layerRoots.map((lr, i) => (
                    <Badge key={i} variant="outline" className="text-[10px] mr-1">
                      {lr.nodeId} (z:{lr.zIndex})
                    </Badge>
                  ))}
                </div>
              </div>
            )}
            {(!snapshot.overlay?.layerRoots || snapshot.overlay.layerRoots.length === 0) && (
              <InfoRow label={t('summary.layerRoots')} value={undefined} warning={false} />
            )}
          </div>
        </div>

        {/* Hit Test */}
        <div>
          <h4 className="text-xs font-medium text-foreground mb-2">{t('summary.hitTest')}</h4>
          <div className="bg-muted/30 rounded-md p-2">
            <InfoRow
              label={t('summary.pointer')}
              value={snapshot.hitTest?.pointer ? `(${snapshot.hitTest.pointer.x}, ${snapshot.hitTest.pointer.y})` : undefined}
            />
            {snapshot.hitTest?.chain && snapshot.hitTest.chain.length > 0 && (
              <div className="mt-2">
                <span className="text-xs text-muted-foreground">{t('summary.hitChain')}</span>
                <div className="mt-1 space-y-1">
                  {snapshot.hitTest.chain.map((entry, i) => (
                    <div key={i} className="flex items-center gap-1">
                      <Badge variant="outline" className="text-[10px]">
                        {entry.role ?? 'Unknown'}
                      </Badge>
                      {entry.testId && (
                        <span className="text-[10px] text-muted-foreground">{entry.testId}</span>
                      )}
                      {entry.label && !redactText && (
                        <span className="text-[10px] text-muted-foreground truncate">{entry.label}</span>
                      )}
                      {entry.label && redactText && (
                        <span className="text-[10px] text-muted-foreground">•••</span>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </ScrollArea>
  )
}

function NodeTab() {
  const snapshot = useBundleStore((s) => s.getSelectedSnapshotA())
  const node = useBundleStore((s) => s.getSelectedNode())
  const redactText = useBundleStore((s) => s.redactText)
  const setSelectedNodeId = useBundleStore((s) => s.setSelectedNodeId)
  const expandNode = useBundleStore((s) => s.expandNode)
  const { t } = useTranslation()

  const handleCopySelector = useCallback(() => {
    if (!node) return
    copyToClipboard(selectorToJson(bestSelector(node)))
  }, [node])

  const handleCopyScriptStep = useCallback(() => {
    if (!node) return
    copyToClipboard(scriptStepToJson(generateScriptStep(node)))
  }, [node])

  const handleJumpToNode = useCallback(
    (nodeId: string) => {
      // Expand the path to the node
      if (snapshot?.semantics) {
        let current = snapshot.semantics.nodesById[nodeId]
        while (current?.parentId) {
          expandNode(current.parentId)
          current = snapshot.semantics.nodesById[current.parentId]
        }
      }
      setSelectedNodeId(nodeId)
    },
    [snapshot, setSelectedNodeId, expandNode]
  )

  if (!node) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground">{t('node.noSelection')}</p>
      </div>
    )
  }

  const displayLabel = redactText ? (node.label ? '•••' : undefined) : node.label
  const displayName = redactText ? (node.name ? '•••' : undefined) : node.name
  const path = snapshot?.semantics ? nodePath(node, snapshot.semantics) : node.id

  return (
    <ScrollArea className="h-full">
      <div className="p-3 space-y-4">
        {/* Actions */}
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={handleCopySelector}>
            <Copy className="h-3.5 w-3.5 mr-1.5" />
            {t('node.copySelector')}
          </Button>
          <Button variant="outline" size="sm" onClick={handleCopyScriptStep}>
            <FileCode className="h-3.5 w-3.5 mr-1.5" />
            {t('node.scriptStep')}
          </Button>
        </div>

        {/* Path */}
        <div>
          <h4 className="text-xs font-medium text-foreground mb-1">{t('node.path')}</h4>
          <p className="text-xs font-mono text-muted-foreground bg-muted/30 rounded px-2 py-1 break-all">
            {path}
          </p>
        </div>

        {/* Properties */}
        <div>
          <h4 className="text-xs font-medium text-foreground mb-2">{t('node.properties')}</h4>
          <div className="bg-muted/30 rounded-md p-2">
            <InfoRow label={t('node.id')} value={node.id} mono />
            <InfoRow label={t('node.role')} value={node.role} />
            <InfoRow label={t('node.testId')} value={node.testId} mono />
            <InfoRow label={t('node.label')} value={displayLabel} />
            <InfoRow label={t('node.name')} value={displayName} />
          </div>
        </div>

        {/* Bounds */}
        {node.bounds && (
          <div>
            <h4 className="text-xs font-medium text-foreground mb-2">{t('node.bounds')}</h4>
            <div className="bg-muted/30 rounded-md p-2">
              <InfoRow label="X" value={node.bounds.x} />
              <InfoRow label="Y" value={node.bounds.y} />
              <InfoRow label="Width" value={node.bounds.w} />
              <InfoRow label="Height" value={node.bounds.h} />
            </div>
          </div>
        )}

        {/* Flags */}
        {node.flags && Object.keys(node.flags).length > 0 && (
          <div>
            <h4 className="text-xs font-medium text-foreground mb-2">{t('node.flags')}</h4>
            <div className="flex flex-wrap gap-1">
              {Object.entries(node.flags).map(([key, value]) => (
                <Badge
                  key={key}
                  variant="outline"
                  className={cn(
                    'text-[10px]',
                    value ? 'bg-green-500/10 text-green-600 border-green-500/30 dark:text-green-400' : 'bg-muted text-muted-foreground'
                  )}
                >
                  {key}: {String(value)}
                </Badge>
              ))}
            </div>
          </div>
        )}

        {/* Actions */}
        {node.actions && Object.keys(node.actions).length > 0 && (
          <div>
            <h4 className="text-xs font-medium text-foreground mb-2">{t('node.actions')}</h4>
            <div className="flex flex-wrap gap-1">
              {Object.entries(node.actions)
                .filter(([, value]) => value)
                .map(([key]) => (
                  <Badge key={key} variant="secondary" className="text-[10px]">
                    {key}
                  </Badge>
                ))}
            </div>
          </div>
        )}

        {/* Relationships */}
        <div>
          <h4 className="text-xs font-medium text-foreground mb-2">{t('node.relationships')}</h4>
          <div className="bg-muted/30 rounded-md p-2 space-y-2">
            {node.parentId && (
              <div className="flex items-center gap-2">
                <span className="text-xs text-muted-foreground w-16">{t('node.parent')}</span>
                <Button
                  variant="link"
                  size="sm"
                  className="h-auto p-0 text-xs font-mono"
                  onClick={() => handleJumpToNode(node.parentId!)}
                >
                  {node.parentId}
                  <ExternalLink className="h-3 w-3 ml-1" />
                </Button>
              </div>
            )}
            {node.children.length > 0 && (
              <div>
                <span className="text-xs text-muted-foreground">{t('node.children')} ({node.children.length}):</span>
                <div className="mt-1 flex flex-wrap gap-1">
                  {node.children.slice(0, 10).map((childId) => (
                    <Button
                      key={childId}
                      variant="outline"
                      size="sm"
                      className="h-auto px-1.5 py-0.5 text-[10px] font-mono bg-transparent"
                      onClick={() => handleJumpToNode(childId)}
                    >
                      {childId}
                    </Button>
                  ))}
                  {node.children.length > 10 && (
                    <span className="text-xs text-muted-foreground">{t('node.more', { count: node.children.length - 10 })}</span>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </ScrollArea>
  )
}

function EventsTab() {
  const snapshot = useBundleStore((s) => s.getSelectedSnapshotA())
  const window = useBundleStore((s) => s.getSelectedWindow())
  const eventsSearchQuery = useBundleStore((s) => s.eventsSearchQuery)
  const setEventsSearchQuery = useBundleStore((s) => s.setEventsSearchQuery)
  const { t } = useTranslation()

  const events = window?.events ?? snapshot?.events ?? []

  const filteredEvents = useMemo(() => {
    let out = events

    // Best-effort: filter to events at/before the selected snapshot if ids are available.
    if (snapshot?.tickId) {
      const maxTick = Number(snapshot.tickId)
      if (!Number.isNaN(maxTick)) {
        out = out.filter((e) => (e.tickId ? Number(e.tickId) <= maxTick : true))
      }
    }

    if (!eventsSearchQuery) return out
    const query = eventsSearchQuery.toLowerCase()
    return out.filter(
      (e) => e.kind.toLowerCase().includes(query) || e.summary.toLowerCase().includes(query)
    )
  }, [events, eventsSearchQuery, snapshot?.tickId])

  if (events.length === 0) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground">{t('events.noEvents')}</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      <div className="px-3 py-2 border-b border-border">
        <div className="relative">
          <Search className="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder={t('events.searchPlaceholder')}
            value={eventsSearchQuery}
            onChange={(e) => setEventsSearchQuery(e.target.value)}
            className="pl-7 h-7 text-xs"
          />
        </div>
      </div>
      <ScrollArea className="flex-1">
        <div className="p-2 space-y-1">
          {filteredEvents.map((event, i) => (
            <div key={i} className="flex items-start gap-2 px-2 py-1.5 rounded bg-muted/30">
              <Badge variant="outline" className="text-[10px] shrink-0">
                {event.kind}
              </Badge>
              <span className="text-xs text-foreground">{event.summary}</span>
            </div>
          ))}
          {filteredEvents.length === 0 && (
            <p className="text-sm text-muted-foreground text-center py-4">{t('events.noMatch')}</p>
          )}
        </div>
      </ScrollArea>
    </div>
  )
}

function PerformanceTab() {
  const snapshot = useBundleStore((s) => s.getSelectedSnapshotA())
  const { t } = useTranslation()

  const perf = snapshot?.perf

  if (!perf) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground">{t('perf.noData')}</p>
      </div>
    )
  }

  const invalidations = perf.invalidations as { count?: number; sources?: string[] } | undefined

  return (
    <ScrollArea className="h-full">
      <div className="p-3 space-y-4">
        {/* Timing */}
        <div>
          <h4 className="text-xs font-medium text-foreground mb-2">{t('perf.timing')}</h4>
          <div className="bg-muted/30 rounded-md p-2">
            <InfoRow label={t('perf.total')} value={perf.totalUs !== undefined ? `${perf.totalUs} μs` : undefined} />
            <InfoRow label={t('perf.layout')} value={perf.layoutUs !== undefined ? `${perf.layoutUs} μs` : undefined} />
            <InfoRow label={t('perf.prepaint')} value={perf.prepaintUs !== undefined ? `${perf.prepaintUs} μs` : undefined} />
            <InfoRow label={t('perf.paint')} value={perf.paintUs !== undefined ? `${perf.paintUs} μs` : undefined} />
          </div>
        </div>

        {/* Timing Bar */}
        {(perf.layoutUs || perf.prepaintUs || perf.paintUs) && (
          <div>
            <h4 className="text-xs font-medium text-foreground mb-2">{t('perf.timingBreakdown')}</h4>
            <div className="flex h-6 rounded overflow-hidden">
              {perf.layoutUs !== undefined && perf.layoutUs > 0 && (
                <div
                  className="bg-blue-500 flex items-center justify-center"
                  style={{
                    width: `${((perf.layoutUs / (perf.totalUs ?? perf.layoutUs)) * 100).toFixed(1)}%`,
                  }}
                  title={`Layout: ${perf.layoutUs}μs`}
                >
                  <span className="text-[10px] text-white font-medium">L</span>
                </div>
              )}
              {perf.prepaintUs !== undefined && perf.prepaintUs > 0 && (
                <div
                  className="bg-amber-500 flex items-center justify-center"
                  style={{
                    width: `${((perf.prepaintUs / (perf.totalUs ?? perf.prepaintUs)) * 100).toFixed(1)}%`,
                  }}
                  title={`Prepaint: ${perf.prepaintUs}μs`}
                >
                  <span className="text-[10px] text-white font-medium">P</span>
                </div>
              )}
              {perf.paintUs !== undefined && perf.paintUs > 0 && (
                <div
                  className="bg-green-500 flex items-center justify-center"
                  style={{
                    width: `${((perf.paintUs / (perf.totalUs ?? perf.paintUs)) * 100).toFixed(1)}%`,
                  }}
                  title={`Paint: ${perf.paintUs}μs`}
                >
                  <span className="text-[10px] text-white font-medium">R</span>
                </div>
              )}
            </div>
            <div className="flex gap-4 mt-2 text-[10px]">
              <span className="flex items-center gap-1">
                <span className="w-2 h-2 rounded bg-blue-500" />
                {t('perf.layout')}
              </span>
              <span className="flex items-center gap-1">
                <span className="w-2 h-2 rounded bg-amber-500" />
                {t('perf.prepaint')}
              </span>
              <span className="flex items-center gap-1">
                <span className="w-2 h-2 rounded bg-green-500" />
                {t('perf.paint')}
              </span>
            </div>
          </div>
        )}

        {/* Cache */}
        {perf.cache && (
          <div>
            <h4 className="text-xs font-medium text-foreground mb-2">{t('perf.cache')}</h4>
            <div className="bg-muted/30 rounded-md p-2">
              <InfoRow label={t('perf.hits')} value={perf.cache.hits} />
              <InfoRow label={t('perf.misses')} value={perf.cache.misses} />
              {perf.cache.hits !== undefined && perf.cache.misses !== undefined && (
                <InfoRow
                  label={t('perf.hitRate')}
                  value={`${(
                    (perf.cache.hits / (perf.cache.hits + perf.cache.misses)) *
                    100
                  ).toFixed(1)}%`}
                />
              )}
            </div>
          </div>
        )}

        {/* Invalidations */}
        {invalidations && (
          <div>
            <h4 className="text-xs font-medium text-foreground mb-2">{t('perf.invalidations')}</h4>
            <div className="bg-muted/30 rounded-md p-2">
              <InfoRow label={t('perf.count')} value={invalidations.count} />
              {invalidations.sources && (
                <div className="mt-2">
                  <span className="text-xs text-muted-foreground">{t('perf.sources')}</span>
                  <div className="flex flex-wrap gap-1 mt-1">
                    {invalidations.sources.map((source, i) => (
                      <Badge key={i} variant="secondary" className="text-[10px]">
                        {source}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </ScrollArea>
  )
}

function RawTab() {
  const snapshot = useBundleStore((s) => s.getSelectedSnapshotA())
  const [viewMode, setViewMode] = useState<'raw' | 'normalized'>('raw')
  const { t } = useTranslation()

  if (!snapshot) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground">{t('raw.noSnapshot')}</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-3 py-2 border-b border-border">
        <Button
          variant={viewMode === 'raw' ? 'secondary' : 'ghost'}
          size="sm"
          className="h-7 text-xs"
          onClick={() => setViewMode('raw')}
        >
          {t('raw.rawJson')}
        </Button>
        <Button
          variant={viewMode === 'normalized' ? 'secondary' : 'ghost'}
          size="sm"
          className="h-7 text-xs"
          onClick={() => setViewMode('normalized')}
        >
          {t('raw.normalized')}
        </Button>
      </div>
      <div className="flex-1 overflow-hidden">
        <RawJsonView data={viewMode === 'raw' ? snapshot.raw : snapshot} />
      </div>
    </div>
  )
}

export function DetailsPanel() {
  const bundle = useBundleStore((s) => s.bundle)
  const compareMode = useBundleStore((s) => s.compareMode)
  const { t } = useTranslation()

  if (!bundle) {
    return (
      <div className="flex flex-col h-full">
        <div className="flex items-center justify-between px-3 py-2 border-b border-border bg-muted/30">
          <h2 className="text-sm font-medium text-foreground">{t('details.summary')}</h2>
        </div>
        <div className="flex-1 flex items-center justify-center p-4">
          <p className="text-sm text-muted-foreground text-center">{t('snapshots.noBundle')}</p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      <Tabs defaultValue="summary" className="flex flex-col h-full">
        <div className="px-3 py-2 border-b border-border bg-muted/30">
          <TabsList className="h-8">
            <TabsTrigger value="summary" className="text-xs px-2 h-6">
              {t('details.summary')}
            </TabsTrigger>
            <TabsTrigger value="node" className="text-xs px-2 h-6">
              {t('details.node')}
            </TabsTrigger>
            <TabsTrigger value="events" className="text-xs px-2 h-6">
              {t('details.events')}
            </TabsTrigger>
            <TabsTrigger value="performance" className="text-xs px-2 h-6">
              {t('details.performance')}
            </TabsTrigger>
            {compareMode && (
              <TabsTrigger value="diff" className="text-xs px-2 h-6">
                {t('details.diff')}
              </TabsTrigger>
            )}
            <TabsTrigger value="raw" className="text-xs px-2 h-6">
              {t('details.raw')}
            </TabsTrigger>
          </TabsList>
        </div>

        <TabsContent value="summary" className="flex-1 mt-0 overflow-hidden">
          <SummaryTab />
        </TabsContent>
        <TabsContent value="node" className="flex-1 mt-0 overflow-hidden">
          <NodeTab />
        </TabsContent>
        <TabsContent value="events" className="flex-1 mt-0 overflow-hidden">
          <EventsTab />
        </TabsContent>
        <TabsContent value="performance" className="flex-1 mt-0 overflow-hidden">
          <PerformanceTab />
        </TabsContent>
        {compareMode && (
          <TabsContent value="diff" className="flex-1 mt-0 overflow-hidden">
            <DiffView />
          </TabsContent>
        )}
        <TabsContent value="raw" className="flex-1 mt-0 overflow-hidden">
          <RawTab />
        </TabsContent>
      </Tabs>
    </div>
  )
}
