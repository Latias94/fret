# Fret Node Paint Root Cached Static Scene Adapter v1 - TODO

Status: Closed
Last updated: 2026-05-25

## CSA-M0 - Scope Freeze

- [x] CSA-010 [owner=codex] [deps=none] [scope=docs/workstreams/fret-node-paint-root-cached-static-scene-adapter-v1]
  Goal: Open a narrow follow-on for cached static group/node scene replay only.
  Validation: `python3 -m json.tool docs/workstreams/fret-node-paint-root-cached-static-scene-adapter-v1/WORKSTREAM.json`
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`
  Handoff: Keep cached edge replay, edge labels, overlays, cache key policy, and public scene schema
  changes out of scope.

## CSA-M1 - Cached Static Scene Adapter Seam

- [x] CSA-020 [owner=codex] [deps=CSA-010] [scope=ecosystem/fret-node/src/ui/canvas/widget/paint_root]
  Goal: Move cached static group/node host/services/scale/scene access behind an adapter plus
  retained `PaintCx` binding.
  Validation: `cargo test -p fret-node --features compat-retained-canvas paint_root_cached_static_scene_adapter`
  Evidence: `static_layer.rs`, `static_cache.rs`, `cached_groups.rs`, `cached_nodes.rs`,
  `cached_static_scene_adapter.rs`, `cached_static_scene_retained_cx.rs`,
  `ecosystem/fret-node/src/lib.rs`
  Handoff: Cached static group/node replay uses the adapter seam. Cached edge replay and overlays
  remain untouched. Complete.

## CSA-M2 - Closeout

- [x] CSA-030 [owner=codex] [deps=CSA-020] [scope=docs/workstreams/fret-node-paint-root-cached-static-scene-adapter-v1]
  Goal: Close the lane and split residual cached replay families into follow-on candidates.
  Validation: `python3 tools/check_workstream_catalog.py`; `git diff --check`
  Evidence: `CLOSEOUT_AUDIT_2026-05-25.md`
  Handoff: Start a separate follow-on for cached edge replay or cached local clip-op emission.
  Complete; see `CLOSEOUT_AUDIT_2026-05-25.md`.
