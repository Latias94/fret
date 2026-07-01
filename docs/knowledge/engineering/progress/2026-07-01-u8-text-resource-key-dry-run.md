---
type: Work Progress
title: U8 scene text resource key dry-run diagnostics
tags: fret,u8,text,glyph-atlas,scene-cache,diagnostics
timestamp: 2026-07-01
related_plan: docs/plans/2026-06-30-001-refactor-ui-framework-architecture-plan.md
git_branch: feat/ui-framework-convergence
---

# Summary

U8 now records a diagnostics-only scene text resource fingerprint after per-frame text residency and
atlas upload flushing. The fingerprint is derived from the current scene's text blobs, glyph keys,
resolved atlas page/UV resources, missing glyph-resource count, and atlas reset generation. Renderer
perf also records when the global text atlas revision changes while the current scene text resource
fingerprint stays stable.

# Decisions

- Keep `text_atlas_revision` in scene and chunk encoding cache keys for now. Cached encodings still
  contain atlas-dependent page/UV data, so directly removing the revision key would risk replaying
  stale atlas slots after eviction or reset.
- Use this slice as a dry-run evidence layer for a future narrower resource key. The next safe
  behavior change should either add per-entry glyph resource generations or make glyph residency
  truly visible-range driven before changing cache invalidation semantics.
- Record the fingerprint only when renderer perf is enabled so normal render frames do not pay the
  extra scene text walk.

# Changed Files

- `crates/fret-render-wgpu/src/text/diagnostics.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/frame_prepare.rs`
- `crates/fret-render-wgpu/src/renderer/types.rs`
- `crates/fret-render-wgpu/src/renderer/config.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/perf_finalize.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/frame_stats.rs`
- `crates/fret-diag/src/perf_keys.rs`
- `docs/workstreams/diag-perf-profiling-infra-v1/perf-key-registry.frame-stats.json`

# Verification

- `cargo fmt --all --check`
- `cargo check -p fret-render-wgpu --all-targets`
- `cargo nextest run -p fret-render-wgpu scene_text_resource_snapshot_ignores_unreferenced_atlas_revision_churn text_scene_resource_key_state_counts_atlas_revision_churn_with_stable_resources --no-fail-fast`
- `cargo check -p fret-bootstrap --lib --features ui-app-driver,diagnostics`
- `cargo check -p fret-diag --all-targets`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive registered_perf_key_inventory_doc_is_in_sync full_registered_perf_key_registry_covers_consumed_debug_stats_fields --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `cargo check -p fret-render-wgpu --target wasm32-unknown-unknown --lib`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/perf/audit_perf_baselines.py --matrix docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md --strict`
- `git diff --check`

# Next Action

Continue U8 with the behavior-changing cut: either add per-entry glyph resource generation keys and
switch scene/chunk cache invalidation to referenced resources, or first cut off prepare-time atlas
insertion and make residency visible glyph range driven.
