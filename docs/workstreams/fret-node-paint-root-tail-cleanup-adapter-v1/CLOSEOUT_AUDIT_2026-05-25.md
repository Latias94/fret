# Fret Node Paint Root Tail Cleanup Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It proved the root frame tail cleanup `PopClip` adapter seam without widening into cached layer
internals, cached/immediate passes, overlays, cache pruning, or public scene schema changes.

## Shipped State

- `tail_cleanup_adapter.rs` defines `PaintRootTailCleanupCx` and `pop_paint_root_tail_clip`.
- `tail_cleanup_retained_cx.rs` is the retained `PaintCx` binding for root frame tail cleanup
  `SceneOp::PopClip` emission.
- `paint_root/tail.rs` now preserves overlay and pruning order, then delegates root frame pop
  emission through the tail cleanup adapter.
- Source-policy coverage in `ecosystem/fret-node/src/lib.rs` keeps the tail cleanup adapter free of
  retained lifecycle context names and `SceneOp`, verifies `tail.rs` no longer emits root `PopClip`
  directly, and verifies the retained binding owns scene emission.

## Split State

The following frame operation families remain intentionally outside this lane:

- cached/immediate pass clip emission,
- cached node/group/edge internal clip ops,
- grid plan and chrome hint routing,
- overlay paint and cache pruning.

The next follow-on should choose one operation family. The smallest likely candidate is
cached/immediate pass clip emission because it still passes retained scene access through pass
execution paths.

## Closeout Evidence

- `docs/workstreams/fret-node-paint-root-frame-grid-diagnostics-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/tail_cleanup_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/tail_cleanup_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_tail_cleanup_adapter` -
  passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 -m json.tool docs/workstreams/fret-node-paint-root-tail-cleanup-adapter-v1/WORKSTREAM.json` -
  passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Residual Risks

- Cached/immediate pass clip emission still carries retained scene access through pass execution.
- Cached node/group/edge internals still construct cache-local clip ops directly; those are separate
  cache layer concerns.
