---
title: Renderer Effects Semantics + Extensibility v1 (TODO)
status: draft
date: 2026-02-25
scope: renderer, effects, caching, portability, diagnostics
---

# TODO

This TODO is ordered by implementation priority (P0 first), and is designed to be landable in small, reviewable PRs.

## P0 — Correctness and contract closure

- [x] Fix `SceneEncodingCacheKey` to include all encode-time inputs that change output:
  - [x] Add a `materials_generation` (or equivalent) to the key so material register/unregister cannot reuse stale encodes.
  - [x] Include encode-time budgets and relevant renderer knobs in the key.
  - [x] Extend miss reasons with new key fields, and surface them in perf snapshots.
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encoding_cache.rs`,
    `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/paint.rs`.

- [x] Make blur radius semantics real:
  - [x] Consume `EffectStep::GaussianBlur.radius_px` in plan compilation.
  - [x] Consume `DropShadowV1.blur_radius_px` (in addition to downsample) and map to the shared blur primitive.
  - [x] Define deterministic degradation behavior when the requested radius is too expensive under budgets.
  - Evidence: `crates/fret-core/src/scene/mod.rs`, `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`.

- [x] Decide and implement `EffectStep::Dither` behavior in effect chains:
  - Implemented: ordered Bayer 4x4 dithering in effect chains (portable, deterministic).
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`.

- [x] Avoid compounding clip coverage across multi-step effect chains:
  - Apply clip/mask coverage only on the final step of a chain to avoid `clip^2` edge darkening
    (e.g. blur → custom refraction with rounded clips).
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`,
    `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` (unit test).

## P1 — Consistency (color, intermediates, diagnostics)

- [x] Document and enforce intermediate color rules:
  - [x] Intermediates are treated as linear storage; effects/compositing remain linear (ADR 0040 / ADR 0117).
  - [x] Non-sRGB 8-bit outputs (`Rgba8Unorm` / `Bgra8Unorm`) use a single final output blit that applies an explicit
        sRGB transfer, avoiding “encoded intermediates”.
  - [x] Add a targeted conformance test for the explicit output transfer path.
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_plan_compiler.rs`,
    `crates/fret-render-wgpu/src/renderer/render_plan.rs`,
    `crates/fret-render-wgpu/src/renderer/pipelines/blit.rs`,
    `crates/fret-render-wgpu/src/renderer/shaders.rs`,
    `crates/fret-render-wgpu/tests/output_srgb_transfer_conformance.rs`.

- [x] Reduce boilerplate for fullscreen effects (recorder dedupe):
  - Extract reusable helpers for “unmasked / masked / mask texture” variants.
  - Migrate existing effect recorders to the helpers (including `DropShadow`).
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs`.

- [x] Split effect WGSL sources into `*.wgsl` files:
  - Keep effect shader sources reviewable and reduce `shaders.rs` merge conflict risk.
  - Evidence: `crates/fret-render-wgpu/src/renderer/shaders.rs`,
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/*.wgsl`.

- [x] Add `EffectStep::NoiseV1` (bounded procedural grain) for acrylic/glass recipes:
  - [x] Deterministic, bounded noise evaluation in effect chains (no hidden time sources).
  - [x] Conformance: scissored FilterContent noise preserves outside-region content and is deterministic.
  - Evidence: `crates/fret-core/src/scene/mod.rs`,
    `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`,
    `crates/fret-render-wgpu/src/renderer/pipelines/noise.rs`,
    `crates/fret-render-wgpu/src/renderer/shaders.rs`,
    `crates/fret-render-wgpu/tests/effect_filter_content_noise_conformance.rs`.

- [x] Unify blur implementation into a shared “blur primitive” module:
  - [x] Single place that maps `(radius_px, quality, budgets, viewport_size)` → downsample + iteration strategy.
  - [x] Shared ping-pong blur pass emission helper reused by `GaussianBlur` and `DropShadow`.
  - Evidence: `crates/fret-render-wgpu/src/renderer/blur_primitive.rs`,
    `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`.

- [x] Improve diagnostics for degradations:
  - [x] Add per-effect degradation counters (requested/applied + budget zero/insufficient/target exhausted).
  - [x] Add blur/shadow “applied quality” summaries (downsample + iteration stats) into perf snapshots.
  - [x] Degrade drop shadow under tight budgets to a deterministic hard shadow (no blur) instead of skipping.
  - [x] Degrade gaussian blur deterministically: fall back to single-scratch blur, then no-op (tracked in quality summary).
  - Evidence: `crates/fret-render-wgpu/src/renderer/types.rs`,
    `crates/fret-render-wgpu/src/renderer/render_plan.rs`,
    `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`,
    `crates/fret-render-wgpu/src/renderer/render_scene/plan_reporting.rs`.

## P1.5 — Vector path semantics closure (paint + dash + MSAA)

These are common “editor-grade UI” needs that often become a long-tail source of visual mismatch
if left unspecified.

- [x] Document the current path paint limitation and make it diagnosable:
  - Historically `SceneOp::Path` encoding degraded `Paint::Material` to a solid base color.
  - Keep a perf counter so any remaining deterministic degradations are visible (unknown id, budgets, etc.).
  - Evidence (current behavior): `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/path.rs`.
  - Evidence (perf counter): `crates/fret-render-wgpu/src/renderer/types.rs`,
    `crates/fret-render-wgpu/src/renderer/render_scene/encode/state.rs`,
    `crates/fret-render-wgpu/src/renderer/render_scene/perf_finalize.rs`.

- [x] Support material/texture paints for `SceneOp::Path` (wgpu backend):
  - Allow `PaintMaterialPolicy::Allow` for paths and implement material evaluation in the path shader.
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_scene/encode/draw/path.rs`,
    `crates/fret-render-wgpu/src/renderer/shaders.rs` (`PATH_SHADER`),
    `crates/fret-render-wgpu/tests/path_material_paint_conformance.rs`.

- [x] Dash semantics consistency:
  - `StrokeRRect` dashes are evaluated in the quad shader using an rrect-perimeter parameterization,
    while `PathStyle::StrokeV2` uses CPU-side dash splitting before tessellation.
  - Write down the expected semantics for `DashPatternV1` (units, phase origin/direction, scale-factor behavior)
    and keep a targeted conformance test that compares rrect vs path outcomes for a “rect-like path” shape.
  - Evidence (current implementations): `crates/fret-render-wgpu/src/renderer/shaders.rs` (rrect),
    `crates/fret-render-wgpu/src/renderer/path.rs` (path dashes).
  - Evidence (semantics): `docs/adr/0271-stroke-rrect-and-dashed-borders-v1.md`,
    `docs/adr/0277-path-stroke-style-v2.md`.
  - Evidence (conformance): `crates/fret-render-wgpu/tests/dash_semantics_rrect_vs_path_conformance.rs`.

- [x] Path MSAA correctness on Vulkan:
  - Default behavior matches GPUI: enable path MSAA when the target format supports resolves.
  - Escape hatch: set `FRET_DISABLE_VULKAN_PATH_MSAA=1` to force the non-MSAA path pipeline if a driver
    misbehaves.
  - Evidence (config): `crates/fret-render-wgpu/src/renderer/config.rs`.
  - Evidence (MSAA pass semantics): `crates/fret-render-wgpu/src/renderer/render_scene/recorders/path_msaa.rs`.
  - Evidence (default + opt-out visibility): `crates/fret-render-wgpu/tests/vulkan_path_msaa_visibility_conformance.rs`.
  - Evidence (multi-pass composite smoke): `crates/fret-render-wgpu/tests/path_msaa_composite_vulkan.rs`.

## P2 — Extensibility (bounded custom effects)

- [x] Design a capability-gated custom effect extension point (wgpu-only first):
  - Fixed, versioned bind shapes with strict limits.
  - Clear layering: core contract stays small; ecosystem provides “recipes” and installation helpers.
  - Design doc: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-abi-wgpu-mvp.md`.

- [x] Implement CustomV1 (wgpu-only MVP):
  - `fret-core`: `EffectId`, `EffectParamsV1`, `EffectStep::CustomV1` + fingerprint/validate.
  - `fret-render-wgpu`: registry + `effects_generation` + cache key inclusion.
  - `fret-render-wgpu`: `RenderPlanPass::CustomEffect` + recorder/executor support.
  - Demo: `apps/fret-examples/src/custom_effect_v1_demo.rs` (wired via `apps/fret-demo`).
  - Conformance: `crates/fret-render-wgpu/tests/effect_custom_v1_conformance.rs`.

- [x] Close CustomV1 semantics (“escape hatch with a ceiling”):
  - [x] Define the stable WGSL contract surface (required entrypoint + premul rules + `render_space`):
    - `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v1-semantics.md`
  - [x] Ensure `render_space` is effect-local for CustomV1 (origin/size match the effect bounds scissor):
    - `crates/fret-render-wgpu/src/renderer/render_scene/helpers.rs`
    - Conformance: `crates/fret-render-wgpu/tests/effect_custom_v1_conformance.rs`
  - [x] Provide a renderer-owned deterministic pattern atlas for v1 recipes (no user textures):
    - WGSL helpers: `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_*_part_a.wgsl`
    - Upload: `crates/fret-render-wgpu/src/renderer/gpu_textures.rs`
    - Conformance: `crates/fret-render-wgpu/tests/effect_custom_v1_conformance.rs`
  - [x] Optimize “padded blur → final CustomV1” so clip/mask coverage is applied exactly once:
    - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` (unit test)

- [x] Implement deterministic chain padding for sampling-extending effects:
  - Use `EffectStep::CustomV1.max_sample_offset_px` and warp/blur bounded parameters to expand earlier step scissors
    when later steps may sample outside their output pixel.
  - Keep clip/mask coverage applied exactly once (final commit back into `srcdst`) to avoid `clip^2` darkening.
  - Evidence: `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`,
    `crates/fret-render-wgpu/tests/effect_custom_v1_conformance.rs`.

- [ ] Add a “theme-like postprocess” demo to validate the CustomV1 ceiling (policy-only, no core changes):
  - [x] Implement and wire: `apps/fret-examples/src/postprocess_theme_demo.rs` (via `apps/fret-demo`).
  - [ ] Add a `fretboard diag` script that captures a small, shareable baseline bundle (screenshots + perf snapshot).

- [x] Keep stitched effect shaders WebGPU/Tint-valid (uniformity + bindings):
  - Ensure masked variants evaluate clip coverage before any non-uniform early returns so SDF AA derivatives remain
    Tint-valid on WebGPU.
  - Evidence: `crates/fret-render-wgpu/src/renderer/shaders.rs`,
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/*_masked_part_b.wgsl`,
    `crates/fret-render-wgpu/src/renderer/tests.rs` (`shaders_validate_for_webgpu`).

## Suggested regression gates

- Unit tests:
  - `cargo nextest run -p fret-render-wgpu` (where feasible) for cache-key correctness and blur radius mapping.
- Determinism checks:
  - A small diag bundle + script that exercises blur/shadow at multiple radii and asserts stable outcomes under budgets.

## Follow-ups (quality/perf)

- [ ] Optional higher-precision intermediate for output transfer:
  - [ ] When output transfer is required, allow rendering into a float intermediate (e.g. `RGBA16Float`) under budgets,
        then encode once to `Rgba8Unorm` / `Bgra8Unorm`.
  - Rationale: avoid the extra unorm quantization step in linear space for display-referred non-sRGB outputs.

- [x] CustomV2 ceiling bump (bounded):
  - [x] Lock the “one extra input” story (chosen: single user image input via `ImageId`):
    - `docs/adr/0300-custom-effect-v2-user-image-input.md`
    - `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v2/README.md`
  - [x] Versioned ABI + capability discovery + conformance exist (wgpu backend).
  - Follow-ups: demo-oriented authoring templates + WebGPU/wasm runtime story tracked under
    `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v2/README.md`.
