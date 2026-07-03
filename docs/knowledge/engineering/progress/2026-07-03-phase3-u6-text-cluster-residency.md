---
type: Work Progress
title: Phase 3 U6 text cluster residency
tags: fret,phase3,u6,text,cluster,residency,renderer
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
subagent: 019f27e8-b218-7bc3-b070-ee0df54a3246
---

# Summary

This slice upgrades WGPU text resource closure from glyph-only residency to cluster-aware residency
while keeping atlas pinning glyph-based.

Changes:

- `TextShape` now stores CPU-side `TextGlyphCluster` metadata alongside glyphs.
- `GlyphInstance` now records a `cluster_index`.
- Shape build records cluster membership during glyph materialization and finalizes cluster
  metadata into the shape.
- `TextFrameResidency` now records cluster-aware entry keys: text blob, atlas reset generation,
  cluster fingerprint/count, glyph fingerprint/count.
- `visible_text` now asks `TextSystem` for cluster residency and uses cluster visual bounds for
  scissor/shadow inclusion.
- Full-blob test/parity helpers remain for U7; U6 only creates the successor oracle.

# Design Finding

`fret-render-text::TextLineCluster` is not always the right residency cluster. For Inter `calt`
ligatures, line clusters can represent caret/visual slices (`0..1`, `1..2`) while the materialized
glyph covers the full visual advance. U6 therefore builds WGPU `TextGlyphCluster` from materialized
glyph visual coverage and uses line clusters only to union visual bounds and expand text ranges.

This preserves caret/selection data in `TextLineLayout` while making renderer residency
shape-cluster aware.

# Verification

Passed:

- `cargo check -p fret-render-wgpu`
- `cargo nextest run -p fret-render-wgpu text_shape_records_cluster_metadata_for_inter_ligature cluster_residency_pins_complete_multi_glyph_cluster visible_text_residency_pins_complete_combining_cluster_under_narrow_scissor visible_text_glyph_residency_excludes_offscreen_suffix_glyphs --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu text --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu visible_text scene_chunk_encoding_cache --no-fail-fast`
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`

# Deletion Gate

Static search in `crates/fret-render-wgpu/src` now finds no remaining
`push_glyph_residency_for_blob` or `push_glyphs(` entrypoints. U7 can use this successor oracle to
retire full-blob helper scaffolding where it no longer provides unique parity value.

# Next

Proceed to U7:

- Rename or remove remaining full-blob text helper scaffolding that is only a parity oracle.
- Keep normal chunk/resource keys on cluster-aware visible residency.
- Preserve any full-blob helper that still has a unique test-only parity role until replaced.
