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
- [ ] REN-VNEXT-webgpu-004 If perf evidence warrants, add bounded `MaterialTileMode` pipeline variants:
  - Note: “tile mode” here refers to the material-kind selector channel in the quad shader (see `material_eval`),
    not gradient tile modes (which are sanitized/degraded today).
  - Decision rule (do not do this without evidence):
    - Only proceed if material work is measurably hot in at least one reproducible bundle:
      - `fretboard diag perf` (or a GPU profiler) shows the quad fragment shader is a top hotspot and `material_eval` dominates.
    - And the current bounded variants are insufficient (confirmed by one of):
      - `material_sampled_quad_ops` is high relative to `quad_draw_calls` in the headless gate’s `headless_renderer_perf_materials`
        output and wall time regresses in `fret-quad-material-stress` on the same machine.
      - or real app perf snapshots show unacceptable regression under WebGPU with no alternative mitigation.
  - Guardrail: keep the key space small and observable in perf snapshots (`pipeline_switches_*`), and update headless baselines
    if (and only if) the added variants are justified.
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

## Always-run guardrails (before/after each milestone)

- [~] REN-VNEXT-guard-001 Keep `python3 tools/check_layering.py` green for all intermediate steps.
  - Last run: 2026-02-15, commit `c4f08adb`.
- [~] REN-VNEXT-guard-002 Add/extend at least one renderer conformance test per new contract.
  - Status: satisfied through M5 (sampling hints conformance gate landed).
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
