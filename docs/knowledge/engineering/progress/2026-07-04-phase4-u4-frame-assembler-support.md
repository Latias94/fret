---
type: Work Progress
title: Phase 4 U4 FrameAssembler support ownership
tags: fret,phase4,renderer,frame-assembler,chunk-manifest
timestamp: 2026-07-04T15:58:01Z
related_plan: ../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Summary

This Phase 4 U4 slice moved chunk-launch support evaluation into `FrameAssembler`.
`ChunkLaunchSupportMatrix` was deleted from the renderer facades so support decisions no longer
look like an app-facing API. Source selection still exposes `ChunkLaunchSupport`, but the decision
now comes from `FrameAssembler::evaluate_support`.

# Design Finding

The next renderer stream class should not be promoted until side-table/resource relocation is
explicitly proven. The safer U4 cut is to make unsupported cases explainable at the assembler
boundary first:

- `FrameAssemblyUnsupportedReason` records unsupported launch, requested stream mismatch, entry
  stream mismatch, missing payloads, payload reassembly blockers, and payload stream mismatch.
- `try_assemble_supported_frame_encoding` is the structured path; the legacy `Option` method is a
  compatibility wrapper for the current renderer hot path.
- Debug flat oracle remains optional evidence. It does not turn mixed manifests into supported
  chunk-native launch.

# Verification

Passed:

- `cargo nextest run -p fret-render-wgpu frame_assembler_evaluates_mixed_manifest_with_structured_reason frame_assembler_rejects_side_table_manifest_before_payload_lookup frame_assembler_assembles_supported_quad_payloads source_selection_debug_flat_oracle_does_not_define_chunk_support --no-fail-fast`
- `cargo nextest run -p fret-render-wgpu --no-fail-fast`
- `cargo nextest run -p fret-render --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_layering.py`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `git diff --check`

Observed:

- `cargo clippy -p fret-render-wgpu -p fret-render --all-targets -- -D warnings` still fails on
  pre-existing lint in `geometry_upload.rs`, `scene_chunk_encoding_cache.rs`, and
  `text/layout_cache_state.rs`. A new `clone_on_copy` warning introduced by this slice was fixed.

# Next Action

The next U4 renderer cut should pick one genuinely new stream class only after the assembler can
name the required side-table/resource relocation inputs. Text/resource streams should remain
unsupported until U5 preserves shaping cluster/run information and a payload side-table plan exists.

# Citations

- [Phase 4 topology epoch plan](../../../plans/2026-07-04-001-refactor-ui-framework-phase4-topology-epoch-plan.md)
- [Runtime contract matrix](../../../runtime-contract-matrix.md)
