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
- [ ] Introduce an explicit dispatch context struct (e.g. `DispatchCx`) carrying:
  - [ ] active input roots + barrier root
  - [ ] active focus roots + focus barrier root
  - [ ] input-scope snapshot (required)
  - [ ] focus-scope snapshot (optional / on-demand)
- [ ] Refactor `dispatch/window.rs` and `dispatch/chain.rs` to thread `DispatchCx` rather than
  ad-hoc snapshots/closures.
- [ ] Remove remaining containment queries that rely on live-tree parent walks during dispatch
  (replace with snapshot queries).
- [ ] Add conformance coverage for nested scenarios:
  - [x] trapped focus scope inside a modal overlay root (portal-style nested roots)
  - [x] multiple stacked trapped scopes (inner scope wins)
  - [x] barrier active during close transitions (focus restoration while pointer underlay stays blocked)

## Nice-to-haves (separate workstreams if they expand)

- [ ] `fretboard diag` scripted repro covering “stale parent pointers” + overlay focus trap outcomes.
- [ ] Perf probe: snapshot build cost vs frame budget in UI gallery worst-case overlays.
