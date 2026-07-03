---
type: Work Progress
title: Phase 3 U7 full-blob text helper scaffolding
tags: fret,phase3,u7,text,renderer,full-blob,oracle
timestamp: 2026-07-03
related_plan: ../../plans/2026-07-03-001-refactor-ui-framework-phase3-retained-bridge-deletion-plan.md
---

# Summary

This slice retires the remaining full-blob text helper names from the normal-looking renderer text
surface after U6 added cluster-aware residency.

Changes:

- Renamed `prepare_for_scene`, `prepare_for_scene_with_perf`, and
  `prepare_for_text_blobs_with_perf` to explicit `test_prepare_full_*` helpers.
- Renamed `text_residency_for_blobs` and `text_resource_snapshot_for_blobs` to explicit
  `test_full_blob_text_*` helpers.
- Deleted the `scene_text_resource_snapshot` wrapper because callers can pass the tested blob set
  directly to the test oracle.
- Kept the full-blob path behind `#[cfg(test)]` only, with crate-local visibility.
- Kept normal retained chunk/resource keys on `text_resource_snapshot_for_residency`, which now
  consumes the cluster-aware successor residency from U6.

# Verification

Passed:

- `cargo check -p fret-render-wgpu`
- `cargo nextest run -p fret-render-wgpu text scene_chunk_encoding_cache --no-fail-fast`
  - Result: 118 passed, 234 skipped. Nextest reported one existing leaky test while returning exit
    code 0.
- Static search over `crates/fret-render-wgpu/src` found no old helper names:
  `text_resource_snapshot_for_blobs`, `text_residency_for_blobs`,
  `prepare_for_text_blobs_with_perf`, `prepare_for_scene_with_perf`, or `prepare_for_scene(`.
- `cargo fmt --all --check`
- `git diff --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`
- `python3 tools/check_adr_numbers.py`
- `python3 tools/check_workstream_catalog.py`

# Deletion Gate

The U7 deletion gate is satisfied for helper names and normal-path exposure: full-blob text
residency remains only as an explicit test oracle, while normal chunk key generation continues to
flow through cluster-aware `TextFrameResidency`.

# Next

Proceed to U8:

- Split renderer source contracts so flat-scene semantics and diagnostic chunks are no longer
  conflated.
- Introduce `FrameAssembler` scaffolding as the owner for chunk payload cache, side-table
  relocation, resource residency, assembly diagnostics, and partial-upload candidate reporting.
