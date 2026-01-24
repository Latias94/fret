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

type JsonObject = Record<string, unknown>

function asObject(v: unknown): JsonObject | null {
  if (!v || typeof v !== 'object' || Array.isArray(v)) return null
  return v as JsonObject
}

function asArray(v: unknown): unknown[] | null {
  return Array.isArray(v) ? v : null
}

function toOptString(v: unknown): string | undefined {
  if (v === null || v === undefined) return undefined
  if (typeof v === 'string') return v
  if (typeof v === 'number' || typeof v === 'bigint') return String(v)
  return undefined
}

function toOptNumber(v: unknown): number | undefined {
  if (typeof v === 'number') return Number.isFinite(v) ? v : undefined
  return undefined
}

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

  const triage = useMemo(() => {
    if (!snapshot) return []

    type Item = { level: 'info' | 'warn'; text: string }
    const out: Item[] = []

    const totalUs = snapshot.perf?.totalUs
    const layoutUs = snapshot.perf?.layoutUs
    const prepaintUs = snapshot.perf?.prepaintUs
    const paintUs = snapshot.perf?.paintUs

    if (typeof totalUs === 'number') {
      if (totalUs >= 1_000_000) out.push({ level: 'warn', text: t('triage.severeFrameTime', { us: totalUs }) })
      else if (totalUs >= 33_000) out.push({ level: 'warn', text: t('triage.slowFrameTime', { us: totalUs }) })
      else out.push({ level: 'info', text: t('triage.frameTime', { us: totalUs }) })

      const phases: Array<{ phase: string; us?: number }> = [
        { phase: 'layout', us: layoutUs },
        { phase: 'prepaint', us: prepaintUs },
        { phase: 'paint', us: paintUs },
      ]
      const best = phases
        .filter((p) => typeof p.us === 'number')
        .sort((a, b) => (b.us ?? 0) - (a.us ?? 0))[0]
      if (best && typeof best.us === 'number') {
        out.push({ level: 'info', text: t('triage.dominantPhase', { phase: best.phase, us: best.us }) })
      }
    }

    const inv = snapshot.perf?.invalidations as any
    const invCalls = typeof inv?.count === 'number' ? inv.count : undefined
    const invNodes = typeof inv?.nodes === 'number' ? inv.nodes : undefined
    if (invCalls !== undefined || invNodes !== undefined) {
      const level = (invCalls ?? 0) >= 1000 ? 'warn' : (invCalls ?? 0) >= 200 ? 'warn' : 'info'
      out.push({ level, text: t('triage.invalidation', { calls: invCalls ?? '—', nodes: invNodes ?? '—' }) })
    }

    if (snapshot.overlay?.barrierRootId) {
      out.push({ level: 'info', text: t('triage.barrierActive', { id: snapshot.overlay.barrierRootId }) })
    }

    const blocking = snapshot.overlay?.layerRoots?.filter((r) => r.blocksUnderlay).map((r) => r.nodeId).filter(Boolean)
    if (blocking && blocking.length > 0) {
      out.push({ level: 'warn', text: t('triage.blocksUnderlay', { count: blocking.length }) })
    }

    const paintCacheHits = snapshot.perf?.cache?.hits
    const paintCacheMisses = snapshot.perf?.cache?.misses
    if (paintCacheHits !== undefined || paintCacheMisses !== undefined) {
      const misses = paintCacheMisses ?? 0
      const hits = paintCacheHits ?? 0
      if (hits + misses > 0 && misses > hits) {
        out.push({ level: 'warn', text: t('triage.paintCacheMisses', { hits, misses }) })
      } else {
        out.push({ level: 'info', text: t('triage.paintCache', { hits, misses }) })
      }
    }

    return out
  }, [snapshot, t])

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
                      {lr.nodeId}
                      {lr.zIndex !== undefined ? ` (z:${lr.zIndex})` : ''}
                      {lr.blocksUnderlay ? ' block' : ''}
                      {lr.hitTestable === false ? ' no-hit' : ''}
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

        {/* Triage */}
        <div>
          <h4 className="text-xs font-medium text-foreground mb-2">{t('summary.triage')}</h4>
          <div className="bg-muted/30 rounded-md p-2 space-y-1">
            {triage.length === 0 ? (
              <span className="text-xs text-muted-foreground">{t('triage.noSignals')}</span>
            ) : (
              triage.map((item, i) => (
                <div key={i} className="flex items-start gap-2 py-1">
                  <Badge
                    variant="outline"
                    className={cn(
                      'text-[10px] mt-0.5',
                      item.level === 'warn'
                        ? 'border-amber-500/30 bg-amber-500/10 text-amber-600 dark:text-amber-400'
                        : 'text-muted-foreground'
                    )}
                  >
                    {item.level}
                  </Badge>
                  <span className="text-xs text-foreground">{item.text}</span>
                </div>
              ))
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
  const [scope, setScope] = useState<'all' | 'upto' | 'tick'>('upto')
  const [expandedIndex, setExpandedIndex] = useState<number | null>(null)

  const filteredEvents = useMemo(() => {
    let out = events

    // Best-effort: filter to events at/before the selected snapshot if ids are available.
    if (snapshot?.tickId) {
      const snapTick = Number(snapshot.tickId)
      if (!Number.isNaN(snapTick)) {
        if (scope === 'upto') {
          out = out.filter((e) => (e.tickId ? Number(e.tickId) <= snapTick : true))
        } else if (scope === 'tick') {
          out = out.filter((e) => (e.tickId ? Number(e.tickId) === snapTick : false))
        }
      }
    } else if (scope !== 'all') {
      scope
      // No tick id: fall back to showing all events.
    }

    if (!eventsSearchQuery) return out
    const query = eventsSearchQuery.toLowerCase()
    return out.filter(
      (e) => e.kind.toLowerCase().includes(query) || e.summary.toLowerCase().includes(query)
    )
  }, [events, eventsSearchQuery, snapshot?.tickId, scope])

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
        <div className="flex items-center gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder={t('events.searchPlaceholder')}
              value={eventsSearchQuery}
              onChange={(e) => setEventsSearchQuery(e.target.value)}
              className="pl-7 h-7 text-xs"
            />
          </div>
          <div className="flex items-center gap-1">
            <Button
              variant={scope === 'all' ? 'secondary' : 'outline'}
              size="sm"
              className="h-7 px-2 text-[11px]"
              onClick={() => setScope('all')}
            >
              {t('events.scopeAll')}
            </Button>
            <Button
              variant={scope === 'upto' ? 'secondary' : 'outline'}
              size="sm"
              className="h-7 px-2 text-[11px]"
              onClick={() => setScope('upto')}
              disabled={!snapshot?.tickId}
              title={!snapshot?.tickId ? t('summary.missing') : undefined}
            >
              {t('events.scopeUpTo')}
            </Button>
            <Button
              variant={scope === 'tick' ? 'secondary' : 'outline'}
              size="sm"
              className="h-7 px-2 text-[11px]"
              onClick={() => setScope('tick')}
              disabled={!snapshot?.tickId}
              title={!snapshot?.tickId ? t('summary.missing') : undefined}
            >
              {t('events.scopeTick')}
            </Button>
          </div>
        </div>
      </div>
      <ScrollArea className="flex-1">
        <div className="p-2 space-y-1">
          {filteredEvents.map((event, i) => {
            const isExpanded = expandedIndex === i
            return (
              <div
                key={`${event.kind}-${event.tickId ?? ''}-${event.frameId ?? ''}-${i}`}
                className="rounded bg-muted/30 px-2 py-1.5"
              >
                <button
                  type="button"
                  className="w-full text-left"
                  onClick={() => setExpandedIndex(isExpanded ? null : i)}
                >
                  <div className="flex items-center gap-2">
                    <Badge variant="outline" className="text-[10px] shrink-0">
                      {event.kind}
                    </Badge>
                    {event.tickId && (
                      <Badge variant="secondary" className="text-[10px] font-mono">
                        {t('events.tick')}: {event.tickId}
                      </Badge>
                    )}
                    {event.frameId && (
                      <Badge variant="secondary" className="text-[10px] font-mono">
                        {t('events.frame')}: {event.frameId}
                      </Badge>
                    )}
                    <span className="ml-auto text-[10px] text-muted-foreground">
                      {isExpanded ? '–' : '+'}
                    </span>
                  </div>
                  <div className="mt-1 text-xs text-foreground break-words">{event.summary}</div>
                </button>
                {isExpanded && event.raw && (
                  <pre className="mt-2 max-h-40 overflow-auto rounded bg-background/40 p-2 text-[10px] text-muted-foreground">
                    {JSON.stringify(event.raw, null, 2).slice(0, 4000)}
                  </pre>
                )}
              </div>
            )
          })}
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
  const debug = asObject(asObject(snapshot?.raw)?.debug)

  if (!perf) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground">{t('perf.noData')}</p>
      </div>
    )
  }

  const invalidations = perf.invalidations as { count?: number; sources?: string[] } | undefined

  const invalidationWalkSummary = useMemo(() => {
    const rows = asArray(debug?.invalidation_walks) ?? []
    if (rows.length === 0) return null

    const byKind = new Map<string, number>()
    const bySource = new Map<string, number>()
    let totalWalkedNodes = 0

    type WalkRow = {
      walkedNodes: number
      kind: string
      source: string
      detail?: string
      truncatedAt?: number
      rootNode?: string
      rootElement?: string
    }

    const top: WalkRow[] = []

    for (const r of rows) {
      const o = asObject(r)
      if (!o) continue

      const walkedNodes = toOptNumber(o.walked_nodes) ?? 0
      const kind = toOptString(o.kind) ?? 'unknown'
      const source = toOptString(o.source) ?? 'unknown'
      const detail = toOptString(o.detail)
      const truncatedAt = toOptNumber(o.truncated_at)
      const rootNode = toOptString(o.root_node)
      const rootElement = toOptString(o.root_element)

      totalWalkedNodes += walkedNodes
      byKind.set(kind, (byKind.get(kind) ?? 0) + 1)
      bySource.set(source, (bySource.get(source) ?? 0) + 1)

      top.push({ walkedNodes, kind, source, detail, truncatedAt, rootNode, rootElement })
    }

    top.sort((a, b) => b.walkedNodes - a.walkedNodes)

    const topKinds = [...byKind.entries()].sort((a, b) => b[1] - a[1]).slice(0, 6)
    const topSources = [...bySource.entries()].sort((a, b) => b[1] - a[1]).slice(0, 6)

    return {
      total: rows.length,
      totalWalkedNodes,
      topKinds,
      topSources,
      topWalks: top.slice(0, 20),
    }
  }, [debug])

  const layoutSolveSummary = useMemo(() => {
    const solves = asArray(debug?.layout_engine_solves) ?? []
    if (solves.length === 0) return null

    type TopMeasure = {
      node?: string
      elementKind?: string
      measureUs: number
      calls?: number
      cacheHits?: number
    }

    type SolveRow = {
      rootNode?: string
      solveUs?: number
      measureUs?: number
      measureCalls?: number
      measureCacheHits?: number
      topMeasures: TopMeasure[]
    }

    const out: SolveRow[] = []

    for (const s of solves) {
      const o = asObject(s)
      if (!o) continue

      const topMeasuresRaw = asArray(o.top_measures) ?? []
      const topMeasures: TopMeasure[] = []
      for (const tm of topMeasuresRaw.slice(0, 12)) {
        const tmo = asObject(tm)
        if (!tmo) continue
        topMeasures.push({
          node: toOptString(tmo.node),
          elementKind: toOptString(tmo.element_kind),
          measureUs: toOptNumber(tmo.measure_time_us) ?? 0,
          calls: toOptNumber(tmo.calls),
          cacheHits: toOptNumber(tmo.cache_hits),
        })
      }

      out.push({
        rootNode: toOptString(o.root_node),
        solveUs: toOptNumber(o.solve_time_us),
        measureUs: toOptNumber(o.measure_time_us),
        measureCalls: toOptNumber(o.measure_calls),
        measureCacheHits: toOptNumber(o.measure_cache_hits),
        topMeasures,
      })
    }

    return out
  }, [debug])

  const cacheRootsSummary = useMemo(() => {
    const roots = asArray(debug?.cache_roots) ?? []
    if (roots.length === 0) return null

    type CacheRootRow = {
      rootNode?: string
      reused?: boolean
      reuseReason?: string
      elementPath?: string
      paintReplayedOps?: number
      subtreeNodes?: number
      directChildNodes?: number
      containedLayout?: boolean
    }

    const rows: CacheRootRow[] = []
    let reusedCount = 0

    for (const r of roots) {
      const o = asObject(r)
      if (!o) continue
      const reused = typeof o.reused === 'boolean' ? o.reused : undefined
      if (reused) reusedCount += 1
      rows.push({
        rootNode: toOptString(o.root),
        reused,
        reuseReason: toOptString(o.reuse_reason),
        elementPath: toOptString(o.element_path),
        paintReplayedOps: toOptNumber(o.paint_replayed_ops),
        subtreeNodes: toOptNumber(o.subtree_nodes),
        directChildNodes: toOptNumber(o.direct_child_nodes),
        containedLayout: typeof o.contained_layout === 'boolean' ? o.contained_layout : undefined,
      })
    }

    return {
      total: roots.length,
      reused: reusedCount,
      rows: rows.slice(0, 20),
    }
  }, [debug])

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
              {'nodes' in (invalidations as any) && (
                <InfoRow label={t('perf.nodes')} value={(invalidations as any).nodes} />
              )}
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

        {/* Invalidation walks (debug) */}
        {invalidationWalkSummary && (
          <div>
            <h4 className="text-xs font-medium text-foreground mb-2">{t('perf.invalidationWalks')}</h4>
            <div className="bg-muted/30 rounded-md p-2 space-y-2">
              <InfoRow label={t('perf.count')} value={invalidationWalkSummary.total} />
              <InfoRow label={t('perf.walkedNodes')} value={invalidationWalkSummary.totalWalkedNodes} />

              {invalidationWalkSummary.topKinds.length > 0 && (
                <div>
                  <div className="text-xs text-muted-foreground">{t('perf.topKinds')}</div>
                  <div className="flex flex-wrap gap-1 mt-1">
                    {invalidationWalkSummary.topKinds.map(([k, n]) => (
                      <Badge key={k} variant="secondary" className="text-[10px]">
                        {k}:{n}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}

              {invalidationWalkSummary.topSources.length > 0 && (
                <div>
                  <div className="text-xs text-muted-foreground">{t('perf.topSources')}</div>
                  <div className="flex flex-wrap gap-1 mt-1">
                    {invalidationWalkSummary.topSources.map(([k, n]) => (
                      <Badge key={k} variant="secondary" className="text-[10px]">
                        {k}:{n}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}

              {invalidationWalkSummary.topWalks.length > 0 && (
                <div>
                  <div className="text-xs text-muted-foreground">{t('perf.topWalks')}</div>
                  <div className="mt-2 space-y-1">
                    {invalidationWalkSummary.topWalks.map((w, i) => (
                      <div key={i} className="flex items-center gap-2 rounded bg-background/40 px-2 py-1">
                        <Badge variant="outline" className="text-[10px] font-mono">
                          {w.walkedNodes}
                        </Badge>
                        <Badge variant="outline" className="text-[10px]">
                          {w.kind}
                        </Badge>
                        <Badge variant="outline" className="text-[10px]">
                          {w.source}
                        </Badge>
                        {w.detail && (
                          <span className="text-[10px] text-muted-foreground truncate">
                            {w.detail}
                          </span>
                        )}
                        {w.truncatedAt !== undefined && (
                          <Badge variant="outline" className="text-[10px] border-amber-500/30 bg-amber-500/10 text-amber-600 dark:text-amber-400">
                            {t('perf.truncated')}
                          </Badge>
                        )}
                        <span className="ml-auto text-[10px] text-muted-foreground font-mono">
                          {w.rootNode ?? w.rootElement ?? ''}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Layout engine solves (debug) */}
        {layoutSolveSummary && (
          <div>
            <h4 className="text-xs font-medium text-foreground mb-2">{t('perf.layoutSolves')}</h4>
            <div className="space-y-2">
              {layoutSolveSummary.map((solve, i) => (
                <div key={i} className="bg-muted/30 rounded-md p-2">
                  <InfoRow label={t('perf.rootNode')} value={solve.rootNode} mono />
                  <InfoRow label={t('perf.solveUs')} value={solve.solveUs} />
                  <InfoRow label={t('perf.measureUs')} value={solve.measureUs} />
                  <InfoRow label={t('perf.measureCalls')} value={solve.measureCalls} />
                  <InfoRow label={t('perf.measureCacheHits')} value={solve.measureCacheHits} />

                  {solve.topMeasures.length > 0 && (
                    <div className="mt-2">
                      <div className="text-xs text-muted-foreground">{t('perf.topMeasures')}</div>
                      <div className="mt-1 space-y-1">
                        {solve.topMeasures.slice(0, 10).map((m, idx) => (
                          <div key={idx} className="flex items-center gap-2 rounded bg-background/40 px-2 py-1">
                            <Badge variant="outline" className="text-[10px] font-mono">
                              {m.measureUs}
                            </Badge>
                            <Badge variant="outline" className="text-[10px]">
                              {m.elementKind ?? 'unknown'}
                            </Badge>
                            {m.calls !== undefined && (
                              <span className="text-[10px] text-muted-foreground">
                                {t('perf.calls')}: {m.calls}
                              </span>
                            )}
                            {m.cacheHits !== undefined && (
                              <span className="text-[10px] text-muted-foreground">
                                {t('perf.cacheHits')}: {m.cacheHits}
                              </span>
                            )}
                            <span className="ml-auto text-[10px] text-muted-foreground font-mono">
                              {m.node ?? ''}
                            </span>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Cache roots (debug) */}
        {cacheRootsSummary && (
          <div>
            <h4 className="text-xs font-medium text-foreground mb-2">{t('perf.cacheRoots')}</h4>
            <div className="bg-muted/30 rounded-md p-2 space-y-2">
              <InfoRow label={t('perf.count')} value={cacheRootsSummary.total} />
              <InfoRow label={t('perf.reused')} value={cacheRootsSummary.reused} />

              <div className="space-y-1">
                {cacheRootsSummary.rows.map((r, i) => (
                  <div key={i} className="rounded bg-background/40 px-2 py-1">
                    <div className="flex items-center gap-2">
                      <Badge variant="outline" className="text-[10px] font-mono">
                        {r.rootNode ?? '—'}
                      </Badge>
                      {r.reused !== undefined && (
                        <Badge
                          variant="outline"
                          className={cn(
                            'text-[10px]',
                            r.reused
                              ? 'border-green-500/30 bg-green-500/10 text-green-600 dark:text-green-400'
                              : 'text-muted-foreground'
                          )}
                        >
                          {r.reused ? t('perf.reusedYes') : t('perf.reusedNo')}
                        </Badge>
                      )}
                      {r.reuseReason && (
                        <Badge variant="secondary" className="text-[10px]">
                          {r.reuseReason}
                        </Badge>
                      )}
                      {r.paintReplayedOps !== undefined && (
                        <span className="text-[10px] text-muted-foreground">
                          {t('perf.replayedOps')}: {r.paintReplayedOps}
                        </span>
                      )}
                      {r.subtreeNodes !== undefined && (
                        <span className="text-[10px] text-muted-foreground">
                          {t('perf.subtreeNodes')}: {r.subtreeNodes}
                        </span>
                      )}
                    </div>
                    {r.elementPath && (
                      <div className="mt-1 text-[10px] text-muted-foreground break-all">
                        {r.elementPath.length > 180 ? `${r.elementPath.slice(0, 180)}…` : r.elementPath}
                      </div>
                    )}
                  </div>
                ))}
              </div>
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
