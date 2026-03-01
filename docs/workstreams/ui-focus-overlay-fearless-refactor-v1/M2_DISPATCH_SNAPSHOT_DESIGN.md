# M2: Dispatch Snapshot Design (Phase C)

This document expands Phase C into a landable design: what to build, where to build it, and how to
migrate without regressing the Phase A/B invariants.

## Motivation

Retained + view-cache reuse means “structural truth” and “routing truth” can diverge briefly. The
runtime should not depend on long-lived parent pointers for correctness-critical routing:

- outside-press containment
- focus containment and focus traversal
- tab stop computation
- hit-test and event chain mapping
- input handler (IME/text) routing

A GPUI-like “per-frame snapshot” anchors these decisions to a coherent view of the world for the
current frame/tick.

Reference anchor (GPUI): `repo-ref/zed/crates/gpui/src/window.rs` (`Frame { dispatch_tree, hitboxes, tab_stops, focus, input_handlers, ... }`).

## Terminology

- **Frame**: a render/layout frame (runner-provided `FrameId`).
- **Tick**: a discrete input dispatch/update step (runtime `TickId`).
- **Snapshot**: a per-window structure built from a coherent view of the UI tree for a single
  `FrameId` (and used for any dispatch/tick that runs against that frame’s structure).

## Snapshot scope

The snapshot is a per-window, per-frame structure built from the UI tree’s active layers.

**Must include:**

- `layer_roots` in paint order (with barrier metadata)
- `hit_test_index` (bounds + transform mapping + occlusion behavior)
- `dispatch_chains` for routed events (capture/preview/bubble, plus mapped events)
- `focus_graph`:
  - focusable nodes
  - focus scopes and trap boundaries
  - “contains focused” relationships
- `tab_stops`:
  - ordered tab stops (including stable tie-breakers)
  - per-scope traversal slices
- `text_input_bindings`:
  - focused text input kind
  - platform query routing table (bounds-for-range, replace-in-range, etc.)

## Where to build it

Build during phases that already discover:

- transforms / mapped event positions
- hit-test ranges
- focusable flags and traversal boundaries

The goal is not to add a new full-tree traversal; reuse existing traversals and record what is
needed.

## Key invariant (why this works)

Even if parent pointers in the retained structure are temporarily stale, the runtime can still
produce a coherent *snapshot forest* by walking **child edges** starting from authoritative roots
(layer roots + barrier root scoping). Snapshot containment queries must only depend on this snapshot
forest (not on retained parent pointers).

## Data model sketch

```text
UiDispatchSnapshot {
  frame_id: FrameId,
  window: AppWindowId,

  layers: Vec<SnapshotLayer> (paint order)
  barrier_root: Option<NodeId>,

  // Snapshot forest (built from child edges)
  parent: SecondaryMap<NodeId, Option<NodeId>>,
  pre: SecondaryMap<NodeId, u32>,
  post: SecondaryMap<NodeId, u32>,

  // Hit test
  hit_index: HitIndex,
  node_to_layer: SecondaryMap<NodeId, UiLayerId>,

  // Dispatch
  mapped_event_cache: MappedEventCache,

  // Focus & traversal
  focusable: BitSet<NodeId>,
  focus_scope_root_for: SecondaryMap<NodeId, Option<NodeId>>,
  trapped_scope_for: SecondaryMap<NodeId, Option<NodeId>>,
  tab_stops: Vec<NodeId>,

  // Text input
  text_input_kind: Option<TextInputKind>,
  text_input_handlers: Vec<TextInputHandlerBinding>,
}
```

This remains “mechanism only”: higher-level policies (Radix/shadcn) stay in `ecosystem/*`.

### Containment query API (descendant checks)

Once `pre`/`post` are assigned by a DFS over the snapshot forest, containment is O(1):

- `is_descendant(root, node) := pre[root] <= pre[node] && post[node] <= post[root]`

This replaces today’s mixture of parent-pointer walks and ad-hoc reachability scans.

## How Phase A/B map into the snapshot

Phase A/B establish behavior invariants that Phase C must preserve:

- Outside-press containment uses child-edge reachability as the baseline truth.
- `prevent_default` can suppress default side effects (focus clearing).

The snapshot becomes the authoritative source for:

- “node is inside layer subtree”
- “node is inside branch subtree”
- “node is inside trapped focus scope”

If a snapshot cannot be built for a frame, fall back to Phase A reachability.

## Migration plan (landable PRs)

Phase C needs to remain landable. The recommended decomposition is 5 PRs:

1) **PR0 (types + plumbing, no behavior change)**
   - Add `UiDispatchSnapshot` types and a builder entrypoint for a single window/frame.
   - Store the snapshot behind a debug flag (or `debug_enabled`) without consuming it.
   - Gate: build + existing tests.

2) **PR1 (build snapshot forest + parity diagnostics)**
   - Build the snapshot forest from child edges starting at active layer roots.
   - Compute `pre`/`post` indices.
   - Add a debug report that compares:
     - Phase A reachability (“reachable from any root via children”) vs
     - snapshot forest membership/containment,
     and records divergences for triage.
   - Gate: Phase A/B targeted tests + layering.

3) **PR2 (outside-press uses snapshot, Phase A fallback)**
   - Use snapshot containment for:
     - “inside layer subtree”
     - “inside branch subtree”
   - Keep the Phase A reachability implementation as a fallback path and as a cross-check in debug.
   - Gate: outside-press tests + dismissible tests.

4) **PR3 (focus containment + focus scopes use snapshot)**
   - Replace focus containment checks with snapshot-based descendant checks.
   - Keep behavior consistent with ADR 0068 / existing focus-scope tests.
   - Gate: focus-scope tests + Phase A/B invariants.

5) **PR4 (tab traversal uses snapshot)**
   - Move tab-stop collection and traversal to snapshot ordering.
   - Ensure stable tie-breakers and deterministic traversal under overlays.
   - Gate: traversal tests + any existing shadcn/menu focus traversal parity tests.

Each PR must keep the gates in `EVIDENCE_AND_GATES.md` green.

## Performance notes

- Make snapshot incrementally buildable by recording data during existing traversals.
- Add a micro-timer span for snapshot build cost and keep a worst-frame budget.

## Non-goals (Phase C)

- Do not move policy decisions (Radix/shadcn) into `crates/fret-ui`.
- Do not require a fully declarative “rebuild every frame” model; this snapshot is compatible with
  the current retained/runtime shape and simply tightens correctness.
