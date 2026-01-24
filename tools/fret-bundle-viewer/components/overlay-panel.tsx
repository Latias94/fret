'use client'

import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useBundleStore } from '@/store/use-bundle-store'
import type { SemanticsNodeModel } from '@/lib/types'
import { useTranslation } from '@/hooks/use-i18n'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { cn } from '@/lib/utils'
import { Move, Maximize2 } from 'lucide-react'

type Bounds = NonNullable<SemanticsNodeModel['bounds']>

type DrawNode = {
  id: string
  bounds: Bounds
  role?: string
  testId?: string
  parentId?: string
}

type Viewport = {
  scale: number
  tx: number
  ty: number
}

function clamp(v: number, lo: number, hi: number): number {
  return Math.max(lo, Math.min(hi, v))
}

function worldSizeFromSnapshot(nodes: DrawNode[], windowSizeLogical?: { w: number; h: number }) {
  if (windowSizeLogical && Number.isFinite(windowSizeLogical.w) && Number.isFinite(windowSizeLogical.h)) {
    if (windowSizeLogical.w > 0 && windowSizeLogical.h > 0) {
      return { w: windowSizeLogical.w, h: windowSizeLogical.h }
    }
  }

  let maxX = 0
  let maxY = 0
  for (const n of nodes) {
    maxX = Math.max(maxX, n.bounds.x + n.bounds.w)
    maxY = Math.max(maxY, n.bounds.y + n.bounds.h)
  }
  if (maxX <= 0 || maxY <= 0) return { w: 1000, h: 700 }
  return { w: maxX, h: maxY }
}

function fitViewport(containerW: number, containerH: number, worldW: number, worldH: number): Viewport {
  const padding = 16
  const cw = Math.max(1, containerW - padding * 2)
  const ch = Math.max(1, containerH - padding * 2)
  const scale = clamp(Math.min(cw / Math.max(1, worldW), ch / Math.max(1, worldH)), 0.02, 50)
  const tx = (containerW - worldW * scale) / 2
  const ty = (containerH - worldH * scale) / 2
  return { scale, tx, ty }
}

function pointToWorld(view: Viewport, px: number, py: number): { x: number; y: number } {
  return { x: (px - view.tx) / view.scale, y: (py - view.ty) / view.scale }
}

function hitTest(nodes: DrawNode[], x: number, y: number): DrawNode | null {
  let best: DrawNode | null = null
  let bestArea = Infinity
  for (const n of nodes) {
    const b = n.bounds
    if (x < b.x || y < b.y || x > b.x + b.w || y > b.y + b.h) continue
    const area = Math.max(1, b.w * b.h)
    if (area < bestArea) {
      best = n
      bestArea = area
    }
  }
  return best
}

function collectSelectedPath(semantics: Record<string, DrawNode>, selectedId: string | null): Set<string> {
  const out = new Set<string>()
  if (!selectedId) return out
  let cur = semantics[selectedId]
  while (cur) {
    out.add(cur.id)
    if (!cur.parentId) break
    cur = semantics[cur.parentId]
  }
  return out
}

export function OverlayPanel() {
  const snapshot = useBundleStore((s) => s.getSelectedSnapshotA())
  const selectedNodeId = useBundleStore((s) => s.selectedNodeId)
  const setSelectedNodeId = useBundleStore((s) => s.setSelectedNodeId)
  const { t } = useTranslation()

  const [showSemantics, setShowSemantics] = useState(true)
  const [showSelectedPath, setShowSelectedPath] = useState(true)
  const [showHitChain, setShowHitChain] = useState(true)
  const [showLayerRoots, setShowLayerRoots] = useState(true)
  const [showBlocksUnderlay, setShowBlocksUnderlay] = useState(true)

  const containerRef = useRef<HTMLDivElement>(null)
  const canvasRef = useRef<HTMLCanvasElement>(null)

  const [viewport, setViewport] = useState<Viewport>({ scale: 1, tx: 0, ty: 0 })
  const viewportRef = useRef(viewport)
  useEffect(() => {
    viewportRef.current = viewport
  }, [viewport])

  const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null)
  const dragRef = useRef<{ active: boolean; lastX: number; lastY: number }>({ active: false, lastX: 0, lastY: 0 })
  const rafHoverRef = useRef<number | null>(null)

  const drawNodes = useMemo<DrawNode[]>(() => {
    const semantics = snapshot?.semantics
    if (!semantics) return []
    const out: DrawNode[] = []
    for (const n of Object.values(semantics.nodesById)) {
      if (!n.bounds) continue
      out.push({
        id: n.id,
        bounds: n.bounds,
        role: n.role,
        testId: n.testId,
        parentId: n.parentId,
      })
    }
    return out
  }, [snapshot?.semantics])

  const nodesById = useMemo<Record<string, DrawNode>>(() => {
    const out: Record<string, DrawNode> = {}
    for (const n of drawNodes) out[n.id] = n
    return out
  }, [drawNodes])

  const selectedPath = useMemo(() => {
    if (!showSelectedPath) return new Set<string>()
    return collectSelectedPath(nodesById, selectedNodeId)
  }, [nodesById, selectedNodeId, showSelectedPath])

  const hitChainIds = useMemo(() => {
    if (!showHitChain) return new Set<string>()
    const chain = snapshot?.hitTest?.chain ?? []
    const out = new Set<string>()
    for (const e of chain) {
      if (e.nodeId) out.add(e.nodeId)
    }
    return out
  }, [showHitChain, snapshot?.hitTest?.chain])

  const layerRoots = useMemo(() => {
    if (!showLayerRoots && !showBlocksUnderlay) {
      return {
        layerRootIds: new Set<string>(),
        blocksUnderlayIds: new Set<string>(),
      }
    }
    const roots = snapshot?.overlay?.layerRoots ?? []
    const layerRootIds = new Set<string>()
    const blocksUnderlayIds = new Set<string>()
    for (const r of roots) {
      if (!r.nodeId) continue
      layerRootIds.add(r.nodeId)
      if (r.blocksUnderlay) blocksUnderlayIds.add(r.nodeId)
    }
    return { layerRootIds, blocksUnderlayIds }
  }, [showLayerRoots, showBlocksUnderlay, snapshot?.overlay?.layerRoots])

  const worldSize = useMemo(() => {
    return worldSizeFromSnapshot(drawNodes, snapshot?.windowSizeLogical)
  }, [drawNodes, snapshot?.windowSizeLogical])

  const scheduleDraw = useCallback(() => {
    const canvas = canvasRef.current
    const container = containerRef.current
    if (!canvas || !container) return

    const rect = container.getBoundingClientRect()
    const cw = Math.max(1, Math.floor(rect.width))
    const ch = Math.max(1, Math.floor(rect.height))
    const dpr = window.devicePixelRatio || 1
    const targetW = Math.floor(cw * dpr)
    const targetH = Math.floor(ch * dpr)
    if (canvas.width !== targetW || canvas.height !== targetH) {
      canvas.width = targetW
      canvas.height = targetH
      canvas.style.width = `${cw}px`
      canvas.style.height = `${ch}px`
    }

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    ctx.save()
    ctx.setTransform(1, 0, 0, 1, 0, 0)
    ctx.clearRect(0, 0, canvas.width, canvas.height)
    ctx.restore()

    ctx.save()
    ctx.scale(dpr, dpr)

    const v = viewportRef.current
    ctx.translate(v.tx, v.ty)
    ctx.scale(v.scale, v.scale)

    const strokePx = 1 / Math.max(0.0001, v.scale)

    // Background bounds (world)
    ctx.strokeStyle = 'rgba(148, 163, 184, 0.35)' // slate-400
    ctx.lineWidth = strokePx
    ctx.strokeRect(0, 0, worldSize.w, worldSize.h)

    // Semantics boxes
    if (showSemantics) {
      for (const n of drawNodes) {
        const b = n.bounds
        const isSelected = n.id === selectedNodeId
        const isInSelectedPath = selectedPath.has(n.id)
        const isHitChain = hitChainIds.has(n.id)
        const isLayerRoot = layerRoots.layerRootIds.has(n.id)
        const isBlocksUnderlay = layerRoots.blocksUnderlayIds.has(n.id)

        if (isBlocksUnderlay && showBlocksUnderlay) {
          ctx.strokeStyle = 'rgba(239, 68, 68, 0.75)' // red-500
        } else if (isLayerRoot && showLayerRoots) {
          ctx.strokeStyle = 'rgba(245, 158, 11, 0.70)' // amber-500
        } else if (isHitChain) {
          ctx.strokeStyle = 'rgba(34, 197, 94, 0.65)' // green-500
        } else if (isInSelectedPath) {
          ctx.strokeStyle = 'rgba(168, 85, 247, 0.55)' // purple-500
        } else {
          ctx.strokeStyle = 'rgba(59, 130, 246, 0.25)' // blue-500
        }

        ctx.lineWidth = isSelected ? strokePx * 2.25 : strokePx
        ctx.strokeRect(b.x, b.y, b.w, b.h)
      }
    }

    // Hover highlight (screen-constant thickness)
    if (hoveredNodeId && nodesById[hoveredNodeId]) {
      const b = nodesById[hoveredNodeId]!.bounds
      ctx.strokeStyle = 'rgba(14, 165, 233, 0.95)' // sky-500
      ctx.lineWidth = strokePx * 2
      ctx.strokeRect(b.x, b.y, b.w, b.h)
    }

    ctx.restore()
  }, [
    drawNodes,
    hitChainIds,
    hoveredNodeId,
    layerRoots.blocksUnderlayIds,
    layerRoots.layerRootIds,
    nodesById,
    selectedNodeId,
    selectedPath,
    showBlocksUnderlay,
    showHitChain,
    showLayerRoots,
    showSemantics,
    worldSize.h,
    worldSize.w,
  ])

  // Re-draw when inputs change.
  useEffect(() => {
    scheduleDraw()
  }, [scheduleDraw, viewport])

  // Auto-fit when snapshot changes or when container size changes.
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const ro = new ResizeObserver(() => {
      const r = container.getBoundingClientRect()
      setViewport((prev) => {
        if (prev.scale === 1 && prev.tx === 0 && prev.ty === 0) {
          return fitViewport(r.width, r.height, worldSize.w, worldSize.h)
        }
        // Keep current viewport; draw will handle canvas resize.
        return prev
      })
      scheduleDraw()
    })
    ro.observe(container)
    return () => ro.disconnect()
  }, [scheduleDraw, worldSize.h, worldSize.w])

  useEffect(() => {
    const container = containerRef.current
    if (!container) return
    const r = container.getBoundingClientRect()
    setViewport(fitViewport(r.width, r.height, worldSize.w, worldSize.h))
  }, [snapshot?.tickId, snapshot?.frameId, worldSize.h, worldSize.w])

  const handleFit = useCallback(() => {
    const container = containerRef.current
    if (!container) return
    const r = container.getBoundingClientRect()
    setViewport(fitViewport(r.width, r.height, worldSize.w, worldSize.h))
  }, [worldSize.h, worldSize.w])

  const handleWheel = useCallback((e: React.WheelEvent<HTMLCanvasElement>) => {
    e.preventDefault()
    const container = containerRef.current
    if (!container) return
    const rect = container.getBoundingClientRect()
    const cx = e.clientX - rect.left
    const cy = e.clientY - rect.top

    const prev = viewportRef.current
    const base = e.ctrlKey ? 1.01 : 1.0025
    const factor = Math.pow(base, -e.deltaY)
    const nextScale = clamp(prev.scale * factor, 0.02, 60)
    const world = pointToWorld(prev, cx, cy)
    const nextTx = cx - world.x * nextScale
    const nextTy = cy - world.y * nextScale
    setViewport({ scale: nextScale, tx: nextTx, ty: nextTy })
  }, [])

  const handlePointerDown = useCallback((e: React.PointerEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current
    if (!canvas) return
    canvas.setPointerCapture(e.pointerId)
    dragRef.current = { active: true, lastX: e.clientX, lastY: e.clientY }
  }, [])

  const handlePointerUp = useCallback((e: React.PointerEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current
    if (!canvas) return
    dragRef.current.active = false
    canvas.releasePointerCapture(e.pointerId)
  }, [])

  const handlePointerMove = useCallback(
    (e: React.PointerEvent<HTMLCanvasElement>) => {
      const container = containerRef.current
      if (!container) return
      const rect = container.getBoundingClientRect()
      const x = e.clientX - rect.left
      const y = e.clientY - rect.top

      if (dragRef.current.active) {
        const dx = e.clientX - dragRef.current.lastX
        const dy = e.clientY - dragRef.current.lastY
        dragRef.current.lastX = e.clientX
        dragRef.current.lastY = e.clientY
        setViewport((prev) => ({ ...prev, tx: prev.tx + dx, ty: prev.ty + dy }))
      }

      if (rafHoverRef.current != null) return
      rafHoverRef.current = window.requestAnimationFrame(() => {
        rafHoverRef.current = null
        const v = viewportRef.current
        const world = pointToWorld(v, x, y)
        const hit = hitTest(drawNodes, world.x, world.y)
        setHoveredNodeId(hit?.id ?? null)
      })
    },
    [drawNodes]
  )

  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const container = containerRef.current
      if (!container) return
      const rect = container.getBoundingClientRect()
      const cx = e.clientX - rect.left
      const cy = e.clientY - rect.top
      const v = viewportRef.current
      const world = pointToWorld(v, cx, cy)
      const hit = hitTest(drawNodes, world.x, world.y)
      if (hit) setSelectedNodeId(hit.id)
    },
    [drawNodes, setSelectedNodeId]
  )

  const hoveredNode = hoveredNodeId ? nodesById[hoveredNodeId] : null

  if (!snapshot) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground">{t('overlay.noSnapshot')}</p>
      </div>
    )
  }

  if (!snapshot.semantics) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground">{t('overlay.noSemantics')}</p>
      </div>
    )
  }

  if (drawNodes.length === 0) {
    return (
      <div className="flex items-center justify-center h-full p-4">
        <p className="text-sm text-muted-foreground">{t('overlay.noBounds')}</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between gap-2 px-3 py-2 border-b border-border bg-muted/30">
        <div className="flex items-center gap-2 min-w-0">
          <h3 className="text-sm font-medium text-foreground shrink-0">{t('overlay.title')}</h3>
          <Badge variant="secondary" className="text-[10px] shrink-0">
            {drawNodes.length}
          </Badge>
        </div>

        <div className="flex items-center gap-1.5">
          <Button variant="ghost" size="sm" className="h-7 px-2 text-xs" onClick={handleFit}>
            <Maximize2 className="mr-1 h-3.5 w-3.5" />
            {t('overlay.fit')}
          </Button>
        </div>
      </div>

      <div className="px-3 py-2 border-b border-border">
        <div className="flex flex-wrap items-center gap-4">
          <div className="flex items-center gap-2">
            <Move className="h-4 w-4 text-muted-foreground" />
            <span className="text-xs text-muted-foreground">{t('overlay.hint')}</span>
          </div>

          <div className="flex flex-wrap items-center gap-3">
            <label className="flex items-center gap-2">
              <Checkbox checked={showSemantics} onCheckedChange={(v) => setShowSemantics(Boolean(v))} />
              <span className="text-xs">{t('overlay.showSemantics')}</span>
            </label>
            <label className="flex items-center gap-2">
              <Checkbox checked={showSelectedPath} onCheckedChange={(v) => setShowSelectedPath(Boolean(v))} />
              <span className="text-xs">{t('overlay.showSelectedPath')}</span>
            </label>
            <label className="flex items-center gap-2">
              <Checkbox checked={showHitChain} onCheckedChange={(v) => setShowHitChain(Boolean(v))} />
              <span className="text-xs">{t('overlay.showHitChain')}</span>
            </label>
            <label className="flex items-center gap-2">
              <Checkbox checked={showLayerRoots} onCheckedChange={(v) => setShowLayerRoots(Boolean(v))} />
              <span className="text-xs">{t('overlay.showLayerRoots')}</span>
            </label>
            <label className="flex items-center gap-2">
              <Checkbox checked={showBlocksUnderlay} onCheckedChange={(v) => setShowBlocksUnderlay(Boolean(v))} />
              <span className="text-xs">{t('overlay.showBlocksUnderlay')}</span>
            </label>
          </div>

          <div className="ml-auto flex items-center gap-2">
            <Label className="text-xs text-muted-foreground">{t('overlay.hover')}</Label>
            <span className={cn('text-xs font-mono truncate max-w-64', !hoveredNode && 'text-muted-foreground')}>
              {hoveredNode ? hoveredNode.id : t('overlay.none')}
            </span>
          </div>
        </div>
      </div>

      <div ref={containerRef} className="relative flex-1 overflow-hidden bg-background">
        <canvas
          ref={canvasRef}
          className="h-full w-full touch-none"
          onWheel={handleWheel}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerCancel={handlePointerUp}
          onClick={handleClick}
        />

        {hoveredNode && (
          <div className="absolute left-2 bottom-2 max-w-[min(32rem,calc(100%-1rem))] rounded-md border border-border bg-card/95 px-2 py-1.5 text-xs shadow-sm">
            <div className="flex items-center gap-2">
              <span className="font-mono truncate">{hoveredNode.id}</span>
              {hoveredNode.role && (
                <Badge variant="outline" className="text-[10px]">
                  {hoveredNode.role}
                </Badge>
              )}
              {hoveredNode.testId && (
                <span className="text-[10px] text-muted-foreground truncate">{hoveredNode.testId}</span>
              )}
            </div>
            <div className="mt-1 text-[10px] text-muted-foreground font-mono">
              {Math.round(hoveredNode.bounds.x)}, {Math.round(hoveredNode.bounds.y)} · {Math.round(hoveredNode.bounds.w)}×{Math.round(hoveredNode.bounds.h)}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
