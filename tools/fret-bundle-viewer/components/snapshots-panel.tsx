'use client'

import React from "react"

import { useMemo, useCallback, useEffect, useRef, useState } from 'react'
import { useBundleStore } from '@/store/use-bundle-store'
import type { SnapshotModel } from '@/lib/types'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { cn } from '@/lib/utils'
import { useTranslation } from '@/hooks/use-i18n'
import { Clock, Layers, Zap, Database } from 'lucide-react'

interface SnapshotRowProps {
  snapshot: SnapshotModel
  index: number
  isSelectedA: boolean
  isSelectedB: boolean
  compareMode: boolean
  onSelectA: (index: number) => void
  onSelectB: (index: number) => void
}

function SnapshotRow({
  snapshot,
  index,
  isSelectedA,
  isSelectedB,
  compareMode,
  onSelectA,
  onSelectB,
}: SnapshotRowProps) {
  const handleClick = useCallback(() => {
    if (compareMode && isSelectedA) {
      onSelectB(index)
    } else {
      onSelectA(index)
    }
  }, [compareMode, isSelectedA, index, onSelectA, onSelectB])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter') {
        handleClick()
      }
    },
    [handleClick]
  )

  const hasSemantics = !!snapshot.semantics
  const totalUs = snapshot.perf?.totalUs
  const layoutUs = snapshot.perf?.layoutUs
  const paintUs = snapshot.perf?.paintUs
  const cacheHits = snapshot.perf?.cache?.hits
  const cacheMisses = snapshot.perf?.cache?.misses
  const invalidationCount =
    snapshot.perf?.invalidations && typeof snapshot.perf.invalidations === 'object'
      ? (snapshot.perf.invalidations as { count?: number }).count
      : undefined

  return (
    <div
      className={cn(
        'flex flex-col gap-1.5 px-3 py-2 border-b border-border cursor-pointer hover:bg-accent/50 transition-colors',
        isSelectedA && 'bg-accent',
        isSelectedB && 'bg-chart-2/10 ring-2 ring-chart-2 ring-inset',
        isSelectedA && isSelectedB && 'bg-accent ring-2 ring-chart-2 ring-inset'
      )}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      tabIndex={0}
      role="option"
      aria-selected={isSelectedA || isSelectedB}
    >
      {/* Row 1: IDs and timestamp */}
      <div className="flex items-center gap-2">
        {/* Selection indicators for compare mode */}
        {compareMode && (
          <div className="flex gap-1">
            <span
              className={cn(
                'w-5 h-5 rounded flex items-center justify-center text-[10px] font-bold',
                isSelectedA ? 'bg-foreground text-background' : 'bg-muted text-muted-foreground'
              )}
            >
              A
            </span>
            <span
              className={cn(
                'w-5 h-5 rounded flex items-center justify-center text-[10px] font-bold',
                isSelectedB ? 'bg-chart-2 text-background' : 'bg-muted text-muted-foreground'
              )}
            >
              B
            </span>
          </div>
        )}

        {snapshot.tickId && (
          <Badge variant="outline" className="text-xs font-mono">
            tick:{snapshot.tickId}
          </Badge>
        )}
        {snapshot.frameId && (
          <Badge variant="outline" className="text-xs font-mono">
            frame:{snapshot.frameId}
          </Badge>
        )}
        <span className="text-xs text-muted-foreground ml-auto font-mono">
          {snapshot.timestampMonoNs ? formatTimestamp(snapshot.timestampMonoNs) : '—'}
        </span>
      </div>

      {/* Row 2: Stats - scrollable on overflow */}
      <div className="flex items-center gap-1.5 overflow-x-auto scrollbar-none pb-0.5">
        {totalUs !== undefined && (
          <Badge variant="secondary" className="text-[10px] gap-0.5 shrink-0 h-5 px-1.5">
            <Clock className="h-2.5 w-2.5" />
            {formatMicroseconds(totalUs)}
          </Badge>
        )}
        {layoutUs !== undefined && (
          <Badge variant="secondary" className="text-[10px] gap-0.5 shrink-0 h-5 px-1.5 bg-blue-500/10 text-blue-600 border-blue-500/30 dark:text-blue-400">
            <Layers className="h-2.5 w-2.5" />
            L:{formatMicroseconds(layoutUs)}
          </Badge>
        )}
        {paintUs !== undefined && (
          <Badge variant="secondary" className="text-[10px] gap-0.5 shrink-0 h-5 px-1.5 bg-green-500/10 text-green-600 border-green-500/30 dark:text-green-400">
            <Zap className="h-2.5 w-2.5" />
            P:{formatMicroseconds(paintUs)}
          </Badge>
        )}
        {invalidationCount !== undefined && (
          <Badge variant="secondary" className="text-[10px] shrink-0 h-5 px-1.5 bg-amber-500/10 text-amber-600 border-amber-500/30 dark:text-amber-400">
            inv:{invalidationCount}
          </Badge>
        )}
        {(cacheHits !== undefined || cacheMisses !== undefined) && (
          <Badge variant="secondary" className="text-[10px] gap-0.5 shrink-0 h-5 px-1.5 bg-purple-500/10 text-purple-600 border-purple-500/30 dark:text-purple-400">
            <Database className="h-2.5 w-2.5" />
            {cacheHits ?? 0}/{(cacheHits ?? 0) + (cacheMisses ?? 0)}
          </Badge>
        )}
        {!hasSemantics && (
          <Badge variant="outline" className="text-[10px] shrink-0 h-5 px-1.5 text-muted-foreground">
            no sem
          </Badge>
        )}
      </div>
    </div>
  )
}

function formatTimestamp(ns: string): string {
  try {
    const num = BigInt(ns)

    // Heuristic:
    // - Fret bundles commonly store `timestamp_unix_ms` (~1e12..1e13).
    // - Some other schemas may store monotonic/epoch nanoseconds (~1e15+).
    if (num < BigInt(1_000_000_000_000_000)) {
      const msEpoch = Number(num)
      if (!Number.isFinite(msEpoch)) return ns
      const d = new Date(msEpoch)
      const iso = d.toISOString()
      // 2026-01-24T12:34:56.789Z -> 12:34:56.789
      return iso.slice(11, 23)
    }

    const us = num / BigInt(1_000)
    const ms = Number(us) / 1000
    if (ms < 1000) return `${ms.toFixed(1)}ms`
    const s = ms / 1000
    if (s < 60) return `${s.toFixed(2)}s`
    const m = Math.floor(s / 60)
    const rem = s % 60
    return `${m}m ${rem.toFixed(1)}s`
  } catch {
    return ns
  }
}

function formatMicroseconds(us: number): string {
  if (us < 1000) return `${us}μs`
  if (us < 1_000_000) return `${(us / 1000).toFixed(1)}ms`
  return `${(us / 1_000_000).toFixed(2)}s`
}

type FilterType = 'all' | 'has_semantics' | 'slow'

export function SnapshotsPanel() {
  const bundle = useBundleStore((s) => s.bundle)
  const selectedWindowIndex = useBundleStore((s) => s.selectedWindowIndex)
  const setSelectedWindowIndex = useBundleStore((s) => s.setSelectedWindowIndex)
  const selectedSnapshotAIndex = useBundleStore((s) => s.selectedSnapshotAIndex)
  const selectedSnapshotBIndex = useBundleStore((s) => s.selectedSnapshotBIndex)
  const setSelectedSnapshotAIndex = useBundleStore((s) => s.setSelectedSnapshotAIndex)
  const setSelectedSnapshotBIndex = useBundleStore((s) => s.setSelectedSnapshotBIndex)
  const compareMode = useBundleStore((s) => s.compareMode)
  const { t } = useTranslation()

  const [filter, setFilter] = useState<FilterType>('all')
  const listRef = useRef<HTMLDivElement>(null)

  const windows = bundle?.windows ?? []
  const selectedWindow = windows[selectedWindowIndex]
  const snapshots = selectedWindow?.snapshots ?? []

  // Compute slow threshold (top 20% by total time)
  const slowThreshold = useMemo(() => {
    const times = snapshots
      .map((s) => s.perf?.totalUs)
      .filter((t): t is number => t !== undefined)
      .sort((a, b) => b - a)
    if (times.length === 0) return Infinity
    const idx = Math.max(0, Math.floor(times.length * 0.2) - 1)
    return times[idx]
  }, [snapshots])

  // Filter snapshots
  const filteredSnapshots = useMemo(() => {
    return snapshots
      .map((s, i) => ({ snapshot: s, originalIndex: i }))
      .filter(({ snapshot }) => {
        if (filter === 'has_semantics') return !!snapshot.semantics
        if (filter === 'slow') {
          const total = snapshot.perf?.totalUs
          return total !== undefined && total >= slowThreshold
        }
        return true
      })
  }, [snapshots, filter, slowThreshold])

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return

      const currentIdx = filteredSnapshots.findIndex((s) => s.originalIndex === selectedSnapshotAIndex)

      if (e.key === 'j' || e.key === 'ArrowDown') {
        e.preventDefault()
        if (currentIdx < filteredSnapshots.length - 1) {
          setSelectedSnapshotAIndex(filteredSnapshots[currentIdx + 1].originalIndex)
        }
      }
      if (e.key === 'k' || e.key === 'ArrowUp') {
        e.preventDefault()
        if (currentIdx > 0) {
          setSelectedSnapshotAIndex(filteredSnapshots[currentIdx - 1].originalIndex)
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [filteredSnapshots, selectedSnapshotAIndex, setSelectedSnapshotAIndex])

  if (!bundle) {
    return (
      <div className="flex flex-col h-full">
        <div className="flex items-center justify-between px-3 py-2 border-b border-border bg-muted/30">
          <h2 className="text-sm font-medium text-foreground">{t('snapshots.title')}</h2>
        </div>
        <div className="flex-1 flex items-center justify-center p-4">
          <p className="text-sm text-muted-foreground text-center">{t('snapshots.noBundle')}</p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header with window selector */}
      <div className="px-3 py-2 border-b border-border bg-muted/30 space-y-2">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-medium text-foreground">{t('snapshots.title')}</h2>
          <Badge variant="secondary" className="text-xs">
            {filteredSnapshots.length} {t('snapshots.of')} {snapshots.length}
          </Badge>
        </div>

        {/* Window tabs */}
        {windows.length > 1 && (
          <Tabs
            value={String(selectedWindowIndex)}
            onValueChange={(v) => setSelectedWindowIndex(Number(v))}
            className="w-full"
          >
            <TabsList className="w-full justify-start h-auto p-1 flex-wrap">
              {windows.map((w, i) => (
                <TabsTrigger
                  key={w.windowId}
                  value={String(i)}
                  className="text-xs px-2 py-1"
                >
                  {w.windowId}
                  <Badge variant="outline" className="ml-1 text-[10px] h-4">
                    {w.snapshots.length}
                  </Badge>
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        )}

        {/* Filters */}
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1.5">
            <Checkbox
              id="filter-semantics"
              checked={filter === 'has_semantics'}
              onCheckedChange={(checked) => setFilter(checked ? 'has_semantics' : 'all')}
            />
            <Label htmlFor="filter-semantics" className="text-xs cursor-pointer">
              {t('snapshots.hasSemantics')}
            </Label>
          </div>
          <div className="flex items-center gap-1.5">
            <Checkbox
              id="filter-slow"
              checked={filter === 'slow'}
              onCheckedChange={(checked) => setFilter(checked ? 'slow' : 'all')}
            />
            <Label htmlFor="filter-slow" className="text-xs cursor-pointer">
              {t('snapshots.slowFrames')}
            </Label>
          </div>
          {filter !== 'all' && (
            <Button
              variant="ghost"
              size="sm"
              className="text-xs h-6 px-2"
              onClick={() => setFilter('all')}
            >
              {t('snapshots.clearFilter')}
            </Button>
          )}
        </div>
      </div>

      {/* Snapshot list */}
      <ScrollArea className="flex-1">
        <div ref={listRef} role="listbox">
          {filteredSnapshots.length === 0 ? (
            <div className="p-4 text-center text-sm text-muted-foreground">
              {t('snapshots.noMatch')}
            </div>
          ) : (
            filteredSnapshots.map(({ snapshot, originalIndex }) => (
              <SnapshotRow
                key={originalIndex}
                snapshot={snapshot}
                index={originalIndex}
                isSelectedA={selectedSnapshotAIndex === originalIndex}
                isSelectedB={selectedSnapshotBIndex === originalIndex}
                compareMode={compareMode}
                onSelectA={setSelectedSnapshotAIndex}
                onSelectB={setSelectedSnapshotBIndex}
              />
            ))
          )}
        </div>
      </ScrollArea>

      {/* Sparkline / Mini chart (simplified bar display) */}
      {snapshots.length > 1 && (
        <div className="px-3 py-2 border-t border-border bg-muted/30">
          <div className="flex items-end gap-px h-8">
            {snapshots.slice(0, 50).map((s, i) => {
              const total = s.perf?.totalUs ?? 0
              const maxTime = Math.max(...snapshots.map((ss) => ss.perf?.totalUs ?? 0))
              const height = maxTime > 0 ? Math.max(2, (total / maxTime) * 32) : 2
              const isSelected = i === selectedSnapshotAIndex
              const isSelectedB = i === selectedSnapshotBIndex
              return (
                <button
                  key={i}
                  type="button"
                  aria-label={`Select frame ${i + 1}, ${total} microseconds`}
                  className={cn(
                    'flex-1 max-w-2 rounded-t transition-colors',
                    isSelected
                      ? 'bg-foreground'
                      : isSelectedB
                        ? 'bg-chart-2'
                        : 'bg-muted-foreground/30 hover:bg-muted-foreground/50'
                  )}
                  style={{ height: `${height}px` }}
                  onClick={() => setSelectedSnapshotAIndex(i)}
                  title={`Frame ${i + 1}: ${total}μs`}
                />
              )
            })}
          </div>
          <p className="text-[10px] text-muted-foreground mt-1 text-center">
            {t('snapshots.frameTiming', { count: Math.min(50, snapshots.length) })}
          </p>
        </div>
      )}
    </div>
  )
}
