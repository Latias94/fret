# Renderer vNext Fearless Refactor v1 — TODO Tracker

Status: Draft (workstream tracker)

This document tracks TODOs for:

- `docs/workstreams/renderer-vnext-fearless-refactor-v1.md`
- `docs/workstreams/renderer-vnext-fearless-refactor-v1-milestones.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `REN-VNEXT-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `fretboard diag` script/suite name

## M0 — Design baseline

- [x] REN-VNEXT-docs-001 Add a short “invariants checklist” appendix for renderer refactors (what must never change).
  - Evidence: `docs/workstreams/renderer-vnext-fearless-refactor-v1.md` (Appendix A).
- [x] REN-VNEXT-adr-001 Draft ADR: isolated opacity / saveLayer(alpha) (group alpha).
- [x] REN-VNEXT-adr-002 Draft ADR: clip path + image mask sources (bounded, cacheable, deterministic).
- [x] REN-VNEXT-adr-003 Draft ADR: paint/material portability closure (capabilities + fallbacks + conformance gates).
- [x] REN-VNEXT-adr-004 Draft ADR: RenderSpace + scissor-sized intermediates (renderer-internal coordinate contract).
  - Evidence: `crates/fret-render-wgpu/src/renderer/resources.rs` (RenderSpace dynamic offsets), `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (per-pass RenderSpace writes + bind-group offsets).
- [x] REN-VNEXT-guard-000 Record the workstream’s baseline gate set and a baseline perf/telemetry snapshot (linkable).

## M1 — RenderPlan compilation substrate

- [~] REN-VNEXT-plan-001 Define the internal RenderPlan IR (segments, sequence points, state snapshots).
  - Draft: `docs/workstreams/renderer-vnext-fearless-refactor-v1.md` (3.1.1–3.1.5).
  - Evidence (partial): `crates/fret-render-wgpu/src/renderer/render_plan_compiler_vnext.rs` (`compile_for_scene_vnext_effects_only`, `EffectMarkerKind::ClipPathPush`/`ClipPathPop`).
- [ ] REN-VNEXT-plan-002 Move budget/degradation decisions into plan compilation (deterministic ordering).
  - Draft: `docs/workstreams/renderer-vnext-fearless-refactor-v1.md` (3.2.1–3.2.4).
- [ ] REN-VNEXT-plan-003 Add telemetry hooks: per-window intermediate peak bytes and degradations applied.
  - Evidence (partial): `crates/fret-render-wgpu/src/renderer/render_plan.rs` (`RenderPlanCompileStats`, `RenderPlanDegradation`),
    `crates/fret-render-wgpu/src/renderer/types.rs` (`RenderPerfSnapshot` fields),
    `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (plumbs plan stats into perf),
    `crates/fret-render-wgpu/src/renderer/config.rs` (perf snapshot output),
    `crates/fret-render-wgpu/src/renderer/render_plan_dump.rs` (JSON dump: estimated peak bytes + degradations list).
- [x] REN-VNEXT-plan-004 Introduce a switch to run old vs new paths and compare results for a small fixed scene set.
  - Evidence: `crates/fret-render-wgpu/src/renderer/mod.rs` (`RenderPlanCompilerFlavor`),
    `crates/fret-render-wgpu/src/renderer/config.rs` (`set_render_plan_compiler_flavor`),
    `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (compiles RenderPlan via selected flavor),
    `crates/fret-render-wgpu/tests/render_plan_compiler_compare_conformance.rs`.

## M2 — Isolated opacity (saveLayerAlpha)

- [x] REN-VNEXT-alpha-001 Decide contract shape: extend `CompositeGroupDesc` vs add a dedicated opacity group op.
- [x] REN-VNEXT-alpha-002 Add a GPU conformance test for isolated alpha vs non-isolated alpha mismatch cases.
- [x] REN-VNEXT-alpha-003 Bound group/effect computation by scissor during encoding (before scissor-sized targets).
- [x] REN-VNEXT-alpha-004 Allocate scissor-sized intermediates for groups/effects (with quality downsample tiers).

## M3 — ClipPath + image masks (bounded + cacheable)

### M3a — ClipPath v1

- [x] REN-VNEXT-clip-001 Decide v1 clip-path contract shape (prepared path handle vs dedicated clip handle).
  - Evidence (v1): `crates/fret-core/src/scene/mod.rs` (`SceneOp::PushClipPath`), `crates/fret-render-wgpu/src/renderer/render_scene/encode/ops.rs` (encoding + effect markers), `crates/fret-render-wgpu/src/renderer/render_plan.rs` (mask pass planning + composite with mask).
- [x] REN-VNEXT-clip-003 Add conformance tests:
  - [x] Clip-path clips to shape (not just bounds): `crates/fret-render-wgpu/tests/clip_path_conformance.rs`
  - [x] Clip capture at push time (does not follow later transforms): `crates/fret-render-wgpu/tests/clip_path_conformance.rs`
  - [x] Budget degradation is deterministic (scissor-only fallback): `crates/fret-render-wgpu/tests/clip_path_conformance.rs`
  - [x] clip-before-transform scrolling cases (partial-overlap cases): `crates/fret-render-wgpu/tests/clip_path_conformance.rs`
  - [x] clip under rotation (affine): `crates/fret-render-wgpu/tests/clip_path_conformance.rs`
  - [x] nested clips + clips + groups: `crates/fret-render-wgpu/tests/clip_path_conformance.rs`

### M3b — Image masks v1

- [x] REN-VNEXT-clip-002 Decide image-mask v1 sampling semantics (minimal enum, deterministic degradation).
  - Evidence: `docs/adr/0273-clip-path-and-image-mask-sources-v1.md` (bounds-as-computation-bound + channel policy), `crates/fret-core/src/scene/mask.rs` (`Mask::Image` sanitize), `crates/fret-render-wgpu/src/renderer/render_scene/encode/mask.rs` (single-active image-mask + deterministic degrade), `crates/fret-render-wgpu/src/renderer/shaders.rs` (`mask_eval` kind=3 sampling).
- [x] REN-VNEXT-mask-001 Add conformance tests for nested masks + groups and paint-only hit-testing invariants.
  - [x] GPU coverage gates for `Mask::Image`: `crates/fret-render-wgpu/tests/mask_image_conformance.rs`
  - [x] Paint-only hit-testing invariants (runtime-level): `crates/fret-ui/src/declarative/tests/core.rs` (`mask_layer_is_paint_only_for_hit_testing_by_default`)

## M4 — Paint/Material evolution (controlled extensibility)

### M4a — Capability matrix + deterministic fallbacks

- [x] REN-VNEXT-paint-001 Inventory where `Paint` is supported vs missing (quad/path/stroke/mask).
  - Evidence: `docs/workstreams/renderer-vnext-fearless-refactor-v1.md` (Appendix B).
- [x] REN-VNEXT-paint-002 Decide whether `SceneOp::Path` should accept `Paint` in v1/v2 (or remain solid-only).
  - Decision (v1): remain solid-only.
  - Evidence: `docs/workstreams/renderer-vnext-fearless-refactor-v1.md` (Appendix B).
- [x] REN-VNEXT-mat-001 Document the renderer’s MaterialId capability matrix and deterministic fallbacks for wasm/mobile.
  - Evidence: `docs/workstreams/renderer-vnext-fearless-refactor-v1.md` (Appendix C).
- [x] REN-VNEXT-mat-002 Fill the capability matrix table with concrete “Must/May/Degrade” decisions per target.
  - Evidence: `docs/workstreams/renderer-vnext-fearless-refactor-v1.md` (Appendix C).
- [x] REN-VNEXT-mat-003 Add at least one conformance scene for `Paint::Material` fallback behavior (unsupported registration, missing id, and budget pressure).
  - Evidence: `crates/fret-render-wgpu/tests/materials_conformance.rs` (unknown id + budget pressure), `crates/fret-render-wgpu/src/renderer/services.rs` (capability-gated registration).

### M4b — Optional contract expansion

- [ ] REN-VNEXT-paint-010 If `Path` accepts `Paint`, add a conformance gate for gradient/material path fills.

## M5 — Sampling hints (bounded state surface)

- [ ] REN-VNEXT-samp-001 Decide where sampling hints live (image op, viewport op, or material).
- [ ] REN-VNEXT-samp-002 Add a small conformance scene that exercises nearest/linear on mixed primitives without reordering.

## Always-run guardrails (before/after each milestone)

- [ ] REN-VNEXT-guard-001 Keep `python3 tools/check_layering.py` green for all intermediate steps.
- [ ] REN-VNEXT-guard-002 Add/extend at least one renderer conformance test per new contract.
- [ ] REN-VNEXT-guard-003 Record a perf snapshot baseline and keep “worst bundles” attachable to milestones.
