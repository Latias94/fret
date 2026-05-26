# Fret Node Paint Root Cache Plan Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It proved the paint-root cache-plan adapter seam and split the next paint family into
`docs/workstreams/fret-node-paint-root-frame-setup-adapter-v1/`.

## Shipped State

- `cache_plan_adapter.rs` defines `PaintRootCachePlanCx`.
- `cache_plan_retained_cx.rs` is the retained `PaintCx` binding.
- `prepare_paint_root_cache_plan` no longer reads retained `PaintCx` fields directly for host,
  bounds, or scale factor.
- Source-policy coverage keeps the adapter/helper source free of retained Cx names.

## Split State

Frame setup remains intentionally outside this lane. It includes several distinct operation
families:

- cache frame begin,
- path cache diagnostics,
- bounds/viewport/render-cull calculations,
- clip scene emission,
- background paint,
- grid paint.

The next lane should audit those families before introducing a frame adapter.

## Closeout Evidence

- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cache_plan.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `cargo test -p fret-node --features compat-retained-canvas paint_root_cache_plan_adapter` -
  passed in CPA-020.
- `cargo check -p fret-node` - passed in CPA-020.
- `cargo check -p fret-node --features compat-retained-canvas` - passed in CPA-020.
- `python3 tools/check_layering.py` - passed in CPA-020.
- `python3 tools/check_workstream_catalog.py` - passed in CPA-020 and rerun for closeout.
- `git diff --check` - passed in CPA-020 and rerun for closeout.

## Residual Risks

- `paint_root/frame.rs` still takes `PaintCx` directly.
- Scene mutation remains coupled to frame setup; the follow-on should not conflate scene emission
  with bounds/viewport inputs or diagnostics.
