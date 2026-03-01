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

## Data model sketch

```text
UiDispatchSnapshot {
  frame_id: FrameId,
  window: AppWindowId,

  layers: Vec<SnapshotLayer> (paint order)
  barrier_root: Option<NodeId>,

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

1) Introduce snapshot struct + builder API (no behavior change).
2) Use snapshot for outside-press containment decisions (keep Phase A as fallback).
3) Use snapshot for focus containment and focus-scope trap.
4) Use snapshot for tab stop ordering and focus traversal.
5) Use snapshot for mapped event chain caching where safe.

Each PR must keep the gates in `EVIDENCE_AND_GATES.md` green.

## Performance notes

- Make snapshot incrementally buildable by recording data during existing traversals.
- Add a micro-timer span for snapshot build cost and keep a worst-frame budget.

