---
type: Work Progress
title: Phase 3 U10 authoritative chunk launch
tags: fret,phase3,u10,renderer,launch,scene-chunks
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U10 moved normal native and web launch to the shared source-selection contract:
supported `SceneChunkManifest` frame classes render through authoritative `ChunkManifest`, while
unsupported manifests fall back through explicit `FlatCompat` diagnostics instead of a hidden
flat-scene plus diagnostic-chunk source.

# Implemented

- Native desktop redraw and web render-loop paths now pass
  `RenderSceneSourcePolicy::chunk_manifest_when_supported()` into `select_render_scene_source`.
- Renderer perf snapshots now expose `render_scene_source_*` counters for authoritative chunk
  frames, explicit flat-compat frames, unsupported fallback frames, and unsupported reason families
  for empty manifests, mixed streams, open/inherited scope, and side-table/resource relocation gaps.
- Bootstrap UI frame stats and `fret-diag` perf-key registry now publish the same
  `renderer_render_scene_source_*` fields for diagnostics bundles and perf gates.
- Renderer tests prove resource-free quad manifests render as chunk-native source with zero
  `FlatCompat` frames, while side-table manifests render through `FlatCompat` with structured
  unsupported counters.
- Runtime contract docs, UI closure map, and ADR implementation alignment now cite U10 launch
  policy and diagnostic evidence.

# Boundaries

U10 does not claim chunk-native support for clip, mask, effect, text, path, image, SVG,
viewport-surface, material, custom-effect, resource, or mixed-stream frames. Those remain explicit
unsupported fallback evidence until U11+ proves per-stream relocation, resource closure, and
partial-upload policy.

# Verification

- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-launch --all-targets`
- `cargo check -p fret-bootstrap --lib`
- `cargo check -p fret-diag --all-targets`
- `cargo nextest run -p fret-render-wgpu source_selection source_unsupported source_chunk resource_free_quad_scene_chunk_manifest unsupported_side_table_manifest --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive registered_perf_key_inventory_doc_is_in_sync full_registered_perf_key_registry_covers_consumed_debug_stats_fields --no-fail-fast`
- `cargo nextest run -p fret-launch --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu --no-fail-fast`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`
- Static launch search: `flat_with_diagnostic_chunks` is absent from normal launch; remaining
  `RenderSceneSourcePolicy::flat_compat()` matches are renderer debug/parity tests only.

# Next

U11 should turn the existing resource-free quad plus vertex-color special case into an explicit
per-stream upload policy table with fallback reasons, write-count/byte budgets, and negative
coverage-gap tests before expanding more stream classes.
