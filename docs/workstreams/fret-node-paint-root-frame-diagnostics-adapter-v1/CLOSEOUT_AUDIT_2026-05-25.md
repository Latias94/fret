# Fret Node Paint Root Frame Diagnostics Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It proved the paint-root frame path-cache diagnostics adapter seam without widening into cache
begin, viewport, clip, background paint, grid paint, tail cleanup, cached/immediate passes, grid
tile diagnostics, or edge label budget diagnostics.

## Shipped State

- `frame_diagnostics_adapter.rs` defines `PaintRootFrameDiagnosticsCx` and
  `record_paint_root_path_cache_stats`.
- `frame_diagnostics_retained_cx.rs` is the retained `PaintCx` binding for path-cache diagnostics
  registry writes.
- `paint_root/frame/cache.rs` now owns only snapshot collection through
  `diagnostics_path_cache_snapshot()` before delegating recording to the diagnostics adapter.
- Source-policy coverage in `ecosystem/fret-node/src/lib.rs` keeps the diagnostics adapter free of
  retained lifecycle context names and verifies the retained binding owns window/node/frame-id and
  registry writes.

## Split State

The following paint-root frame operation families remain intentionally outside this lane:

- background paint,
- grid paint,
- tail cleanup / `SceneOp::PopClip`,
- cached/immediate pass clip emission,
- grid tile diagnostics and edge label budget diagnostics.

The next follow-on should choose one operation family. The smallest likely candidate is background
paint because it still combines a chrome/skin policy lookup with a retained scene quad emission.

## Closeout Evidence

- `docs/workstreams/fret-node-paint-root-frame-clip-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame/cache.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_diagnostics_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/frame_diagnostics_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `cargo fmt --package fret-node` - passed in FDA-020.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_frame_diagnostics_adapter` -
  passed in FDA-020.
- `cargo check -p fret-node` - passed in FDA-020.
- `cargo check -p fret-node --features compat-retained-canvas` - passed in FDA-020.
- `python3 -m json.tool docs/workstreams/fret-node-paint-root-frame-diagnostics-adapter-v1/WORKSTREAM.json` -
  passed in FDA-020 and rerun for closeout.
- `python3 tools/check_workstream_catalog.py` - passed in FDA-020 and rerun for closeout.
- `python3 tools/check_layering.py` - passed in FDA-020.
- `git diff --check` - passed in FDA-020 and rerun for closeout.

## Residual Risks

- `paint_root/frame.rs` still takes retained `PaintCx` because background paint and grid paint still
  need retained context access.
- Grid tile diagnostics and edge label budget diagnostics still use retained registry writes in
  their own files; they should not be folded into this closed lane.
