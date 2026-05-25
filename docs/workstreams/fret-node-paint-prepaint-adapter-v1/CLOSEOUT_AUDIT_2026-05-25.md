# Fret Node Paint Prepaint Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It proved one retained-agnostic prepaint lifecycle seam and audited paint root scope. Remaining
paint-root work is intentionally split into a narrower follow-on:
`docs/workstreams/fret-node-paint-root-cache-plan-adapter-v1/`.

## Shipped State

- `prepaint_cull_window_adapter.rs` owns cull-window view-state sync, bounds access, key
  calculation, and debug-record dispatch behind `PrepaintCullWindowCx`.
- `retained_widget_cull_window.rs` binds `PrepaintCx` explicitly and forwards to the adapter.
- `retained_widget_cull_window_shift.rs` records key-shift debug output through the adapter seam.
- Source-policy coverage in `ecosystem/fret-node/src/lib.rs` keeps adapter helpers free of retained
  Cx names.

## Split State

NPA-030 found that `canvas.paint_root(cx)` is not a single adapter seam. It crosses observation,
view-state sync, frame setup, cache planning, static layer replay/store, cached/immediate scene
emission, and tail cleanup.

The next narrow paint proof should target cache-plan preparation only:

- host access,
- bounds,
- scale factor,
- derived output publication,
- static-cache plan keys.

## Closeout Evidence

- `docs/workstreams/fret-node-paint-prepaint-adapter-v1/PAINT_ROOT_SCOPE_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/prepaint_cull_window_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_cull_window_shift.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `cargo test -p fret-node --features compat-retained-canvas paint_prepaint_adapter` - passed in
  NPA-030.
- `cargo check -p fret-node` - passed in NPA-030.
- `cargo check -p fret-node --features compat-retained-canvas` - passed in NPA-030.
- `python3 tools/check_layering.py` - passed in NPA-030.
- `python3 tools/check_workstream_catalog.py` - passed in NPA-030 and rerun for closeout.
- `git diff --check` - passed in NPA-030 and rerun for closeout.

## Residual Risks

- The retained paint root still takes `PaintCx` across many modules. This is known and explicitly
  moved to the cache-plan follow-on.
- Frame setup and scene emission remain coupled to retained paint context; do not widen the
  cache-plan follow-on to include them unless a later audit splits a dedicated task.
