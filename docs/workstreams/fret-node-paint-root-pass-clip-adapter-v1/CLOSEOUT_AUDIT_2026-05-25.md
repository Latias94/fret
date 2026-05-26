# Fret Node Paint Root Pass Clip Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It proved the immediate paint-root pass scene adapter seam without widening into cached layer
internals, edge paint routing, overlay layer routing, or public scene schema changes.

## Shipped State

- `pass_scene_adapter.rs` defines `PaintRootPassSceneCx` and named immediate pass scene operations.
- `pass_scene_retained_cx.rs` is the retained `PaintCx` binding for static group, selected-group
  overlay, and static node scene routing.
- `paint_root/immediate_pass.rs` now preserves pass order but no longer reads `cx.scene`,
  `cx.services`, or `cx.scale_factor` for static scene routing.
- `paint_root/cached_pass.rs` remains direct-scene-free at the pass-router level.
- Source-policy coverage in `ecosystem/fret-node/src/lib.rs` keeps the pass scene adapter free of
  retained lifecycle context names and scene ops, verifies immediate/cached pass-router files do not
  read retained scene sink fields directly, and verifies the retained binding owns those reads.

## Split State

The following scene-routing families remain intentionally outside this lane:

- cached static group/node layer scene replay,
- cached edge replay,
- immediate edge paint routing,
- immediate overlay layer paint routing,
- grid plan and chrome hint routing.

The next follow-on should choose one cached internal replay family. The smallest likely candidate is
cached static layer scene replay because this lane already proved pass-level scene routing should use
named operations rather than raw retained fields.

## Closeout Evidence

- `docs/workstreams/fret-node-paint-root-tail-cleanup-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/immediate_pass.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_pass.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/pass_scene_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/pass_scene_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `python3 -m json.tool docs/workstreams/fret-node-paint-root-pass-clip-adapter-v1/WORKSTREAM.json` -
  passed.
- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_pass_scene_adapter` -
  passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Residual Risks

- Cached group/node/edge internals still own cache-local scene replay and should be handled in
  smaller cache-layer seams.
- Immediate edge and overlay routing still accept retained `PaintCx` and should be split only after
  cached replay seams are isolated.
