'use client'

import { useMemo } from 'react'
import { useBundleStore } from '@/store/use-bundle-store'
import { diffSnapshots, getDiffSummary } from '@/lib/diff'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import { useTranslation } from '@/hooks/use-i18n'
import { Plus, Minus, RefreshCw } from 'lucide-react'

export function DiffView() {
  const snapshotA = useBundleStore((s) => s.getSelectedSnapshotA())
  const snapshotB = useBundleStore((s) => s.getSelectedSnapshotB())
  const compareMode = useBundleStore((s) => s.compareMode)
  const setSelectedNodeId = useBundleStore((s) => s.setSelectedNodeId)
  const { t } = useTranslation()

  const diff = useMemo(() => {
    if (!snapshotA || !snapshotB) return null
    return diffSnapshots(snapshotA, snapshotB)
  }, [snapshotA, snapshotB])

  if (!compareMode) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground text-center">
          {t('diff.selectTwo')}
        </p>
      </div>
    )
  }

  if (!snapshotB) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground text-center">
          {t('diff.selectTwo')}
        </p>
      </div>
    )
  }

  if (!diff) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground text-center">
          {t('diff.unableCompute')}
        </p>
      </div>
    )
  }

  const nodesA = snapshotA?.semantics?.nodesById ?? {}
  const nodesB = snapshotB?.semantics?.nodesById ?? {}

  return (
    <div className="flex flex-col h-full">
      {/* Summary */}
      <div className="px-3 py-2 border-b border-border bg-muted/30">
        <div className="flex items-center gap-3">
          <Badge variant="outline" className="gap-1 bg-green-50 text-green-700 border-green-200">
            <Plus className="h-3 w-3" />
            {diff.added.length} {t('diff.added')}
          </Badge>
          <Badge variant="outline" className="gap-1 bg-red-50 text-red-700 border-red-200">
            <Minus className="h-3 w-3" />
            {diff.removed.length} {t('diff.removed')}
          </Badge>
          <Badge variant="outline" className="gap-1 bg-amber-50 text-amber-700 border-amber-200">
            <RefreshCw className="h-3 w-3" />
            {diff.changed.length} {t('diff.changed')}
          </Badge>
        </div>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-3 space-y-4">
          {/* Added nodes */}
          {diff.added.length > 0 && (
            <div>
              <h4 className="text-xs font-medium text-foreground mb-2 flex items-center gap-1.5">
                <Plus className="h-3.5 w-3.5 text-green-600" />
                {t('diff.added')}
              </h4>
              <div className="space-y-1">
                {diff.added.map((id) => {
                  const node = nodesB[id]
                  return (
                    <button
                      key={id}
                      type="button"
                      className="w-full text-left px-2 py-1.5 rounded bg-green-50 hover:bg-green-100 transition-colors"
                      onClick={() => setSelectedNodeId(id)}
                    >
                      <div className="flex items-center gap-2">
                        <Badge variant="outline" className="text-[10px]">
                          {node?.role ?? t('common.unknown')}
                        </Badge>
                        {node?.testId && (
                          <span className="text-xs text-muted-foreground">{node.testId}</span>
                        )}
                        <span className="text-xs font-mono text-muted-foreground ml-auto">{id}</span>
                      </div>
                    </button>
                  )
                })}
              </div>
            </div>
          )}

          {/* Removed nodes */}
          {diff.removed.length > 0 && (
            <div>
              <h4 className="text-xs font-medium text-foreground mb-2 flex items-center gap-1.5">
                <Minus className="h-3.5 w-3.5 text-red-600" />
                {t('diff.removed')}
              </h4>
              <div className="space-y-1">
                {diff.removed.map((id) => {
                  const node = nodesA[id]
                  return (
                    <button
                      key={id}
                      type="button"
                      className="w-full text-left px-2 py-1.5 rounded bg-red-50 hover:bg-red-100 transition-colors"
                      onClick={() => setSelectedNodeId(id)}
                    >
                      <div className="flex items-center gap-2">
                        <Badge variant="outline" className="text-[10px]">
                          {node?.role ?? t('common.unknown')}
                        </Badge>
                        {node?.testId && (
                          <span className="text-xs text-muted-foreground">{node.testId}</span>
                        )}
                        <span className="text-xs font-mono text-muted-foreground ml-auto">{id}</span>
                      </div>
                    </button>
                  )
                })}
              </div>
            </div>
          )}

          {/* Changed nodes */}
          {diff.changed.length > 0 && (
            <div>
              <h4 className="text-xs font-medium text-foreground mb-2 flex items-center gap-1.5">
                <RefreshCw className="h-3.5 w-3.5 text-amber-600" />
                {t('diff.changed')}
              </h4>
              <div className="space-y-1">
                {diff.changed.map((id) => {
                  const nodeA = nodesA[id]
                  const nodeB = nodesB[id]
                  return (
                    <button
                      key={id}
                      type="button"
                      className="w-full text-left px-2 py-1.5 rounded bg-amber-50 hover:bg-amber-100 transition-colors"
                      onClick={() => setSelectedNodeId(id)}
                    >
                      <div className="flex items-center gap-2">
                        <Badge variant="outline" className="text-[10px]">
                          {nodeB?.role ?? nodeA?.role ?? t('common.unknown')}
                        </Badge>
                        {(nodeB?.testId || nodeA?.testId) && (
                          <span className="text-xs text-muted-foreground">
                            {nodeB?.testId ?? nodeA?.testId}
                          </span>
                        )}
                        <span className="text-xs font-mono text-muted-foreground ml-auto">{id}</span>
                      </div>
                    </button>
                  )
                })}
              </div>
            </div>
          )}

          {diff.added.length === 0 && diff.removed.length === 0 && diff.changed.length === 0 && (
            <p className="text-sm text-muted-foreground text-center py-8">
              {t('diff.noChanges')}
            </p>
          )}
        </div>
      </ScrollArea>
    </div>
  )
}
