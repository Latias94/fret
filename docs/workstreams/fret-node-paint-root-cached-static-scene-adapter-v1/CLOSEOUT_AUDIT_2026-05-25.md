# Fret Node Paint Root Cached Static Scene Adapter v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-25

## Verdict

This lane is closed.

It proved the cached static group/node scene adapter seam without widening into cached edge replay,
edge labels, overlays, cache key policy, or public scene schema changes.

## Shipped State

- `cached_static_scene_adapter.rs` defines `PaintRootCachedStaticSceneCx` and cached static route
  inputs for host, services, scale factor, and scene replay.
- `cached_static_scene_retained_cx.rs` is the retained `PaintCx` binding for those route inputs.
- `static_layer.rs` and `static_cache.rs` no longer depend on `PaintCx` or direct `cx.scene` reads
  for cached static group/node scene replay.
- `cached_groups.rs` and `cached_nodes.rs` no longer read `cx.app`, `cx.services`, or
  `cx.scale_factor` directly when building cached static scene ops.
- Source-policy coverage in `ecosystem/fret-node/src/lib.rs` keeps the adapter free of retained
  lifecycle context names and scene ops, verifies cached static replay helpers use the adapter, and
  verifies the retained binding owns retained field access.

## Split State

The following scene-routing families remain intentionally outside this lane:

- cached edge scene replay,
- cached edge-label scene replay,
- cached local clip-op emission into temporary scenes,
- immediate edge paint routing,
- immediate or cached overlay layer paint routing.

The next follow-on should choose one cached edge replay family. Cached local clip-op emission is also
a valid follow-on, but edge replay has direct retained `cx.scene` reads and is the sharper next seam.

## Closeout Evidence

- `docs/workstreams/fret-node-paint-root-pass-clip-adapter-v1/CLOSEOUT_AUDIT_2026-05-25.md`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/static_layer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/static_cache.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_groups.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_nodes.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_static_scene_adapter.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_root/cached_static_scene_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`

## Fresh Gates

- `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-static-scene-adapter-v1/WORKSTREAM.json` -
  passed.
- `cargo fmt --package fret-node` - passed.
- `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_static_scene_adapter` -
  passed.
- `cargo check -p fret-node` - passed.
- `cargo check -p fret-node --features compat-retained-canvas` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `python3 tools/check_layering.py` - passed.
- `git diff --check` - passed.

## Residual Risks

- Cached edge and edge-label replay still read retained `cx.scene` directly.
- Cache-local `SceneOp::PushClipRect` / `PopClip` emission remains in cached group/node builders and
  can be split after edge replay if the source-policy line needs to become scene-op agnostic.
