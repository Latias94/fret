# Renderer vNext Fearless Refactor v1 — TODO Tracker

Status: Active (workstream tracker)

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

- [x] REN-VNEXT-plan-001 Define the internal RenderPlan IR (segments, sequence points, state snapshots).
  - Draft: `docs/workstreams/renderer-vnext-fearless-refactor-v1.md` (3.1.1–3.1.5).
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_plan.rs` (`RenderPlanSegment`, `RenderPlanSegmentFlags`),
    `crates/fret-render-wgpu/src/renderer/render_plan_compiler.rs` (`alloc_segment`, sequence points at markers + path MSAA batches).
- [x] REN-VNEXT-plan-005 Remove the legacy plan compiler (and temporary switches/tests) after vNext parity is proven.
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (RenderPlan compilation has no flavor switch),
    `crates/fret-render-wgpu/src/renderer/render_plan.rs` (`compile_for_scene` delegates to vNext),
    `crates/fret-render-wgpu/Cargo.toml` (no legacy compiler feature).
- [x] REN-VNEXT-plan-002 Move budget/degradation decisions into plan compilation (deterministic ordering).
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_plan_compiler.rs` (scopes-aware effective budgets for effect chains),
    `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` (clip-mask bytes are budget-accounted; mask tiers respect unavailable targets).
- [x] REN-VNEXT-plan-003 Add telemetry hooks: per-window intermediate peak bytes and degradations applied.
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_plan.rs` (`RenderPlanCompileStats`, `RenderPlanDegradation`),
    `crates/fret-render-wgpu/src/renderer/types.rs` (`RenderPerfSnapshot` fields),
    `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (plumbs plan stats into perf),
    `crates/fret-render-wgpu/src/renderer/config.rs` (perf snapshot output),
    `crates/fret-render-wgpu/src/renderer/render_plan_dump.rs` (JSON dump: estimated peak bytes + degradations list),
    `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (segment stability counters: changed segments + pass growth).
- [x] REN-VNEXT-perf-110 Unify per-frame GPU buffer rotation for quad instances and vertex streams (viewport/text/path).
  - Goal: remove duplicated capacity growth + frame-rotation logic without changing bind group indices or shader semantics.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/buffers.rs` (`RingBuffer<T>`, `StorageRingBuffer<T>`)
    - `crates/fret-render-wgpu/src/renderer/resources.rs` (initialization)
    - `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (uploads + `next_pair`/`next_buffer`)
    - `crates/fret-render-wgpu/src/renderer/pipelines/quad.rs` (layout access via ring)
  - Gates (2026-02-16):
    - `python3 tools/check_layering.py`
    - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
    - `cargo test -p fret-render-wgpu --test text_paint_conformance`
- [x] REN-VNEXT-plan-004 Introduce a switch to run old vs new paths and compare results for a small fixed scene set.
  - Note: This was a temporary safety rail during rollout and has been removed after completing `REN-VNEXT-plan-005`.

## M2 — Isolated opacity (saveLayerAlpha)

- [x] REN-VNEXT-alpha-001 Decide contract shape: extend `CompositeGroupDesc` vs add a dedicated opacity group op.
- [x] REN-VNEXT-alpha-002 Add a GPU conformance test for isolated alpha vs non-isolated alpha mismatch cases.
- [x] REN-VNEXT-alpha-003 Bound group/effect computation by scissor during encoding (before scissor-sized targets).
- [x] REN-VNEXT-alpha-004 Allocate scissor-sized intermediates for groups/effects (with quality downsample tiers).

## M3 — ClipPath + image masks (bounded + cacheable)

### M3a — ClipPath v1

- [x] REN-VNEXT-clip-001 Decide v1 clip-path contract shape (prepared path handle vs dedicated clip handle).
  - Evidence (v1): `crates/fret-core/src/scene/mod.rs` (`SceneOp::PushClipPath`), `crates/fret-render-wgpu/src/renderer/render_scene/encode/ops.rs` (encoding + effect markers), `crates/fret-render-wgpu/src/renderer/render_plan_compiler.rs` (`EffectMarkerKind::ClipPathPush`/`ClipPathPop`).
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

- [x] REN-VNEXT-paint-010 If `Path` accepts `Paint`, add a conformance gate for gradient/material path fills.
  - Status: N/A for v1 (decision: `SceneOp::Path` remains solid-only).
  - Follow-up: if v2 expands `Path` to accept `Paint`, add a dedicated work item and gate at that time.

## M5 — Sampling hints (bounded state surface)

- [x] REN-VNEXT-samp-001 Decide where sampling hints live (image op, viewport op, or material).
  - Decision (v1): sampling hints live on image sampling sites (`SceneOp::Image*`, `SceneOp::MaskImage`, `Mask::Image`), not on `Paint`/`Material`.
  - Evidence: `docs/adr/0276-image-sampling-hints-v1.md`, `crates/fret-core/src/scene/mod.rs` (`ImageSamplingHint`, `SceneOp::{Image,ImageRegion,MaskImage}`),
    `crates/fret-core/src/scene/mask.rs` (`Mask::Image { sampling }`).
- [x] REN-VNEXT-samp-002 Add a small conformance scene that exercises nearest/linear on mixed primitives without reordering.
  - Evidence: `crates/fret-render-wgpu/tests/image_sampling_hint_conformance.rs`
- [x] REN-VNEXT-samp-003 Plumb sampling hints through the UI mechanism layer and add ecosystem opt-in helpers.
  - Goal: keep `crates/fret-ui` as a pure mechanism/pass-through, while allowing policy layers to opt in.
  - Evidence: `crates/fret-ui/src/element.rs` (`ImageProps.sampling`), `crates/fret-ui/src/declarative/host_widget/paint.rs` (SceneOp plumb),
    `ecosystem/fret-ui-kit/src/image_sampling.rs` (`ImageSamplingExt`), `ecosystem/fret-ui-kit/tests/image_sampling_ext_smoke.rs`,
    `ecosystem/fret-ui-shadcn/src/media_image.rs` (`MediaImage::sampling_hint`),
    `apps/fret-ui-gallery/src/ui/previews/gallery/atoms/media/image_object_fit.rs` (explicit Linear vs opt-in Nearest demo),
    `tools/diag-scripts/ui-gallery-image-sampling-hints-screenshots.json` (scripted screenshot/bundle gate for Linear vs Nearest).

## M5b — WebGPU/Tint uniformity closure (derivatives + sampling)

- [x] REN-VNEXT-webgpu-001 Make WGSL shaders satisfy WebGPU uniformity rules (Tint):
  - Derivative ops (`fwidth`, `dpdx`, `dpdy`) and sampling (`textureSample`) are not gated by non-uniform control flow.
  - Evidence: `crates/fret-render-wgpu/src/renderer/shaders.rs` (`mask_eval`, `paint_eval`, dashed border mask).
- [x] REN-VNEXT-webgpu-002 Recover performance after uniformity fixes:
  - Avoid “evaluate all material patterns per pixel” in the quad shader on web/mobile.
  - Preferred direction: compile a small set of shader/pipeline variants keyed by stable (bounded) paint/material kinds.
  - Landed: quad pipeline variants keyed by `(fill_kind, border_kind, border_present, dash_enabled)` using WGSL `override` constants.
  - Evidence: `crates/fret-render-wgpu/src/renderer/types.rs` (`QuadPipelineKey`), `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/quad.rs` (batch split),
    `crates/fret-render-wgpu/src/renderer/pipelines/quad.rs` (pipeline constants), `crates/fret-render-wgpu/src/renderer/shaders.rs` (override + `paint_eval_fill/border`),
    `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (variant selection per draw).
  - Landed: split `Paint::Material` params-only vs sampled into bounded per-side variants:
    - overrides: `FRET_{FILL,BORDER}_MATERIAL_SAMPLED`
    - goal: avoid material catalog `textureSample` on params-only paths.
    - Evidence: `crates/fret-render-wgpu/src/renderer/shaders.rs` (`material_eval(sample_catalog)`), `crates/fret-render-wgpu/src/renderer/pipelines/quad.rs` (constants),
      `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/quad.rs` (keying by `stop_count` aux channel).
  - Notes:
    - Headless perf gate exists: `python3 tools/perf/headless_quad_material_stress_gate.py` (baseline in `docs/workstreams/perf-baselines/`).
    - Any further variants are tracked as `REN-VNEXT-webgpu-004` and must be evidence-driven.
- [x] REN-VNEXT-webgpu-003 Add a stronger guardrail for WebGPU shader portability:
  - Keep `renderer::tests::shaders_validate_for_webgpu` as a baseline (Naga),
  - and add an optional browser (Tint) compile gate to catch uniformity drift early.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/tests.rs` (`webgpu_tint_compiles_all_wgsl_shaders`)

## M6 — Perf recovery follow-ups (evidence-driven)

- [x] REN-VNEXT-perf-001 Add a headless quad/material stress gate:
  - Target: quad shader hot paths (fill/border paint kinds, dash on/off, material sampled vs params-only).
  - Output: stable counters + baseline in `docs/workstreams/perf-baselines/`.
  - Motivation: keep pipeline-variant decisions bounded and justified.
  - Evidence:
    - `apps/fret-quad-material-stress/src/main.rs`
    - `tools/perf/headless_quad_material_stress_gate.py`
    - `docs/workstreams/perf-baselines/quad-material-stress-headless.windows-local.v1.json`
- [x] REN-VNEXT-diag-001 Surface renderer material counters in `diag perf` stats:
  - Motivation: make `REN-VNEXT-webgpu-004` evidence-driven (avoid expanding the pipeline key space blindly).
  - Output keys:
    - `top_renderer_material_quad_ops`
    - `top_renderer_material_sampled_quad_ops`
    - `top_renderer_material_distinct`
    - `top_renderer_material_unknown_ids`
    - `top_renderer_material_degraded_due_to_budget`
  - Evidence:
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`UiFrameStatsV1`)
    - `crates/fret-diag/src/stats.rs` (`BundleStatsSnapshotRow`)
    - `crates/fret-diag/src/lib.rs` (`diag perf --json` output)
- [ ] REN-VNEXT-webgpu-004 If perf evidence warrants, add bounded `MaterialTileMode` pipeline variants:
  - Note: “tile mode” here refers to the material-kind selector channel in the quad shader (see `material_eval`),
    not gradient tile modes (which are sanitized/degraded today).
  - Decision rule (do not do this without evidence):
    - Only proceed if material work is measurably hot in at least one reproducible bundle:
      - `fretboard diag perf` (or a GPU profiler) shows the quad fragment shader is a top hotspot and `material_eval` dominates.
      - Use `top_renderer_material_*` counters from `fretboard diag perf --json` (added in `REN-VNEXT-diag-001`) to keep this quantitative.
    - And the current bounded variants are insufficient (confirmed by one of):
      - `material_sampled_quad_ops` is high relative to `quad_draw_calls` in the headless gate’s `headless_renderer_perf_materials`
        output and wall time regresses in `fret-quad-material-stress` on the same machine.
      - or real app perf snapshots show unacceptable regression under WebGPU with no alternative mitigation.
  - Guardrail: keep the key space small and observable in perf snapshots (`pipeline_switches_*`), and update headless baselines
    if (and only if) the added variants are justified.
  - Status note (2026-02-15): `diag perf ui-gallery-steady` shows renderer encode is not a dominant contributor on the native Vulkan path
    (see `docs/workstreams/renderer-vnext-fearless-refactor-v1-milestones.md`). Keep this item pending until a WebGPU-specific bundle/profiler
    shows `material_eval` dominates and the existing bounded variants are insufficient.
- [x] REN-VNEXT-clean-001 Remove dead/legacy shader branches once variants cover all active cases.
  - Landed: quad shader skips inner-border SDF work when `FRET_BORDER_PRESENT=0` (compile-time override),
    keeping WebGPU uniformity rules satisfied while reducing waste in borderless variants.
  - Evidence: `crates/fret-render-wgpu/src/renderer/shaders.rs` (`fs_main`, `FRET_BORDER_PRESENT`)
  - Landed: CPU encoding skips converting `border_paint` (and avoids material budgets/counters) when the border widths are zero.
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/quad.rs` (`encode_quad`, `border_present`)
  - Landed: Image/mask/viewport surface draws now tighten the scissor to the transformed quad bounds (intersected with current scissor).
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/image.rs`, `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/mask.rs`,
    `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/viewport_surface.rs`
  - Landed: Text draws now tighten the scissor to the glyph-quad bounds per `(kind, atlas_page)` batch.
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs` (`flush_group`, `bounds_of_quad_points`)

## M8 — Renderer internals modularization (fearless, contract-preserving)

- [x] REN-VNEXT-refactor-001 Add a renderer-internals refactor design doc (staged plan + gates).
  - Evidence: `docs/workstreams/renderer-vnext-fearless-refactor-v1-refactor-design.md`
- [x] REN-VNEXT-refactor-002 Add an ADR that codifies internal ownership boundaries + always-run gates for refactors.
  - Landed: “encode/compile/execute” ownership and regression discipline without contract changes.
  - Evidence:
    - `docs/adr/0201-renderer-internals-modularization-and-gates-v1.md`
    - `docs/adr/IMPLEMENTATION_ALIGNMENT.md` (row update)
- [x] REN-VNEXT-refactor-010 Stage 1: centralize stable GPU globals (material catalog view/sampler, etc.).
  - Landed (step 1): reduce bind-group rebuild churn by making “stable sampler + linear/nearest pair” explicit and reusing renderer-owned globals
    in uniform bind groups.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/bind_group_caches.rs` (`SamplingBindGroups`, `SamplingBindGroups::pick`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/helpers.rs` (`pick_image_bind_group`, `pick_uniform_bind_group_for_mask_image`)
    - `crates/fret-render-wgpu/src/renderer/bind_group_builders.rs` (`UniformBindGroupGlobals`, `UniformMaskImageBindGroupGlobals`)
    - `crates/fret-render-wgpu/src/renderer/buffers.rs` (`rebuild_uniform_bind_group`)
    - `crates/fret-render-wgpu/src/renderer/resources.rs` (`UniformBindGroupGlobals::create`)
  - Landed (step 2): extract stable bind-group layouts/samplers/views into `GpuGlobals` and migrate call sites.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_globals.rs` (`GpuGlobals`)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::globals`)
    - `crates/fret-render-wgpu/src/renderer/pipelines/` (pipeline layouts bind `globals.*_bind_group_layout`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/bind_groups.rs` (prepares via `globals` samplers/layouts)
  - Landed (step 3): extract texture ownership + upload-once state into `GpuTextures`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_textures.rs` (`GpuTextures`)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::textures`)
    - `crates/fret-render-wgpu/src/renderer/resources.rs` (`ensure_material_catalog_uploaded`, `ensure_mask_image_identity_uploaded`)
  - Landed (step 4): extract quad/viewport pipeline caches into `GpuPipelines`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (`GpuPipelines`)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::pipelines`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/scene_draw.rs` (`quad_pipeline_ref`, `viewport_pipeline_ref`)
  - Landed (step 5): move text pipeline caches into `GpuPipelines`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (text pipeline cache fields)
    - `crates/fret-render-wgpu/src/renderer/pipelines/text.rs` (`ensure_text_*`, `*_pipeline_ref`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/scene_draw.rs` (uses `*_pipeline_ref`)
  - Landed (step 6): move mask/path pipeline caches into `GpuPipelines`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (mask/path pipeline cache fields)
    - `crates/fret-render-wgpu/src/renderer/pipelines/{mask,path,path_clip_mask}.rs` (ensure + `*_pipeline_ref`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/{scene_draw,path_clip_mask,path_msaa}.rs` (uses `*_pipeline_ref`)
  - Landed (step 7): move composite pipeline caches into `GpuPipelines`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (composite pipeline cache fields)
    - `crates/fret-render-wgpu/src/renderer/pipelines/composite.rs` (`ensure_composite_pipeline`, `*_ref`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/{effects,path_msaa}.rs` (call sites)
  - Landed (step 8): move clip-mask pipeline cache into `GpuPipelines`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (clip-mask pipeline cache field)
    - `crates/fret-render-wgpu/src/renderer/pipelines/clip_mask.rs` (`ensure_clip_mask_pipeline`, `clip_mask_pipeline_ref`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs` (`record_clip_mask_pass`)
  - Landed (step 9): move blit/blur pipeline caches into `GpuPipelines`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (blit/blur fields)
    - `crates/fret-render-wgpu/src/renderer/pipelines/{blit,blur}.rs` (`ensure_*`, `*_ref`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/{blit,blur}.rs` (call sites)
  - Landed (step 10): move scale-nearest pipeline caches into `GpuPipelines`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (scale-nearest fields)
    - `crates/fret-render-wgpu/src/renderer/pipelines/scale_nearest.rs` (`ensure_scale_nearest_pipelines`, `*_ref`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/scale_nearest.rs` (call sites)
  - Landed (step 11): move backdrop-warp pipeline caches into `GpuPipelines`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (backdrop-warp pipeline cache fields)
    - `crates/fret-render-wgpu/src/renderer/pipelines/backdrop_warp.rs` (`ensure_backdrop_warp_pipeline`, `*_ref`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/backdrop_warp.rs` (uses `*_ref`)
  - Landed (step 12): move effect pipeline caches into `GpuPipelines` (color-adjust, color-matrix, alpha-threshold, drop-shadow).
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_pipelines.rs` (effect pipeline cache fields)
    - `crates/fret-render-wgpu/src/renderer/pipelines/{color_adjust,color_matrix,alpha_threshold,drop_shadow}.rs` (ensure + `*_ref`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs` (uses `*_ref`)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`
  - Goal: reduce churn in bind group rebuild paths and make ownership explicit.
  - Gate: `cargo test -p fret-render-wgpu --lib` + `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
- [x] REN-VNEXT-refactor-020 Stage 2: consolidate GPU buffer lifecycle management (capacity growth + dependent bind group rebuilds).
  - Goal: one place to reason about “recreate buffer → rebuild bind group → invalidate caches”.
  - Landed (step 1): centralize uniform-dependent buffer replacement so every resize flows through a single rebuild+invalidate path.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/buffers.rs` (`ensure_*_capacity`, `rebuild_uniform_bind_group`)
    - `crates/fret-render-wgpu/src/renderer/uniform_resources.rs` (`ensure_*_capacity`)
  - Landed (step 2): make uniform-resource invalidation explicit and versioned (uniform buffers ↔ mask-image override bind groups).
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/uniform_resources.rs` (`UniformResources::revision`, `UniformResources::bump_revision`)
    - `crates/fret-render-wgpu/src/renderer/bind_group_caches.rs` (`ensure_uniform_mask_image_override_bind_groups`, `invalidate_uniform_mask_image_override_bind_groups`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/bind_groups.rs` (`prepare_uniform_mask_image_bind_groups`)
  - Landed (step 3): extract uniform/clip/mask/render-space buffers into a dedicated `UniformResources` subsystem.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/uniform_resources.rs` (`UniformResources`)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::uniforms`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (writes to `uniforms.*_buffer`)
  - Landed (step 4): extract effect parameter buffers (clip-mask params, scale-nearest params, effect uniform params) into `GpuEffectParams`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_effect_params.rs` (`GpuEffectParams`)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::effect_params`)
    - `crates/fret-render-wgpu/src/renderer/resources.rs` (`GpuEffectParams` init)
    - `crates/fret-render-wgpu/src/renderer/pipelines/clip_mask.rs` (layout uses `effect_params`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (scale-param capacity via `effect_params`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/recorders/{scale_nearest,backdrop_warp,effects}.rs` (buffer uses)
  - Gate: run the anchor conformance set listed in ADR 0201.
- [x] REN-VNEXT-refactor-030 Stage 3: extract bind group caches as explicit services with local invalidation.
  - Goal: isolate `image_bind_groups`, `viewport_bind_groups`, and mask-image override bind groups behind a single cache facade.
  - Landed (step 1): move viewport/image sampler+texture bind group caching behind `BindGroupCaches` methods (no recorder-side closures).
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/bind_group_caches.rs` (`ensure_viewport_sampler_texture_bind_group`, `ensure_image_sampler_texture_bind_groups`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/bind_groups.rs` (call sites in `prepare_*_bind_groups`)
  - Landed (step 2): move uniform mask-image override bind group caching behind `BindGroupCaches` methods (no recorder-side closures).
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/bind_group_caches.rs` (`ensure_uniform_mask_image_override_bind_groups`, `invalidate_uniform_mask_image_override_bind_groups`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/bind_groups.rs` (`prepare_uniform_mask_image_bind_groups`)
  - Landed (step 3): codify cache key/invalidation contract and provide an explicit full invalidation entrypoint.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/bind_group_caches.rs` (`BindGroupCaches` contract doc, `invalidate_all`)
  - Gate: run the anchor conformance set listed in ADR 0201.

- [x] REN-VNEXT-refactor-040 Stage 4: extract image/render-target registries + revision/generation tracking into an explicit subsystem.
  - Goal: keep “resource registry mutation → revision/generation bump → bind group cache invalidation” localized and reviewable.
  - Landed (step 1): move registry state (`ImageRegistry`, `RenderTargetRegistry`) + revision/generation counters into `GpuRegistries`.
  - Landed (step 2): move revision/generation bump rules behind `GpuRegistries` mutation helpers (register/update/unregister).
  - Landed (step 3): co-locate registry mutations + bind-group cache invalidation in `GpuResources` to make the change chain explicit.
  - Landed (step 4): route resource reads and bind-group preparation through `GpuResources` APIs (fields are private; no call-site map poking).
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/gpu_registries.rs` (`GpuRegistries`)
    - `crates/fret-render-wgpu/src/renderer/gpu_resources.rs` (`GpuResources`)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::gpu_resources`)
    - `crates/fret-render-wgpu/src/renderer/resources.rs` (register/update/unregister call sites)
    - `crates/fret-render-wgpu/src/renderer/render_scene/bind_groups.rs` (`ensure_*_for_*` calls)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (scene encoding cache key uses generations)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-050 Stage 5: extract scene encoding cache as an explicit subsystem.
  - Goal: make the encode-cache ownership (`key/cache/scratch`) reviewable and keep the allocation-reuse semantics stable.
  - Landed (step 1): move cache bookkeeping into `SceneEncodingCache` and update call sites in `render_scene/execute`.
  - Landed (step 2): move the cache hit/miss paths + perf accounting into a single helper to keep behavior drift-free.
  - Landed (step 3): isolate cache key construction + encoding acquisition helpers in a dedicated `render_scene` module.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/scene_encoding_cache.rs` (`SceneEncodingCache`)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::scene_encoding_cache`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/encoding_cache.rs` (`build_scene_encoding_cache_key`, `acquire_scene_encoding_for_frame`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call sites)
    - `crates/fret-render-wgpu/src/renderer/tests.rs` (cache busts on text quality changes)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-060 Stage 6: extract debug postprocess selection into a dedicated helper.
  - Goal: keep debug-only plan mutations localized while preserving existing degrade/budget semantics and perf counters.
  - Landed (step 1): move debug postprocess selection to `Renderer::pick_debug_postprocess`.
  - Landed (step 2): move plan compilation + tracing/perf accounting behind a single helper.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/debug_postprocess.rs` (`pick_debug_postprocess`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
    - `crates/fret-render-wgpu/src/renderer/render_scene/plan_compile.rs` (`compile_render_plan_for_scene`)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-070 Stage 7: extract per-frame plan dispatch into a dedicated helper.
  - Goal: isolate command-encoder / frame-target lifetime + pass recording loop to keep `render_scene_execute` orchestrative.
  - Landed (step 1): move pass recording + encoder finish + intermediate release tracking into `Renderer::dispatch_render_plan`.
  - Landed (step 2): move render-space uniform packing + upload behind a dedicated helper.
  - Landed (step 3): move per-frame geometry uploads (instances/paints/vertices + quad vertex bases) behind a single helper.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/dispatch.rs` (`dispatch_render_plan`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
    - `crates/fret-render-wgpu/src/renderer/render_scene/render_space_upload.rs` (`upload_render_space_uniforms_for_plan`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/uploads.rs` (`upload_frame_geometry`)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-080 Stage 8: extract effect pipeline setup into a dedicated helper.
  - Goal: isolate per-plan pass scanning + `ensure_*` effect pipelines + capacity ensures to keep `execute` linear.
  - Landed (step 1): move effect pipeline selection + scale params/render space capacity behind `Renderer::ensure_effect_pipelines_for_plan`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/effect_pipelines.rs` (`ensure_effect_pipelines_for_plan`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-090 Stage 9: extract per-frame uniform uploads + bind group preparation into a dedicated helper.
  - Goal: keep uniform/clips/masks uploads + bind-group prep drift-free while further linearizing `execute`.
  - Landed (step 1): move uniform/clips/masks capacity + writes + bind-group prep behind `Renderer::upload_frame_uniforms_and_prepare_bind_groups`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/frame_bindings.rs` (`upload_frame_uniforms_and_prepare_bind_groups`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-100 Stage 10: extract render plan diagnostics and perf reporting into a dedicated helper.
  - Goal: keep plan degradation accounting + segment report drift-free while reducing `execute` surface area.
  - Landed (step 1): move render plan perf counters, segment report update, and plan dump behind `Renderer::record_render_plan_diagnostics_for_frame`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/plan_reporting.rs` (`record_render_plan_diagnostics_for_frame`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-110 Stage 11: extract frame perf aggregation + snapshot into a dedicated helper.
  - Goal: isolate the “frame perf → aggregated perf + last-frame snapshot” bookkeeping while keeping accounting drift-free.
  - Landed (step 1): move SVG/memory snapshots, aggregated perf accumulation, and `last_frame_perf` construction behind `Renderer::finalize_frame_perf_after_dispatch`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/perf_finalize.rs` (`finalize_frame_perf_after_dispatch`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-120 Stage 12: extract per-frame text + svg preparation into dedicated helpers.
  - Goal: keep tracing/perf accounting stable while making `execute` a linear “prepare → encode → compile → upload → dispatch” driver.
  - Landed (step 1): move text prepare + atlas snapshot behind `Renderer::prepare_text_for_frame`.
  - Landed (step 2): move SVG ops prepare + perf snapshot behind `Renderer::prepare_svg_for_frame`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/frame_prepare.rs` (`prepare_text_for_frame`, `prepare_svg_for_frame`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call sites)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-130 Stage 13: extract frame-level pipeline ensures into a dedicated helper.
  - Goal: isolate “ensure core pipelines + compute path MSAA samples” without changing tracing/perf semantics.
  - Landed (step 1): move `ensure_*` pipeline calls and path-samples selection behind `Renderer::ensure_frame_pipelines_and_path_samples`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/frame_pipelines.rs` (`ensure_frame_pipelines_and_path_samples`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-140 Stage 14: extract per-frame perf initialization into a dedicated helper.
  - Goal: keep upload/ingest pending counters initialization drift-free while further linearizing `execute`.
  - Landed (step 1): move per-frame perf initialization behind `Renderer::begin_frame_perf_collection`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/frame_perf_init.rs` (`begin_frame_perf_collection`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (call site)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-150 Stage 15: reuse a renderer-owned scratch buffer for per-frame RenderSpace uniform uploads.
  - Goal: avoid per-frame heap allocations for RenderSpace uniform bytes without changing per-pass RenderSpace semantics.
  - Landed (step 1): store a `Vec<u8>` scratch buffer on `Renderer` and reuse it in `upload_render_space_uniforms_for_plan`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/render_space_upload.rs` (`upload_render_space_uniforms_for_plan`)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::render_space_bytes_scratch`)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-160 Stage 16: reuse a renderer-owned scratch buffer for per-frame viewport uniform uploads.
  - Goal: avoid per-frame heap allocations for viewport uniform bytes without changing uniform stride/layout semantics.
  - Landed (step 1): add `UniformResources::write_viewport_uniforms_into` and reuse a `Renderer`-owned scratch buffer.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/uniform_resources.rs` (`write_viewport_uniforms_into`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/frame_bindings.rs` (call site)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::viewport_uniform_bytes_scratch`)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-170 Stage 17: reuse renderer-owned scratch for plan quad vertices/bases used by non-fullscreen quad passes.
  - Goal: avoid per-frame heap allocations for plan-derived quad vertices and per-pass `base` indices.
  - Landed (step 1): build vertices + bases into `Renderer` scratch, upload once, and return the bases `Vec` to the caller so it can be
    returned to the renderer after dispatch.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/quad_vertices.rs` (`build_plan_quad_vertices_into`, `upload_plan_quad_vertices`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/execute.rs` (returns bases to scratch after dispatch)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::{plan_quad_vertices_scratch,plan_quad_vertex_bases_scratch}`)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-180 Stage 18: reuse renderer-owned scratch for per-frame segment pass counts + segment report.
  - Goal: avoid per-frame heap allocations in perf-only render-plan diagnostics while keeping segment-drift metrics stable.
  - Landed (step 1): store scratch vectors on `Renderer` and swap the segment report with the last-frame report for reuse.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_scene/plan_reporting.rs` (`record_render_plan_diagnostics_for_frame`)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::{render_plan_*_scratch}`)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

- [x] REN-VNEXT-refactor-190 Stage 19: reuse renderer-owned scratch for RenderPlan JSON dump bytes.
  - Goal: avoid per-dump allocation when `FRET_RENDERPLAN_DUMP*` is enabled (debugging/tracing runs).
  - Landed (step 1): serialize JSON into a `Renderer`-owned `Vec<u8>` via `serde_json::to_writer_pretty`.
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/render_plan_dump.rs` (`maybe_dump_render_plan_json`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/plan_reporting.rs` (call site)
    - `crates/fret-render-wgpu/src/renderer/mod.rs` (`Renderer::render_plan_dump_bytes_scratch`)
  - Gates:
    - `cargo test -p fret-render-wgpu --lib`
    - `cargo nextest run -p fret-render-wgpu --test clip_path_conformance --test mask_image_conformance --test composite_group_conformance --test viewport_surface_metadata_conformance`

## M7 — Post-v1 semantic expansions (deferred backlog)

These items are intentionally *not* part of the vNext refactor’s v1 closure. They are common UI
renderer semantics that are either missing today or only available via approximation. Track them
here as a backlog so we can spin them into dedicated workstreams (3-doc format: purpose/TODO/
milestones) when implementation begins.

- [x] REN-VNEXT-sem-010 Path fill paint surface: allow `SceneOp::Path` to accept `Paint` (gradients/materials).
  - Current: `SceneOp::Path` is solid-only (`color: Color`).
  - Risk: binding surface + key space growth; needs bounded, capability-gated fallbacks (wasm/mobile).
  - Tracking: `docs/workstreams/path-paint-surface-v1.md`
- [x] REN-VNEXT-sem-020 General path stroke: introduce a “stroke arbitrary vector paths” surface with bounded stroke style.
  - Landed via: `PathStyle::StrokeV2(StrokeStyleV2)` for vector path preparation + `SceneOp::Path` rendering.
  - Evidence: `docs/workstreams/path-stroke-style-v2.md`, `crates/fret-core/src/vector_path.rs`.
- [x] REN-VNEXT-sem-030 `StrokeStyleV2`: join/cap/miter + dash semantics (and constant-px stroke width semantics as an explicit follow-up).
  - Landed: join/cap/miter + dash for vector path strokes (deterministic, scale-aware).
  - Deferred: constant-px stroke width under non-uniform transforms (requires a transform-aware contract).
  - Tracking: `docs/workstreams/path-stroke-style-v2.md`.
  - Evidence:
    - `crates/fret-core/src/vector_path.rs` (`StrokeStyleV2`, `PathStyle::StrokeV2`)
    - `crates/fret-render-wgpu/src/renderer/path.rs` (`build_dashed_lyon_path`, `tessellate_path_commands`)
    - `crates/fret-render-wgpu/tests/path_stroke_style_v2_conformance.rs`
- [x] REN-VNEXT-sem-040 Sweep/conic gradient (bounded): add `Paint::SweepGradient`.
  - Contract: `docs/adr/0280-sweep-gradient-paint-v1.md`
  - Evidence:
    - `crates/fret-core/src/scene/paint.rs` (`SweepGradient`, sanitize)
    - `crates/fret-core/src/scene/{validate.rs,fingerprint.rs}`
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/{quad.rs,path.rs,text.rs}`
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`paint_eval*`, `paint_eval_{fill,border}`)
    - `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs` (`gpu_sweep_gradient_smoke_conformance`)
  - Gates:
    - `python3 tools/check_layering.py`
    - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
    - `cargo test -p fret-render-wgpu --test paint_gradient_conformance`
- [x] REN-VNEXT-sem-050 Blend modes v2 (bounded): expand `BlendMode` beyond `Over/Add/Multiply/Screen`.
  - Landed (v2 fixed-function subset): `Darken`, `Lighten`, `Subtract`.
  - Contract: `docs/adr/0281-compositing-blend-modes-v2-bounded.md`
  - Evidence:
    - `crates/fret-core/src/scene/composite.rs` (`BlendMode` + `pipeline_index`)
    - `crates/fret-render-wgpu/src/renderer/pipelines/composite.rs` (`blend_state_for_mode`)
    - `crates/fret-render-wgpu/tests/composite_group_conformance.rs` (`gpu_composite_group_blend_modes_v2_smoke_conformance`)
  - Gates:
    - `python3 tools/check_layering.py`
    - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
    - `cargo test -p fret-render-wgpu --test composite_group_conformance`
  - Guardrail: keep the enum small and portable; do not mirror the full CSS list without evidence.
- [x] REN-VNEXT-sem-060 Text paint expansion: gradient/material text, text outline/stroke, and/or text shadow semantics.
  - Status (2026-02-18): v1 landed for painted text fills (solid + gradients) and a bounded text shadow surface.
    - Landed (v1): `SceneOp::Text` carries `paint: Paint` with bounded, deterministic degradations.
    - Landed (v1): GPU readback conformance gate for text gradient paint.
    - Landed (v1): `SceneOp::Text.shadow: Option<TextShadowV1>` (single layer, no blur) for portable text shadows.
    - Landed (adoption): ui-gallery probe uses `Paint::LinearGradient` for text.
    - Landed (prep): unified paint→GPU encoding helper (quad/path/text) with explicit material policy
      (text/path still deterministically degrade materials to a solid base color).
    - Landed (v1): `TextOutlineV1` contract + wgpu (mask+subpixel) implementation + conformance; adoption landed via ui-gallery probe.
    - Tracking workstream (outline/stroke):
      - `docs/workstreams/text-outline-stroke-surface-v1.md`
      - `docs/workstreams/text-outline-stroke-surface-v1-todo.md`
      - `docs/workstreams/text-outline-stroke-surface-v1-milestones.md`
      - Audit is recorded (atlas is coverage, not SDF/MSDF): `docs/workstreams/text-outline-stroke-surface-v1-todo.md` (`TOS-audit-010`)
  - Tracking: `docs/workstreams/text-paint-surface-v1.md` (purpose/TODO/milestones)
  - ADRs:
    - `docs/adr/0279-text-paint-surface-v1.md`
    - `docs/adr/0283-text-shadow-surface-v1.md`
  - Evidence:
    - `crates/fret-core/src/scene/mod.rs` (`SceneOp::Text.shadow`, `TextShadowV1`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/paint.rs` (`paint_to_gpu`, `PaintMaterialPolicy`)
    - `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/text.rs` (`encode_text` renders shadow layer; material still degrades)
    - `crates/fret-core/src/scene/mod.rs` (`TextOutlineV1`, `SceneOp::Text { outline }`)
    - `crates/fret-render-wgpu/src/renderer/pipelines/text.rs` (outline pipeline variant)
    - `crates/fret-render-wgpu/tests/text_outline_conformance.rs` (`gpu_text_outline_v1_renders_a_visible_ring_for_mask_glyphs`)
    - `apps/fret-ui-gallery/src/ui/previews/pages/editors/text/outline_stroke.rs`
    - `crates/fret-render-wgpu/tests/text_paint_conformance.rs` (`gpu_text_shadow_v1_renders_a_separate_layer`)
  - Gates:
    - `cargo nextest run -p fret-render-wgpu --test text_paint_conformance`
    - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
- [x] REN-VNEXT-sem-070 Pattern/tile semantics: support `TileMode::{Repeat,Mirror}` for gradients.
  - Landed (v1): `TileMode::{Repeat,Mirror}` is preserved in `Paint` + `Mask` sanitization and implemented in WGSL
    gradient evaluation (linear/radial/sweep) via a deterministic tiling function.
  - Evidence:
    - `crates/fret-core/src/scene/paint.rs` (`Paint::sanitize` preserves `tile_mode`)
    - `crates/fret-core/src/scene/mask.rs` (`Mask::sanitize` preserves `tile_mode`)
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`gradient_tile_mode_apply`, gradient eval uses it)
    - `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs` (repeat/mirror smoke)
    - `crates/fret-render-wgpu/tests/mask_gradient_conformance.rs` (repeat/mirror smoke)
  - Gates:
    - `cargo nextest run -p fret-render-wgpu --test paint_gradient_conformance --test mask_gradient_conformance`
    - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`
- [x] REN-VNEXT-sem-080 Wider color spaces: support `ColorSpace::Oklab` (and verify portability).
  - Landed (v1): `ColorSpace::Oklab` is preserved in sanitization and used for gradient stop interpolation
    (linear/radial/sweep paints). Masks preserve the enum for forward-compatibility (mask gradients are alpha-only today).
  - Evidence:
    - `crates/fret-core/src/scene/paint.rs` (`Paint::sanitize` preserves `color_space`)
    - `crates/fret-core/src/scene/mask.rs` (`Mask::sanitize` preserves `color_space`)
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`paint_mix_colorspace`, Oklab conversions)
    - `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs` (Oklab midpoint vs sRGB/linear)
  - Gates:
    - `cargo nextest run -p fret-render-wgpu --test paint_gradient_conformance`
    - `cargo test -p fret-render-wgpu shaders_validate_for_webgpu`

- [x] REN-VNEXT-sem-090 Backdrop warp/refraction (bounded): add a “backdrop warp” effect step to enable
      true liquid-glass style distortion (displacement + optional chromatic aberration), with deterministic
      degradation on wasm/mobile.
  - Tracking:
    - `docs/workstreams/renderer-effect-backdrop-warp-v1.md`
    - `docs/workstreams/renderer-effect-backdrop-warp-v1-todo.md`
    - `docs/workstreams/renderer-effect-backdrop-warp-v1-milestones.md`
  - Evidence:
    - `docs/adr/0284-backdrop-warp-effect-step-v1.md`
    - `crates/fret-render-wgpu/tests/effect_backdrop_warp_conformance.rs`
    - `tools/diag-scripts/liquid-glass-backdrop-warp-steady.json`
    - `apps/fret-examples/src/liquid_glass_demo.rs`
  - Extension (v2, texture-driven warp field):
    - Status: landed (conformance + perf baseline recorded)
    - Tracking:
      - `docs/workstreams/renderer-effect-backdrop-warp-v2.md`
      - `docs/workstreams/renderer-effect-backdrop-warp-v2-todo.md`
      - `docs/workstreams/renderer-effect-backdrop-warp-v2-milestones.md`
    - Evidence:
      - `docs/adr/0285-backdrop-warp-effect-step-v2-texture-field.md`
      - `crates/fret-render-wgpu/tests/effect_backdrop_warp_v2_conformance.rs`
      - `tools/diag-scripts/liquid-glass-backdrop-warp-v2-steady.json`
      - `docs/workstreams/perf-baselines/policies/liquid-glass-backdrop-warp-v2-steady.v1.json`
      - `docs/workstreams/perf-baselines/liquid-glass-backdrop-warp-v2-steady.windows-rtx4090.v1.json`
      - `apps/fret-examples/src/liquid_glass_demo.rs`

- [x] REN-VNEXT-sem-100 Drop shadow (blur-based, bounded): add a general drop shadow effect step for
      non-text content (cards/popovers), with deterministic degradation and perf gates.
  - Tracking:
    - `docs/workstreams/renderer-drop-shadow-effect-v1.md`
    - `docs/workstreams/renderer-drop-shadow-effect-v1-todo.md`
    - `docs/workstreams/renderer-drop-shadow-effect-v1-milestones.md`
  - Evidence:
    - `docs/adr/0286-drop-shadow-effect-step-v1.md`
    - `crates/fret-core/src/scene/mod.rs` (`EffectStep::DropShadowV1`)
    - `crates/fret-render-wgpu/src/renderer/pipelines/drop_shadow.rs`
    - `crates/fret-render-wgpu/src/renderer/shaders.rs` (`DROP_SHADOW_*`)
    - `crates/fret-render-wgpu/tests/effect_drop_shadow_v1_conformance.rs`
    - `tools/diag-scripts/drop-shadow-v1-steady.json`
    - `docs/workstreams/perf-baselines/drop-shadow-v1-steady.windows-rtx4090.v1.json`
    - `tools/perf/diag_drop_shadow_v1_gate.ps1`

- [x] REN-VNEXT-sem-110 Clip-path + mask closure: keep rect scissor fast paths hot, and make slow-path
      clip/mask intermediates cacheable and WebGPU-uniformity-safe (no divergent sampling hazards).
  - Tracking:
    - `docs/workstreams/renderer-clip-mask-closure-v1.md`
    - `docs/workstreams/renderer-clip-mask-closure-v1-todo.md`
    - `docs/workstreams/renderer-clip-mask-closure-v1-milestones.md`
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/clip_path_mask_cache.rs`
    - `tools/perf/headless_clip_mask_stress_gate.py`
    - `docs/workstreams/perf-baselines/clip-mask-stress-headless.windows-local.v1.json`

## Always-run guardrails (before/after each milestone)

- [~] REN-VNEXT-guard-001 Keep `python3 tools/check_layering.py` green for all intermediate steps.
  - Last run: 2026-02-16, commit `246030f3`.
- [~] REN-VNEXT-guard-002 Add/extend at least one renderer conformance test per new contract.
  - Status: satisfied through M5 (sampling hints gate) and stroke v2 (join/cap/dash conformance gate landed).
- [~] REN-VNEXT-guard-003 Record a perf snapshot baseline and keep “worst bundles” attachable to milestones.
  - Last capture: 2026-02-15, commit `c4f08adb`.
  - Evidence: `docs/workstreams/renderer-vnext-fearless-refactor-v1-milestones.md` (Perf snapshot record).
- [~] REN-VNEXT-guard-004 Keep a cheap headless perf gate green (stable counters).
  - Gate: `python3 tools/perf/headless_svg_atlas_stress_gate.py`
  - Baseline: `docs/workstreams/perf-baselines/svg-atlas-stress-headless.windows-local.v1.json`
  - Landed: 2026-02-15, commit `49181551`.
- [~] REN-VNEXT-guard-005 Keep external texture imports perf baselines from regressing.
  - Motivation: renderer refactors (uniformity/variants/pipelines) must not silently degrade the imported-frame contract path
    (`RenderTargetId` + `SceneOp::ViewportSurface`), especially on wasm/WebGPU where copies can dominate.
  - Tracking: `docs/workstreams/external-texture-imports-v1.md` (see `EXT-web-perf-131`).
  - Gate (web copy path): `tools/diag-scripts/external-texture-imports-web-copy-perf-steady.json`
  - Baseline: `docs/workstreams/perf-baselines/external-texture-imports-web-copy.web-local.v1.json` (recorded 2026-02-15).
  - Landed observability (2026-02-16):
    - `RenderTargetMetadata.requested_ingest_strategy` (requested) and `RenderTargetMetadata.ingest_strategy`
      (effective) are surfaced in renderer perf snapshots for both:
      - `render_target_updates_ingest_*` (pre-render update churn), and
      - `viewport_draw_calls_ingest_*` (draw-side attribution).
    - `render_target_updates_ingest_fallbacks` counts requested != effective at update time (best-effort).
    - This is a *best-effort* diagnostic signal: it does not change ingest behavior yet.
  - Next (v2): use these counters in perf baselines to enforce that any fallback-only path stays within budget.
