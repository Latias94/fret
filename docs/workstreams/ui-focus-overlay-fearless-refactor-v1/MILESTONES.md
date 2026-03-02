# UI Focus + Overlay Focus Containment (Fearless Refactor v1) — Milestones

Tracking doc: `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/DESIGN.md`
TODO board: `docs/workstreams/ui-focus-overlay-fearless-refactor-v1/TODO.md`

## M1 — Focus containment correctness under retained drift (A + B)

Goal:

- FocusScope trap correctness holds even if retained `parent` pointers are temporarily stale.
- Active-layer containment (modal barrier) does not leak focus/capture to underlay.

Exit gates:

- `crates/fret-ui` tests cover Tab traversal + pointer-focus clamping + stale-parent simulation.
- No policy leakage into `crates/fret-ui` (mechanism-only).

Status: Implemented in this worktree (2026-03-01)

## M2 — Expand conformance matrix (nested overlays + transitions)

Goal: cover the most failure-prone editor-grade compositions:

- nested roots (portal-style overlays),
- stacked traps,
- close-transition focus restore while barrier remains active.

Exit gates:

- Add at least 3 focused regression tests in `crates/fret-ui` and/or shadcn recipe tests in
  `ecosystem/fret-ui-shadcn` that exercise the above sequences.

Progress (2026-03-02):

- Close-transition style focus barrier (hit-test-inert layer) is covered by:
  - `crates/fret-ui/src/tree/tests/focus_barrier_transition.rs`

## M3 — Snapshot-first dispatch (C phase)

Goal: containment during dispatch never depends on retained `parent` pointers.

Exit gates:

- Introduce and thread a single dispatch context across window and chain dispatch.
- Remove parent-walk containment checks from dispatch paths (snapshot-only).

Status: In progress (2026-03-02)

- Dispatch-time layer membership queries no longer use retained `parent` pointers in:
  - `crates/fret-ui/src/tree/dispatch/window.rs`
  - `crates/fret-ui/src/tree/dispatch/chain.rs`
- Focus traversal availability no longer depends on retained `parent` pointers:
  - snapshot membership + snapshot parent traversal: `crates/fret-ui/src/tree/ui_tree_outside_press.rs`
  - regression test: `crates/fret-ui/src/tree/tests/focus_traversal_availability.rs`
