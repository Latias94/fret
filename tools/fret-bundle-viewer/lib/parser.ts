import { z } from 'zod'
import type {
  BundleModel,
  WindowModel,
  SnapshotModel,
  SemanticsModel,
  SemanticsNodeModel,
  NormalizedEvent,
  PerfData,
  UiMessage,
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
  parent: CoerceString.optional(),
  parent_id: CoerceString.optional(),
  parentId: CoerceString.optional(),
  children: z.array(CoerceString).optional(),
  child_ids: z.array(CoerceString).optional(),
  childIds: z.array(CoerceString).optional(),
  flags: z.record(z.union([z.boolean(), z.null()])).optional(),
  actions: z.record(z.boolean()).optional(),
})

function normalizeNode(raw: unknown, warnings: UiMessage[]): SemanticsNodeModel | null {
  const parsed = SemanticsNodeSchema.safeParse(raw)
  if (!parsed.success) {
    warnings.push({ key: 'warn.semanticsNodeParseFailed' })
    return null
  }
  const d = parsed.data
  const id = d.id ?? d.node_id ?? d.nodeId
  if (!id) {
    warnings.push({ key: 'warn.semanticsNodeMissingId' })
    return null
  }
  return {
    id,
    role: d.role,
    label: d.label,
    name: d.name,
    testId: d.test_id ?? d.testId,
    bounds: d.bounds ?? d.rect,
    parentId: d.parent ?? d.parent_id ?? d.parentId,
    children: d.children ?? d.child_ids ?? d.childIds ?? [],
    flags: d.flags,
    actions: d.actions,
  }
}

function normalizeSemantics(raw: unknown, warnings: UiMessage[]): SemanticsModel | undefined {
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

  // Some semantics exports only provide `parent` links. Build `children` lists in that case.
  {
    const anyHasChildren = Object.values(nodesById).some((n) => n.children.length > 0)
    const anyHasParent = Object.values(nodesById).some((n) => !!n.parentId)
    if (!anyHasChildren && anyHasParent) {
      for (const node of Object.values(nodesById)) {
        node.children = []
      }
      for (const node of Object.values(nodesById)) {
        if (!node.parentId) continue
        const parent = nodesById[node.parentId]
        if (!parent) continue
        parent.children.push(node.id)
      }
    }
  }

  // Find roots (nodes without parents)
  if (Array.isArray(obj.roots)) {
    for (const r of obj.roots) {
      if (r && typeof r === 'object') {
        const rr = r as Record<string, unknown>
        if (rr.root != null) {
          roots.push(String(rr.root))
          continue
        }
      }
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

function normalizeEvents(raw: unknown, warnings: UiMessage[]): NormalizedEvent[] {
  if (!Array.isArray(raw)) return []
  const events: NormalizedEvent[] = []
  for (const e of raw) {
    if (e && typeof e === 'object') {
      const obj = e as Record<string, unknown>
      const tickId =
        obj.tick_id != null ? String(obj.tick_id) : obj.tickId != null ? String(obj.tickId) : undefined
      const frameId =
        obj.frame_id != null ? String(obj.frame_id) : obj.frameId != null ? String(obj.frameId) : undefined
      const windowId =
        obj.window != null
          ? String(obj.window)
          : obj.window_id != null
            ? String(obj.window_id)
            : obj.windowId != null
              ? String(obj.windowId)
              : undefined
      const debug = typeof obj.debug === 'string' ? obj.debug : undefined
      events.push({
        kind: String(obj.kind ?? obj.type ?? obj.event_type ?? 'unknown'),
        summary: String(
          debug ?? obj.summary ?? obj.description ?? obj.message ?? JSON.stringify(e).slice(0, 100)
        ),
        tickId,
        frameId,
        windowId,
        raw: e,
      })
    }
  }
  return events
}

function normalizePerf(raw: unknown, warnings: UiMessage[]): PerfData | undefined {
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

function normalizePerfFromDebugStats(raw: unknown): PerfData | undefined {
  if (!raw || typeof raw !== 'object') return undefined
  const stats = raw as Record<string, unknown>

  const layoutUs = typeof stats.layout_time_us === 'number' ? stats.layout_time_us : undefined
  const prepaintUs = typeof stats.prepaint_time_us === 'number' ? stats.prepaint_time_us : undefined
  const paintUs = typeof stats.paint_time_us === 'number' ? stats.paint_time_us : undefined
  const totalUs =
    layoutUs !== undefined || prepaintUs !== undefined || paintUs !== undefined
      ? (layoutUs ?? 0) + (prepaintUs ?? 0) + (paintUs ?? 0)
      : undefined

  const cacheHits = typeof stats.paint_cache_hits === 'number' ? stats.paint_cache_hits : undefined
  const cacheMisses = typeof stats.paint_cache_misses === 'number' ? stats.paint_cache_misses : undefined

  const invalidationCalls =
    typeof stats.invalidation_walk_calls === 'number' ? stats.invalidation_walk_calls : undefined
  const invalidationNodes =
    typeof stats.invalidation_walk_nodes === 'number' ? stats.invalidation_walk_nodes : undefined

  return {
    totalUs,
    layoutUs,
    prepaintUs,
    paintUs,
    invalidations:
      invalidationCalls !== undefined || invalidationNodes !== undefined
        ? { count: invalidationCalls, nodes: invalidationNodes }
        : undefined,
    cache:
      cacheHits !== undefined || cacheMisses !== undefined
        ? { hits: cacheHits, misses: cacheMisses }
        : undefined,
  }
}

function normalizeSnapshot(raw: unknown, warnings: UiMessage[]): SnapshotModel {
  if (!raw || typeof raw !== 'object') {
    return { raw }
  }

  const obj = raw as Record<string, unknown>
  const debug = obj.debug && typeof obj.debug === 'object' ? (obj.debug as Record<string, unknown>) : undefined
  const debugStats = debug?.stats
  const debugSemantics = debug?.semantics

  const snapshot: SnapshotModel = {
    tickId: obj.tick_id != null ? String(obj.tick_id) : obj.tickId != null ? String(obj.tickId) : undefined,
    frameId: obj.frame_id != null ? String(obj.frame_id) : obj.frameId != null ? String(obj.frameId) : undefined,
    timestampMonoNs:
      obj.timestamp_unix_ms != null
        ? String(obj.timestamp_unix_ms)
        : obj.timestamp_mono_ns != null
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
    semantics: normalizeSemantics(debugSemantics ?? obj.semantics ?? obj.semantic_tree ?? obj.tree, warnings),
    focus: undefined,
    overlay: undefined,
    hitTest: undefined,
    events: normalizeEvents(obj.events ?? obj.recent_events ?? obj.recentEvents, warnings),
    perf:
      normalizePerfFromDebugStats(debugStats) ??
      normalizePerf(obj.perf ?? obj.performance ?? obj.timing, warnings),
    raw,
  }

  // Window size
  const size =
    obj.window_bounds ?? obj.window_size_logical ?? obj.windowSizeLogical ?? obj.window_size ?? obj.size
  if (size && typeof size === 'object') {
    const s = size as Record<string, unknown>
    snapshot.windowSizeLogical = {
      w: typeof s.w === 'number' ? s.w : typeof s.width === 'number' ? s.width : 0,
      h: typeof s.h === 'number' ? s.h : typeof s.height === 'number' ? s.height : 0,
    }
  }

  // Focus
  const focus =
    obj.focus ??
    obj.focus_state ??
    obj.focusState ??
    (debugStats && typeof debugStats === 'object' ? debugStats : undefined)
  if (focus && typeof focus === 'object') {
    const f = focus as Record<string, unknown>
    snapshot.focus = {
      focusedNodeId:
        f.focused_node_id != null
          ? String(f.focused_node_id)
          : f.focused_node != null
            ? String(f.focused_node)
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
  const barrierRootId =
    debugSemantics && typeof debugSemantics === 'object' && (debugSemantics as Record<string, unknown>).barrier_root != null
      ? String((debugSemantics as Record<string, unknown>).barrier_root)
      : overlay && typeof overlay === 'object'
        ? (() => {
            const o = overlay as Record<string, unknown>
            if (o.barrier_root_id != null) return String(o.barrier_root_id)
            if (o.barrierRootId != null) return String(o.barrierRootId)
            return undefined
          })()
        : undefined

  const layerRootsRaw =
    (debugSemantics && typeof debugSemantics === 'object' ? (debugSemantics as Record<string, unknown>).roots : undefined) ??
    (overlay && typeof overlay === 'object'
      ? (overlay as Record<string, unknown>).layer_roots ??
        (overlay as Record<string, unknown>).layerRoots ??
        (overlay as Record<string, unknown>).layers
      : undefined) ??
    debug?.layers_in_paint_order

  if (barrierRootId || Array.isArray(layerRootsRaw)) {
    snapshot.overlay = {
      barrierRootId,
      layerRoots: Array.isArray(layerRootsRaw)
        ? layerRootsRaw.map((l: unknown) => {
            if (!l || typeof l !== 'object') return {}
            const lr = l as Record<string, unknown>
            return {
              nodeId:
                lr.root != null
                  ? String(lr.root)
                  : lr.node_id != null
                    ? String(lr.node_id)
                    : lr.nodeId != null
                      ? String(lr.nodeId)
                      : undefined,
              zIndex: typeof lr.z_index === 'number' ? lr.z_index : typeof lr.zIndex === 'number' ? lr.zIndex : undefined,
              hitTestable:
                typeof lr.hit_testable === 'boolean'
                  ? lr.hit_testable
                  : typeof lr.hitTestable === 'boolean'
                    ? lr.hitTestable
                    : undefined,
              blocksUnderlay:
                typeof lr.blocks_underlay_input === 'boolean'
                  ? lr.blocks_underlay_input
                  : typeof lr.blocks_underlay === 'boolean'
                    ? lr.blocks_underlay
                    : typeof lr.blocksUnderlay === 'boolean'
                      ? lr.blocksUnderlay
                      : undefined,
            }
          })
        : undefined,
    }
  }

  // Hit test
  const hitTest = debug?.hit_test ?? obj.hit_test ?? obj.hitTest ?? obj.hit_testing
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

function normalizeWindow(raw: unknown, index: number, warnings: UiMessage[]): WindowModel {
  if (!raw || typeof raw !== 'object') {
    return { windowId: `window-${index}`, snapshots: [] }
  }

  const obj = raw as Record<string, unknown>
  const windowId =
    obj.window != null
      ? String(obj.window)
      : obj.window_id != null
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

  const events = normalizeEvents(obj.events, warnings)

  return { windowId, snapshots, events: events.length > 0 ? events : undefined }
}

export function parseBundle(jsonText: string, fileName?: string): BundleModel {
  const warnings: UiMessage[] = []
  let parsed: unknown

  try {
    parsed = JSON.parse(jsonText)
  } catch (e) {
    warnings.push({
      key: 'error.jsonParse',
      detail: e instanceof Error ? e.message : String(e),
    })
    return {
      meta: { fileName, fileSize: jsonText.length },
      windows: [],
      warnings,
    }
  }

  if (!parsed || typeof parsed !== 'object') {
    warnings.push({ key: 'error.bundleRootNotObject' })
    return {
      meta: { fileName, fileSize: jsonText.length },
      windows: [],
      warnings,
    }
  }

  const root = parsed as Record<string, unknown>
  let windows: WindowModel[] = []

  const schemaVersion = typeof root.schema_version === 'number' ? root.schema_version : undefined
  const exportedUnixMs = typeof root.exported_unix_ms === 'number' ? root.exported_unix_ms : undefined
  const outDir = typeof root.out_dir === 'string' ? root.out_dir : undefined

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
    warnings.push({ key: 'warn.cannotFindWindowsOrSnapshots' })
  }

  // Check for empty data
  if (windows.length === 0) {
    warnings.push({ key: 'warn.noWindows' })
  } else {
    const totalSnapshots = windows.reduce((sum, w) => sum + w.snapshots.length, 0)
    if (totalSnapshots === 0) {
      warnings.push({ key: 'warn.noSnapshots' })
    }
  }

  return {
    meta: { fileName, fileSize: jsonText.length, schemaVersion, exportedUnixMs, outDir },
    windows,
    warnings,
  }
}
