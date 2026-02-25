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

- [ ] Unify blur implementation into a shared “blur primitive” module:
  - [ ] Single place that maps `(radius_px, quality, budgets, viewport_size)` → passes + downsample strategy.
  - [ ] Shared degradation counters + reasons used by `GaussianBlur`, `DropShadow`, and future effects.

- [x] Improve diagnostics for degradations:
  - [x] Add per-effect degradation counters (requested/applied + budget zero/insufficient/target exhausted).
  - [x] Add blur/shadow “applied quality” summaries (downsample + iteration stats) into perf snapshots.
  - [x] Degrade drop shadow under tight budgets to a deterministic hard shadow (no blur) instead of skipping.
  - Evidence: `crates/fret-render-wgpu/src/renderer/types.rs`,
    `crates/fret-render-wgpu/src/renderer/render_plan.rs`,
    `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`,
    `crates/fret-render-wgpu/src/renderer/render_scene/plan_reporting.rs`.

## P2 — Extensibility (bounded custom effects)

- [ ] Design a capability-gated custom effect extension point (wgpu-only first):
  - [ ] Fixed, versioned bind shapes (params-only; params + 1 catalog texture; params + 1 user texture) with strict limits.
  - [ ] Explicit cost model + budgeting hooks so the plan can reject/degrade deterministically.
  - [ ] Clear layering: core contract stays small; ecosystem can provide “recipes” that map to the extension.
  - Non-goal: arbitrary user-provided WGSL in core without a bounded ABI and capability gates.

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
