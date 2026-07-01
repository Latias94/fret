---
type: Work Progress
title: U8 prepare atlas residency split
tags: fret,u8,text,glyph-atlas,residency,renderer-cache
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U8 now separates CPU text preparation from GPU glyph-atlas residency. `TextSystem::prepare` still
builds layout, glyph keys, bounds, decorations, and shape/blob cache entries, but it no longer probes,
touches, inserts into, or queues uploads for the glyph atlas. Atlas residency is now owned by the
frame residency path (`prepare_for_scene` / pin-bucket prewarm).

# Decisions

- Delete the old prepare-time atlas epoch and glyph-kind lookup cache. They only existed to make
  prepare-time atlas probing cheaper, and keeping them would preserve the wrong ownership model.
- Keep CPU glyph rasterization during shape construction for now because prepared shapes still need
  glyph bounds and image-content kind. The next U8 visible-range slice can consider a bounds-only path.
- Retain current whole-scene frame prewarm semantics. Visible-range glyph residency remains a separate
  follow-up so this slice does not mix ownership cleanup with culling policy.

# Changed Files

- `crates/fret-render-wgpu/src/text/prepare/*`
- `crates/fret-render-wgpu/src/text/atlas.rs`
- `crates/fret-render-wgpu/src/text/atlas_runtime_state.rs`
- `crates/fret-render-wgpu/src/text/bootstrap.rs`
- `crates/fret-render-wgpu/src/text/fonts.rs`
- `crates/fret-render-wgpu/src/text/mod.rs`
- `crates/fret-render-wgpu/src/text/tests.rs`
- `crates/fret-render-wgpu/src/renderer/tests.rs`
- `docs/workstreams/renderer-modularity-fearless-refactor-v1/`

# Deleted Files

- `crates/fret-render-wgpu/src/text/atlas_epoch.rs`
- `crates/fret-render-wgpu/src/text/glyph_kind_cache.rs`

# Verification

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo nextest run -p fret-render-wgpu prepare_does_not_probe_resident_atlas_glyphs prepare_does_not_insert_unreferenced_glyphs_into_atlas scene_text_resource_snapshot_ignores_unreferenced_prepare unreferenced_text_atlas_churn_does_not_bust_scene_or_chunk_encoding_cache --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu --no-fail-fast`
- `cargo check -p fret-render-wgpu --target wasm32-unknown-unknown --lib`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `git diff --check`

# Next Action

Continue U8 by replacing whole-scene text prewarm with a visible glyph residency prepass that derives
the glyph set from actual render visibility.
