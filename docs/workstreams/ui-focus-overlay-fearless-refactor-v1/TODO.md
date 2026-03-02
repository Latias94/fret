# UI Focus + Overlay Focus Containment (Fearless Refactor v1) — TODO

Tracking doc: `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/DESIGN.md`

## A + B (landed in this worktree)

- [x] Gate focus requests against active-layer reachability (child-edge authoritative).
- [x] Use `UiDispatchSnapshot` to make focus containment robust under stale retained `parent` pointers.
- [x] Add regression tests for:
  - [x] Tab traversal trapping inside `FocusScopeProps { trap_focus: true }`
  - [x] Pointer-focus clamping when a trapped scope is active
  - [x] Stale parent-pointer simulation (child edges still correct)

## C (snapshot-first dispatch) — follow-ups

- [x] Remove `node_in_any_layer(...)` membership checks from dispatch paths (use per-dispatch snapshots).
- [x] Make focus traversal focusable collection resilient to stale retained `parent` pointers
  (snapshot membership + snapshot parent traversal).
- [x] Introduce an explicit dispatch context struct (`DispatchCx`) carrying:
  - [x] active input roots + barrier root
  - [x] active focus roots + focus barrier root
  - [x] input-scope snapshot
  - [x] focus-scope snapshot (eager for now; can be made lazy later)
- [x] Refactor `dispatch/window.rs` and `dispatch/chain.rs` to thread `DispatchCx` rather than
  ad-hoc snapshots/closures.
- [x] Avoid retained-parent fallbacks for trapped focus scope detection when a dispatch snapshot is available.
- [x] Remove remaining containment queries that rely on live-tree parent walks during dispatch
  (replace with snapshot queries).
  - Evidence: event chain construction uses dispatch snapshots for ancestor traversal in
    `crates/fret-ui/src/tree/dispatch/event_chain.rs` (threaded from `dispatch/window.rs` and
    `dispatch/chain.rs`).
- [x] Make hover ancestor queries resilient to stale retained `parent` pointers (Pressable/HoverRegion).
- [x] Make HoverRegion “hover edge” transitions disable view-cache reuse for the containing cache
  roots (rerender-on-hover-edge), so hover-driven overlays cannot get stuck behind cache hits.
- [x] Add conformance coverage for nested scenarios:
  - [x] trapped focus scope inside a modal overlay root (portal-style nested roots)
  - [x] multiple stacked trapped scopes (inner scope wins)
  - [x] barrier active during close transitions (focus restoration while pointer underlay stays blocked)

## Nice-to-haves (separate workstreams if they expand)

- [x] `fretboard diag` scripted repro covering overlay focus trap + hover/cursor outcomes.
  - Script: `tools/diag-scripts/ui-gallery/overlay/ui-gallery-overlay-focus-trap-hover-cursor.json`
- [x] Minimal hovercard-open scripted gate under view-cache reuse (hover edge should rerender the
  relevant cache root).
  - Script: `tools/diag-scripts/ui-gallery/overlay/ui-gallery-hovercard-open.json`
- [x] Perf probe: snapshot build cost vs frame budget in UI gallery worst-case overlays.
  - Script: `tools/diag-scripts/ui-gallery/perf/ui-gallery-overlay-pointer-move-steady.json`
  - Suite membership: `tools/diag-scripts/suites/perf-ui-gallery-steady/ui-gallery-overlay-pointer-move-steady.json`
