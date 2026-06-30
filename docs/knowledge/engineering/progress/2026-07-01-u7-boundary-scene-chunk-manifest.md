---
type: Work Progress
title: U7 boundary scene chunk manifest
tags: fret,ui,scene,renderer,u7
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
source_session: 019f143b-4f62-7333-a9b1-c3c54cf1409e
---

# Summary

U7's fourth implementation slice publishes retained scene chunks as boundary-owned manifests instead
of adding chunk sidecar ranges to the flat `SceneRecording`. `BoundarySceneFragmentDebug` can now
append retained chunk entries, `BoundaryFrameProducts::scene_fragment` stores them with the typed
scene-fragment product, and `UiTree::scene_fragment_chunk_manifest(...)` exposes the current
boundary product for future renderer bridge work.

# Decision

Do not make `SceneRecording` the source of retained chunk identity. It remains the compatibility
bridge for flat replay.

Two read-only explorers informed the next cuts:

- Explorer `019f1ac0-8ff3-7932-88a0-597f24becaaa`: publish chunk manifests from boundary products,
  not `SceneRecording` sidecars. Independently encoded chunks still need transform, clip, mask,
  effect/composite/backdrop, opacity, resource generation, and text/glyph context.
- Explorer `019f1ac0-dad2-7df2-98a9-d7e838fb2921`: renderer reuse should first attach to
  `RenderPlanSegment` metadata/reporting, not `SceneEncodingState` caching or `geometry_upload`
  writes.

# Verified State

Relevant checks passed:

- `cargo check -p fret-ui --all-targets`
- `cargo check -p fret-code-editor --all-targets`
- `cargo nextest run -p fret-ui boundary_frame_products_own_boundary_dirty_prepaint_interaction_scene_and_paint_cache_state canvas_prepaint_can_prepare_text_scene_fragment_before_paint --no-fail-fast`
- `cargo nextest run -p fret-code-editor row_scene_replay_plan_reports_scene_chunk_debug_metadata --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

# Open Threads

The next renderer-facing U7 slice should add evidence-only chunk/reuse-candidate fields to render
plan segment reporting/perf. It should not change scene encoding cache behavior, pass ordering,
partial upload ownership, or dirty GPU range writes yet.

# Citations

- [Plan](../../../plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md)
- [Boundary scene chunk diagnostics](2026-07-01-u7-boundary-scene-chunk-diagnostics.md)
- [Scene chunk compatibility bridge](2026-07-01-u7-scene-chunk-compatibility-bridge.md)
