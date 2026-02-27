---
title: Custom Effect V2 (Milestones)
status: draft
date: 2026-02-26
scope: renderer, effects, extensibility, abi
---

# Milestones

## M0 — Decision locked

Exit criteria:

- [x] One v2 binding shape is chosen (with rationale) and written down:
  - A single user-provided image input referenced by `ImageId` (plus `UvRect` + `ImageSamplingHint`).
  - Evidence: `README.md`, `docs/adr/0300-custom-effect-v2-user-image-input.md`.
- [x] Capability discovery shape is specified (what does the app learn at runtime?).
  - Current shape: `RendererCapabilities.custom_effect_v2_user_image` (wgpu backend).
  - Evidence: `crates/fret-render-wgpu/src/capabilities.rs`.

## M1 — Core surface + backend skeleton

Exit criteria:

- [x] `fret-core` has versioned `CustomV2` surfaces (types + validation + fingerprint mixing).
  - Evidence: `crates/fret-core/src/effects.rs`, `crates/fret-core/src/scene/mod.rs`,
    `crates/fret-core/src/scene/validate.rs`, `crates/fret-core/src/scene/fingerprint.rs`.
- [x] `fret-render-wgpu` has a registry skeleton + cache key inclusion and can compile an identity CustomV2.
  - Evidence: `crates/fret-render-wgpu/src/renderer/services.rs`,
    `crates/fret-render-wgpu/src/renderer/pipelines/custom_effect.rs`,
    `crates/fret-render-wgpu/src/renderer/render_scene/effect_pipelines.rs`.

## M2 — Extra input works (the “ceiling bump”)

Exit criteria:

- [x] The chosen extra input is usable from WGSL and is capability-gated.
  - Evidence: WGSL prelude `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/custom_effect_v2_*`,
    gate `crates/fret-render-wgpu/src/renderer/services.rs` (`register_custom_effect_v2`).
- [x] Conformance tests cover:
  - determinism for fixed params + inputs,
  - scissor/mask correctness,
  - degradation under budgets.
  - Evidence: `crates/fret-render-wgpu/tests/effect_custom_v2_conformance.rs`.

## M3 — Example demos (apps)

Exit criteria:

- [x] A demo page exists (gallery or `fret-examples`) and has a scripted diagnostics bundle for regressions.
  - Evidence: `apps/fret-examples/src/liquid_glass_demo.rs`,
    `tools/diag-scripts/liquid-glass-custom-v2-corners-screenshot.json`.
- [x] Add at least one additional “authoring template” demo that uses the v2 input image (e.g. LUT/noise/normal-map)
  and documents the expected input color space and sampling hints.
  - Evidence: `apps/fret-examples/src/custom_effect_v2_web_demo.rs`, `apps/fret-demo-web/src/wasm.rs`.
