---
type: Work Progress
title: Phase 3 U11 partial upload stream policy
tags: fret,phase3,u11,renderer,geometry-upload,diagnostics
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

Phase 3 U11 replaced the implicit quad plus vertex-color partial-upload special case with an
explicit resident geometry upload stream policy table. Partial upload is now allowed only when a
stream names its closure owner, relocation dependencies, resource dependencies, support status, and
write budget.

# Implemented

- `ResidentGeometryUploadStreamPolicy` now classifies each geometry stream in
  `crates/fret-render-wgpu/src/renderer/geometry_upload.rs`.
- `QuadInstances` remain partial-upload capable with unbounded budget metadata.
- Resource-free `VertexColor` viewport vertices are the first supported non-quad partial stream.
  Their initial budget is one partial write and six viewport vertices per frame.
- Paint, text, glyph, text-vertex, and path streams remain full-upload-only with
  `StreamPolicyUnsupported` fallback classification.
- Viewport-vertex segments that carry image, viewport, mask, text, or path flags are demoted to
  full upload before signatures can enter resident partial planning.
- Renderer perf snapshots, bootstrap UI frame stats, `fret-diag`, and the generated perf-key
  registry now expose the finite budget, unsupported stream-policy fallbacks, and budget-overflow
  fallbacks.
- Runtime contract docs, UI closure map, and ADR 0327 implementation alignment now cite the U11
  stream-policy boundary.

# Verification

- `cargo fmt --all`
- `cargo fmt --all --check`
- `git diff --check`
- `cargo check -p fret-render-wgpu --all-targets`
- `cargo check -p fret-bootstrap --lib`
- `cargo check -p fret-diag --all-targets`
- `cargo nextest run -p fret-render-wgpu geometry_upload uploads --no-fail-fast`
- `cargo nextest run -p fret-diag registered_perf_keys_are_unique registered_perf_key_units_match_names registered_perf_key_contract_keeps_stats_and_gate_keys_additive registered_perf_key_inventory_doc_is_in_sync full_registered_perf_key_registry_covers_consumed_debug_stats_fields --no-fail-fast`
- `cargo nextest run -p fret-bootstrap --lib --no-fail-fast`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`

# Next

U12 should clean public app facade constructors and cookbook raw seams. The key deletion pressure is
to stop default examples from teaching `LocalState::new_in`, raw model/action seams,
`fret::advanced`, `UiTree`, or manual driver assembly as the first-contact path.
