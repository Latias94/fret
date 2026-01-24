import { z } from 'zod'
import type {
  BundleModel,
  WindowModel,
  SnapshotModel,
  SemanticsModel,
  SemanticsNodeModel,
  NormalizedEvent,
  PerfData,
} from './types'

// Zod schemas for best-effort parsing
const CoerceString = z.union([z.string(), z.number()]).transform((v) => String(v))

const BoundsSchema = z
  .object({
    x: z.number().optional(),
    y: z.number().optional(),
    w: z.number().optional(),
    width: z.number().optional(),
    h: z.number().optional(),
    height: z.number().optional(),
  })
  .transform((b) => ({
    x: b.x ?? 0,
    y: b.y ?? 0,
    w: b.w ?? b.width ?? 0,
    h: b.h ?? b.height ?? 0,
  }))

const SemanticsNodeSchema = z.object({
  id: CoerceString.optional(),
  node_id: CoerceString.optional(),
  nodeId: CoerceString.optional(),
  role: z.string().optional(),
  label: z.string().optional(),
  name: z.string().optional(),
  test_id: z.string().optional(),
  testId: z.string().optional(),
  bounds: BoundsSchema.optional(),
  rect: BoundsSchema.optional(),
  parent_id: CoerceString.optional(),
  parentId: CoerceString.optional(),
  children: z.array(CoerceString).optional(),
  child_ids: z.array(CoerceString).optional(),
  childIds: z.array(CoerceString).optional(),
  flags: z.record(z.boolean()).optional(),
  actions: z.record(z.boolean()).optional(),
})

function normalizeNode(raw: unknown, warnings: string[]): SemanticsNodeModel | null {
  const parsed = SemanticsNodeSchema.safeParse(raw)
  if (!parsed.success) {
    warnings.push(`Failed to parse semantics node: ${parsed.error.message}`)
    return null
  }
  const d = parsed.data
  const id = d.id ?? d.node_id ?? d.nodeId
  if (!id) {
    warnings.push('Semantics node missing id')
    return null
  }
  return {
    id,
    role: d.role,
    label: d.label,
    name: d.name,
    testId: d.test_id ?? d.testId,
    bounds: d.bounds ?? d.rect,
    parentId: d.parent_id ?? d.parentId,
    children: d.children ?? d.child_ids ?? d.childIds ?? [],
    flags: d.flags,
    actions: d.actions,
  }
}

function normalizeSemantics(raw: unknown, warnings: string[]): SemanticsModel | undefined {
  if (!raw || typeof raw !== 'object') return undefined

  const obj = raw as Record<string, unknown>
  const nodesById: Record<string, SemanticsNodeModel> = {}
  const roots: string[] = []

  // Handle nodes as array or object
  let nodesArray: unknown[] = []
  if (Array.isArray(obj.nodes)) {
    nodesArray = obj.nodes
  } else if (obj.nodes && typeof obj.nodes === 'object') {
    nodesArray = Object.values(obj.nodes)
  } else if (Array.isArray(obj.tree)) {
    nodesArray = obj.tree
  } else if (Array.isArray(raw)) {
    nodesArray = raw
  }

  for (const nodeRaw of nodesArray) {
    const node = normalizeNode(nodeRaw, warnings)
    if (node) {
      nodesById[node.id] = node
    }
  }

  // Infer parent relationships from children
  for (const node of Object.values(nodesById)) {
    for (const childId of node.children) {
      const child = nodesById[childId]
      if (child && !child.parentId) {
        child.parentId = node.id
      }
    }
  }

  // Find roots (nodes without parents)
  if (Array.isArray(obj.roots)) {
    for (const r of obj.roots) {
      roots.push(String(r))
    }
  } else if (obj.root_id || obj.rootId) {
    roots.push(String(obj.root_id ?? obj.rootId))
  } else {
    for (const node of Object.values(nodesById)) {
      if (!node.parentId) {
        roots.push(node.id)
      }
    }
  }

  if (Object.keys(nodesById).length === 0) {
    return undefined
  }

  return { roots, nodesById }
}

function normalizeEvents(raw: unknown, warnings: string[]): NormalizedEvent[] {
  if (!Array.isArray(raw)) return []
  const events: NormalizedEvent[] = []
  for (const e of raw) {
    if (e && typeof e === 'object') {
      const obj = e as Record<string, unknown>
      events.push({
        kind: String(obj.kind ?? obj.type ?? obj.event_type ?? 'unknown'),
        summary: String(obj.summary ?? obj.description ?? obj.message ?? JSON.stringify(e).slice(0, 100)),
        raw: e,
      })
    }
  }
  return events
}

function normalizePerf(raw: unknown, warnings: string[]): PerfData | undefined {
  if (!raw || typeof raw !== 'object') return undefined
  const obj = raw as Record<string, unknown>
  return {
    totalUs: typeof obj.total_us === 'number' ? obj.total_us : typeof obj.totalUs === 'number' ? obj.totalUs : undefined,
    layoutUs:
      typeof obj.layout_us === 'number' ? obj.layout_us : typeof obj.layoutUs === 'number' ? obj.layoutUs : undefined,
    prepaintUs:
      typeof obj.prepaint_us === 'number'
        ? obj.prepaint_us
        : typeof obj.prepaintUs === 'number'
          ? obj.prepaintUs
          : undefined,
    paintUs:
      typeof obj.paint_us === 'number' ? obj.paint_us : typeof obj.paintUs === 'number' ? obj.paintUs : undefined,
    invalidations: obj.invalidations,
    cache:
      obj.cache && typeof obj.cache === 'object'
        ? {
            hits: typeof (obj.cache as Record<string, unknown>).hits === 'number' ? (obj.cache as Record<string, unknown>).hits as number : undefined,
            misses: typeof (obj.cache as Record<string, unknown>).misses === 'number' ? (obj.cache as Record<string, unknown>).misses as number : undefined,
          }
        : undefined,
  }
}

function normalizeSnapshot(raw: unknown, warnings: string[]): SnapshotModel {
  if (!raw || typeof raw !== 'object') {
    return { raw }
  }

  const obj = raw as Record<string, unknown>

  const snapshot: SnapshotModel = {
    tickId: obj.tick_id != null ? String(obj.tick_id) : obj.tickId != null ? String(obj.tickId) : undefined,
    frameId: obj.frame_id != null ? String(obj.frame_id) : obj.frameId != null ? String(obj.frameId) : undefined,
    timestampMonoNs:
      obj.timestamp_mono_ns != null
        ? String(obj.timestamp_mono_ns)
        : obj.timestampMonoNs != null
          ? String(obj.timestampMonoNs)
          : obj.timestamp != null
            ? String(obj.timestamp)
            : undefined,
    scaleFactor:
      typeof obj.scale_factor === 'number'
        ? obj.scale_factor
        : typeof obj.scaleFactor === 'number'
          ? obj.scaleFactor
          : undefined,
    windowSizeLogical: undefined,
    semantics: normalizeSemantics(obj.semantics ?? obj.semantic_tree ?? obj.tree, warnings),
    focus: undefined,
    overlay: undefined,
    hitTest: undefined,
    events: normalizeEvents(obj.events ?? obj.recent_events ?? obj.recentEvents, warnings),
    perf: normalizePerf(obj.perf ?? obj.performance ?? obj.timing, warnings),
    raw,
  }

  // Window size
  const size = obj.window_size_logical ?? obj.windowSizeLogical ?? obj.window_size ?? obj.size
  if (size && typeof size === 'object') {
    const s = size as Record<string, unknown>
    snapshot.windowSizeLogical = {
      w: typeof s.w === 'number' ? s.w : typeof s.width === 'number' ? s.width : 0,
      h: typeof s.h === 'number' ? s.h : typeof s.height === 'number' ? s.height : 0,
    }
  }

  // Focus
  const focus = obj.focus ?? obj.focus_state ?? obj.focusState
  if (focus && typeof focus === 'object') {
    const f = focus as Record<string, unknown>
    snapshot.focus = {
      focusedNodeId:
        f.focused_node_id != null
          ? String(f.focused_node_id)
          : f.focusedNodeId != null
            ? String(f.focusedNodeId)
            : f.focused != null
              ? String(f.focused)
              : undefined,
      activeDescendantId:
        f.active_descendant_id != null
          ? String(f.active_descendant_id)
          : f.activeDescendantId != null
            ? String(f.activeDescendantId)
            : undefined,
    }
  }

  // Overlay
  const overlay = obj.overlay ?? obj.overlay_routing ?? obj.overlayRouting
  if (overlay && typeof overlay === 'object') {
    const o = overlay as Record<string, unknown>
    const layerRoots = o.layer_roots ?? o.layerRoots ?? o.layers
    snapshot.overlay = {
      barrierRootId:
        o.barrier_root_id != null
          ? String(o.barrier_root_id)
          : o.barrierRootId != null
            ? String(o.barrierRootId)
            : undefined,
      layerRoots: Array.isArray(layerRoots)
        ? layerRoots.map((l: unknown) => {
            if (l && typeof l === 'object') {
              const lr = l as Record<string, unknown>
              return {
                nodeId: lr.node_id != null ? String(lr.node_id) : lr.nodeId != null ? String(lr.nodeId) : undefined,
                zIndex: typeof lr.z_index === 'number' ? lr.z_index : typeof lr.zIndex === 'number' ? lr.zIndex : undefined,
                hitTestable: typeof lr.hit_testable === 'boolean' ? lr.hit_testable : typeof lr.hitTestable === 'boolean' ? lr.hitTestable : undefined,
                blocksUnderlay: typeof lr.blocks_underlay === 'boolean' ? lr.blocks_underlay : typeof lr.blocksUnderlay === 'boolean' ? lr.blocksUnderlay : undefined,
              }
            }
            return {}
          })
        : undefined,
    }
  }

  // Hit test
  const hitTest = obj.hit_test ?? obj.hitTest ?? obj.hit_testing
  if (hitTest && typeof hitTest === 'object') {
    const ht = hitTest as Record<string, unknown>
    const pointer = ht.pointer ?? ht.last_pointer ?? ht.lastPointer
    const chain = ht.chain ?? ht.hit_chain ?? ht.hitChain
    snapshot.hitTest = {
      pointer:
        pointer && typeof pointer === 'object'
          ? {
              x: typeof (pointer as Record<string, unknown>).x === 'number' ? (pointer as Record<string, unknown>).x as number : undefined,
              y: typeof (pointer as Record<string, unknown>).y === 'number' ? (pointer as Record<string, unknown>).y as number : undefined,
            }
          : undefined,
      chain: Array.isArray(chain)
        ? chain.map((c: unknown) => {
            if (c && typeof c === 'object') {
              const ch = c as Record<string, unknown>
              return {
                nodeId: ch.node_id != null ? String(ch.node_id) : ch.nodeId != null ? String(ch.nodeId) : undefined,
                role: typeof ch.role === 'string' ? ch.role : undefined,
                testId: typeof ch.test_id === 'string' ? ch.test_id : typeof ch.testId === 'string' ? ch.testId : undefined,
                label: typeof ch.label === 'string' ? ch.label : undefined,
              }
            }
            return {}
          })
        : undefined,
    }
  }

  return snapshot
}

function normalizeWindow(raw: unknown, index: number, warnings: string[]): WindowModel {
  if (!raw || typeof raw !== 'object') {
    return { windowId: `window-${index}`, snapshots: [] }
  }

  const obj = raw as Record<string, unknown>
  const windowId =
    obj.window_id != null
      ? String(obj.window_id)
      : obj.windowId != null
        ? String(obj.windowId)
        : obj.id != null
          ? String(obj.id)
          : `window-${index}`

  let snapshots: SnapshotModel[] = []
  const snapshotsRaw = obj.snapshots ?? obj.frames ?? obj.history
  if (Array.isArray(snapshotsRaw)) {
    snapshots = snapshotsRaw.map((s) => normalizeSnapshot(s, warnings))
  } else if (snapshotsRaw && typeof snapshotsRaw === 'object') {
    // Single snapshot as object
    snapshots = [normalizeSnapshot(snapshotsRaw, warnings)]
  }

  return { windowId, snapshots }
}

export function parseBundle(jsonText: string, fileName?: string): BundleModel {
  const warnings: string[] = []
  let parsed: unknown

  try {
    parsed = JSON.parse(jsonText)
  } catch (e) {
    warnings.push(`JSON parse error: ${e instanceof Error ? e.message : 'Unknown error'}`)
    return {
      meta: { fileName, fileSize: jsonText.length },
      windows: [],
      warnings,
    }
  }

  if (!parsed || typeof parsed !== 'object') {
    warnings.push('Bundle root is not an object')
    return {
      meta: { fileName, fileSize: jsonText.length },
      windows: [],
      warnings,
    }
  }

  const root = parsed as Record<string, unknown>
  let windows: WindowModel[] = []

  // Try to find windows array
  const windowsRaw = root.windows ?? root.window_list ?? root.windowList
  if (Array.isArray(windowsRaw)) {
    windows = windowsRaw.map((w, i) => normalizeWindow(w, i, warnings))
  } else if (root.snapshots || root.frames || root.history) {
    // Single window case - the root contains snapshots directly
    windows = [normalizeWindow(root, 0, warnings)]
  } else if (root.window_id || root.windowId || root.semantics) {
    // Root is a single snapshot
    windows = [
      {
        windowId: 'window-0',
        snapshots: [normalizeSnapshot(root, warnings)],
      },
    ]
  } else {
    warnings.push('Could not find windows or snapshots in bundle')
  }

  // Check for empty data
  if (windows.length === 0) {
    warnings.push('No windows found in bundle')
  } else {
    const totalSnapshots = windows.reduce((sum, w) => sum + w.snapshots.length, 0)
    if (totalSnapshots === 0) {
      warnings.push('No snapshots found in any window')
    }
  }

  return {
    meta: { fileName, fileSize: jsonText.length },
    windows,
    warnings,
  }
}
