# UI Focus + Overlay Focus Containment (Fearless Refactor v1)

Status: Draft (workstream note)

This workstream hardens `crates/fret-ui` focus/overlay containment so that:

- Modal barriers never leak focus/capture to the underlay.
- Trapped focus scopes never allow focus to leave the trapped subtree (Tab traversal and pointer-focus).
- Correctness does not depend on retained `parent` pointers being perfect at all times.

This is **mechanism-only** work. It must not introduce Radix/shadcn policy knobs into `fret-ui`
(see ADR 0066 and ADR 0067).

## Background / why this exists

In the retained prototype (`UiTree`), some code paths relied on `Node::parent` and `is_descendant(...)`
to answer questions like:

- “Is node X inside the currently-active modal barrier scope?”
- “Is the current focus inside the trapped FocusScope?”

However, retained/view-cache reuse can temporarily create a state where:

- `child` edges remain correct (the subtree is still reachable from active roots),
- but one or more `parent` pointers are stale or temporarily `None`.

When that happens, parent-walk containment checks can incorrectly conclude that a node is *outside*
the active scope and allow focus to escape, or incorrectly clear focus/capture.

The key design choice in this workstream is to treat **reachability via child edges** (from the
active layer roots) as the authoritative membership signal during dispatch.

## Non-goals

- Implementing overlay dismissal/focus restore policy in `fret-ui` (Radix outcomes live in
  `ecosystem/fret-ui-kit` / `ecosystem/fret-ui-shadcn`). See ADR 0067.
- Designing a new public component API for FocusScope (policy should remain component-owned). See
  ADR 0068.
- Changing the long-term authoring model (declarative element tree rebuild each frame) beyond
  documenting the “C phase” direction.

## Current state (A + B phase landed)

The runtime now enforces focus containment using a dispatch-time snapshot forest derived from
**child edges**:

- Active-layer membership checks use `UiDispatchSnapshot::pre` (reachability set).
- Descendant checks use the snapshot forest when both nodes are within the snapshot.
- Fallback logic still exists (reachability via children) when a snapshot is not available.

Evidence anchors:

- Focus request gating: `crates/fret-ui/src/tree/dispatch/focus.rs`
- Dispatch context threading: `crates/fret-ui/src/tree/dispatch/ctx.rs`
- Window + chain dispatch snapshot usage: `crates/fret-ui/src/tree/dispatch/window.rs`,
  `crates/fret-ui/src/tree/dispatch/chain.rs`, `crates/fret-ui/src/tree/commands.rs`
- Dispatch-time event chain construction uses snapshot parent traversal (pointer mapping + cursor queries):
  `crates/fret-ui/src/tree/dispatch/event_chain.rs`
- Key dispatch capture/bubble chains use focus snapshots (no retained-parent walks):
  `crates/fret-ui/src/tree/dispatch/window.rs`
- Focus traversal snapshot membership + snapshot parent traversal:
  `crates/fret-ui/src/tree/ui_tree_outside_press.rs`
- Regression tests (trap + stale parent pointers): `crates/fret-ui/src/tree/tests/focus_scope.rs`
- Regression test (cursor icon query under stale parent pointers):
  `crates/fret-ui/src/tree/tests/cursor_icon_query.rs`
- Regression test (key dispatch under stale parent pointers):
  `crates/fret-ui/src/tree/tests/dispatch_phase.rs`
- Regression test (focus traversal availability under stale parent pointers):
  `crates/fret-ui/src/tree/tests/focus_traversal_availability.rs`
- Regression test (hit-test-inert focus barrier layer):
  `crates/fret-ui/src/tree/tests/focus_barrier_transition.rs`
- Regression tests (layer-root focus scopes + stacked traps):
  `crates/fret-ui/src/tree/tests/focus_scope_layered.rs`

## Plan: A + B → C (snapshot-first dispatch)

### A — “Correctness first” patches (minimal churn)

Goal: eliminate parent-pointer dependence for scope containment decisions.

- Replace `node_in_any_layer(...)` / parent-walk checks with reachability via child edges.
- Add tests that explicitly simulate stale parent pointers while keeping child edges correct.
- Keep APIs internal (`pub(in crate::tree)` / `pub(crate)`).

### B — Dispatch snapshot as the containment source of truth

Goal: ensure containment answers are consistent *within a dispatch*.

- Build one `UiDispatchSnapshot` for the active input layers per dispatch.
- Pass the snapshot into focus request gating so “active scope membership” is evaluated against the
  same forest used by routing/hit testing.
- Avoid “check against live tree mid-mutation” pitfalls.

### C — Snapshot-first architecture (fearless refactor target)

Goal: make “dispatch correctness” independent from the retained data structure details.

Proposed shape:

1. Introduce a `DispatchCx` (or similar) that is constructed once per dispatch (or once per frame)
   and contains:
   - active input layer roots + barrier root,
   - active focus layer roots + focus barrier root,
   - `UiDispatchSnapshot` for input scope and (optional) a second snapshot for focus scope,
   - cached queries needed by dispatch (e.g. fast membership checks).
2. Remove containment decisions that consult `Node::parent` during dispatch:
   - focus gating (`focus_request_is_allowed`),
   - barrier scope enforcement (focus/capture cleanup),
   - focus traversal trapping.
3. Make “parent pointers” an optimization only:
   - they may exist, but they are not allowed to be required for correctness.
4. Converge on “build a forest from child edges” as the canonical graph model for dispatch, which
   matches how DOM / Compose / Flutter treat focus scope graphs: focus containment is expressed
   over the attached tree, not over a best-effort parent cache.

Performance notes:

- Snapshots are built only from **active roots** (modal-aware), not the entire retained tree.
- The snapshot construction is O(nodes in active roots). This is intended to be acceptable in the
  current retained prototype, and becomes even more natural in the long-term “rebuild per frame”
  declarative model.

## Risks and mitigations

- Risk: snapshot building overhead in hot dispatch paths.
  - Mitigation (C): build once per dispatch/frame and reuse across all per-event operations.
  - Mitigation (A/B): keep snapshot scope limited to active roots.
- Risk: accidental policy leakage (Radix behavior knobs) into `fret-ui`.
  - Mitigation: keep the work scoped to containment *mechanisms* only; all policy remains in
    ecosystem crates per ADR 0066 / ADR 0067.

## Related contracts / workstreams

- ADR 0066: `fret-ui` runtime contract surface + modal barrier gates
- ADR 0067: overlay policy architecture (dismissal/focus/portal in component layer)
- ADR 0068: focus traversal and focus scopes
- Overlay + input arbitration v2 progress: `docs/workstreams/overlay-input-arbitration-v2.md`
