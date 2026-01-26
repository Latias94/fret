// Core types for Fret Bundle Viewer

import type { TranslationKey } from './i18n'

export interface UiMessage {
  key: TranslationKey
  params?: Record<string, string | number>
  detail?: string
}

export interface SemanticsNodeModel {
  id: string
  role?: string
  label?: string
  name?: string
  testId?: string
  bounds?: { x: number; y: number; w: number; h: number }
  parentId?: string
  children: string[]
  flags?: Record<string, boolean | null>
  actions?: Record<string, boolean>
}

export interface SemanticsModel {
  roots: string[]
  nodesById: Record<string, SemanticsNodeModel>
}

export interface LayerRoot {
  nodeId?: string
  zIndex?: number
  hitTestable?: boolean
  blocksUnderlay?: boolean
}

export interface HitChainEntry {
  nodeId?: string
  role?: string
  testId?: string
  label?: string
}

export interface NormalizedEvent {
  kind: string
  summary: string
  tickId?: string
  frameId?: string
  windowId?: string
  raw?: unknown
}

export interface PerfData {
  totalUs?: number
  layoutUs?: number
  prepaintUs?: number
  paintUs?: number
  invalidations?: unknown
  cache?: {
    hits?: number
    misses?: number
  }
}

export interface SnapshotModel {
  tickId?: string
  frameId?: string
  timestampMonoNs?: string
  scaleFactor?: number
  windowSizeLogical?: { w: number; h: number }
  semantics?: SemanticsModel
  focus?: { focusedNodeId?: string; activeDescendantId?: string }
  overlay?: {
    barrierRootId?: string
    layerRoots?: LayerRoot[]
  }
  hitTest?: {
    pointer?: { x?: number; y?: number }
    chain?: HitChainEntry[]
  }
  events?: NormalizedEvent[]
  perf?: PerfData
  raw: unknown
}

export interface WindowModel {
  windowId: string
  events?: NormalizedEvent[]
  snapshots: SnapshotModel[]
}

export interface BundleMeta {
  fileName?: string
  fileSize?: number
  schemaVersion?: number
  exportedUnixMs?: number
  outDir?: string
  zip?: ZipImportMeta
}

export interface ZipArtifact {
  path: string
  fileName: string
  sizeBytes: number
  truncated?: boolean
  text: string
}

export interface ZipScreenshot {
  path: string
  fileName: string
  sizeBytes: number
  objectUrl: string
  meta?: ZipScreenshotMeta
}

export interface ZipScreenshotMeta {
  windowId?: string
  tickId?: string
  frameId?: string
  scaleFactor?: number
  widthPx?: number
  heightPx?: number
}

export interface ZipImportMeta {
  zipFileName?: string
  bundlePathInZip?: string
  artifacts?: ZipArtifact[]
  screenshots?: ZipScreenshot[]
}

export interface BundleModel {
  meta: BundleMeta
  windows: WindowModel[]
  warnings: UiMessage[]
}

export interface Selector {
  kind: 'test_id' | 'role_and_name' | 'node_id'
  id?: string
  role?: string
  name?: string
}

export interface ScriptStep {
  type: string
  target: Selector
}

export interface DiffResult {
  added: string[]
  removed: string[]
  changed: string[]
}
