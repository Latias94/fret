---
title: Custom Effect V2 (TODO)
status: draft
date: 2026-02-26
scope: renderer, effects, extensibility, abi
---

# TODO (ordered)

## P0 — Decide the “one extra input” story

- [x] Decision locked: add a single **user-provided image texture** input referenced by `ImageId`.
  - Rationale: unlocks LUT/noise/normal-map recipes without growing a renderer-owned catalog into an implicit “asset system”.
  - Boundedness: exactly one extra sampled image (+ sampler) with fixed bind shape; no resource tables in v2.
  - See: `docs/adr/0300-custom-effect-v2-user-image-input.md` and `README.md`.

Constraints:

- Must work under existing budgeting semantics (no implicit allocations).
- Must be expressible in `fret-core::EffectStep` without leaking backend handles.

## P1 — Versioned ABI and capability discovery

- [x] Define `CustomEffectDescriptorV2` + `EffectStep::CustomV2 { ... }` shape.
  - Evidence: `crates/fret-core/src/effects.rs`, `crates/fret-core/src/scene/mod.rs`.
- [x] Extend renderer capabilities to report supported custom effect shapes.
  - Evidence: `crates/fret-render-wgpu/src/capabilities.rs` (`RendererCapabilities.custom_effect_v2_user_image`).
- [ ] Add plan reporting fields (shape + pass count + scratch usage summary).
  - Note: current plan reporting covers effect pass counts and degradation; it does not yet emit a per-effect ABI shape
    summary for custom effects.

## P2 — Implementation and conformance

- [x] Implement v2 registry + pipeline/cache key generation bump (similar to CustomV1).
  - Evidence: `crates/fret-render-wgpu/src/renderer/services.rs`,
    `crates/fret-render-wgpu/src/renderer/pipelines/custom_effect.rs`,
    `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs`.
- [x] Add conformance tests:
  - effect reads user texture deterministically under scissor,
  - chain padding + clip coverage semantics remain correct,
  - deterministic degradation paths under budget exhaustion.
  - Evidence: `crates/fret-render-wgpu/tests/effect_custom_v2_conformance.rs`.
- [x] Extend WebGPU/WGSL guardrails to cover stitched CustomV1/V2 modules (not just built-in shaders).
  - Evidence: `crates/fret-render-wgpu/src/renderer/tests.rs`.
- [x] Make masked effect shaders WebGPU/Tint-uniformity-safe for clip SDF derivatives:
  - Root cause: `dpdx`/`dpdy`/`fwidth` must be called from uniform control flow on WebGPU (Tint validation).
  - Fix: compute `clip_alpha(...)` before any non-uniform early returns / bounds checks in masked fragment shaders.
  - Evidence: `crates/fret-render-wgpu/src/renderer/shaders.rs`,
    `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/*_masked_part_b.wgsl`,
    `crates/fret-render-wgpu/src/renderer/tests.rs` (`shaders_validate_for_webgpu`).
- [x] Allow CustomV2 user WGSL to use derivatives on WebGPU:
  - Remove non-uniform early returns prior to calling `fret_custom_effect(...)` (replace bounds checks with clamped
    sampling + a final `select(...)`).
  - Add a small “derivatives smoke” custom effect that compiles under Tint (browser WebGPU).
  - Evidence:
    - `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v2_*_part_b.wgsl`
    - `crates/fret-render-wgpu/src/renderer/tests.rs` (`CUSTOM_EFFECT_DERIVATIVES_SMOKE_WGSL`)

## P3 — Ecosystem authoring ergonomics

- [x] Provide `fret-ui-kit` helper(s) for registering and caching CustomV2 programs.
  - Evidence: `ecosystem/fret-ui-kit/src/custom_effects.rs` (`CustomEffectProgramV2`).
- [x] Turn the web demo into a small inspector-style parameter harness so effect authors can verify that each
  contract field is wired correctly (sampling, `UvRect`, blur radius/downsample, strength/tint, mode/quality,
  rounded clips).
  - Evidence: `apps/fret-examples/src/custom_effect_v2_web_demo.rs`.
- [ ] Provide demo-oriented “authoring templates” (in `apps/fret-examples/`), not ecosystem recipes:
  - an identity/starter CustomV2 (register + params + input image),
  - [x] a LUT color grade example:
    - Evidence: `apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs` (web runner via `apps/fret-demo-web`)
  - a simple “glass chrome” highlight driven by a normal/noise map input.
