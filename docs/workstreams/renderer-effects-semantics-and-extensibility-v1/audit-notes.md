---
title: Renderer Effects Semantics + Extensibility v1 (Audit Notes)
status: draft
date: 2026-02-25
scope: renderer, effects, shaders, diagnostics, extensibility
---

# Audit notes (fearless refactor targets)

This note captures “what looks solid”, “what looks risky”, and a small set of landable next steps
for extending the renderer’s effect semantics without turning the backend into a one-off fork.

## What looks solid (keep / build on)

- **Effect stack is bounded and deterministic**: `EffectChain` is fixed-size and effect parameters
  are sanitized at the contract layer, which keeps budgeting decisions predictable.
- **Plan-based execution is an extensibility-friendly shape**: effect compilation happens before
  recording, so budgeting/degradation can be deterministic and traceable.
- **Masking model is explicit**: effect passes already distinguish clip-based masking vs mask-texture
  masking (`mask_uniform_index` vs `mask`), which is the right abstraction to extend safely.
- **Color management direction is correct**: effects and compositing are treated as linear-space
  operations, and output transfer is explicit for non-sRGB display formats (ADR 0040 / ADR 0117).

## Refactor hazards (current pain points)

- **`shaders.rs` is a monolith**:
  - large diff surface and higher risk of accidental regressions in unrelated shaders.
  - hard to review “small” shader changes (a new effect shader becomes a big file edit).
- **Effect pipeline boilerplate is duplicated**:
  - many passes repeat the same “unmasked / masked / mask texture” structure.
  - recorder logic repeats bind-group creation patterns and perf accounting.
- **Degradation diagnostics are under-specified at the effect granularity**:
  - we track plan-level degradations, but it’s still hard to answer “which effect degraded and why”
    for a given frame without reading the compiled plan dump.
- **Intermediate precision is format-driven, not intent-driven**:
  - some combinations (non-sRGB 8-bit output + explicit output transfer) can introduce an extra
    quantization step in linear space.

## Wasm/WebGPU compatibility notes (CustomV2 focus)

- **WGSL validation needs a WebGPU/Tint backstop**:
  - native validation via Naga is necessary but not sufficient for wasm, because the browser WebGPU
    implementation validates WGSL via Tint.
  - Guardrail: `crates/fret-render-wgpu/src/renderer/tests.rs` includes stitched CustomV1/V2 WGSL modules
    in both `shaders_validate_for_webgpu` (Naga/WebGPU rules) and the optional browser
    `wasm-webgpu-tests` Tint compile gate.
- **Ensure Naga WGSL parsing is available on wasm32 builds**:
  - Custom effect registration parses and validates stitched WGSL via `naga::front::wgsl`.
  - `fret-render-wgpu` must enable `naga`'s `wgsl-in` feature so wasm32 builds can compile.
  - Evidence: `crates/fret-render-wgpu/Cargo.toml`.
- **Capability gating is conservative and likely portable**:
  - `RendererCapabilities.custom_effect_v2_user_image` is based on `Rgba8Unorm` being filterable and usable
    as a sampled texture, which should hold on most WebGPU adapters.
  - Apps should still treat this as a runtime probe and fall back to CustomV1/no-op deterministically.
- **Not related, but worth remembering**:
  - WebGPU `ExternalTexture` import is currently reported as unsupported on wasm32 in
    `crates/fret-render-wgpu/src/capabilities.rs` (wgpu web backend limitation as of wgpu 28).

## Landable next steps (recommended)

P1 — Quality and maintainability:

- Extract a shared “fullscreen effect pass” helper for:
  - pipeline selection (unmasked vs masked vs mask texture),
  - bind-group creation (texture + params + optional mask),
  - perf accounting (uniform bytes and pass counters).
  Done (initial): `record_fullscreen_param_effect_pass` + `record_fullscreen_texture_effect_pass`.
  Migrated passes (so far): `ColorAdjust`, `AlphaThreshold`, `ColorMatrix`, `Dither`, `Noise`, `DropShadow`.
  Anchors: `crates/fret-render-wgpu/src/renderer/render_scene/recorders/effects.rs`,
  `crates/fret-render-wgpu/src/renderer/pipelines/*`.

- Split WGSL sources into `*.wgsl` files and `include_str!()` them:
  - keep per-pass shaders adjacent to their pipeline module.
  - reduce merge conflicts and make shader diffs reviewable.
  Done (initial): moved effect shaders (blit + explicit sRGB encode, drop shadow, dither, noise) into
  `crates/fret-render-wgpu/src/renderer/pipelines/wgsl/*.wgsl` and `include_str!()` them from `shaders.rs`.
  Anchor: `crates/fret-render-wgpu/src/renderer/shaders.rs`.

P1 — Visual correctness:

- Rounded-rect edge coverage should not “thin out” at corners (a common artifact when AA width is derived from
  `fwidth` for diagonals and when border coverage is computed by subtracting two AA-ed shapes).
  - Fix: use an isotropic gradient AA estimate for SDF coverage and compute border coverage via multiplication
    (`alpha_outer * (1 - alpha_inner)`).
  - Evidence: `crates/fret-render-wgpu/src/renderer/shaders.rs`.
  - Regression harness: `tools/diag-scripts/liquid-glass-custom-v2-corners-screenshot.json`.

P1/P2 — Diagnostics:

- Add per-effect degradation counters (requested vs applied) into `RenderPerfSnapshot`:
  - budget zero, insufficient budget, target exhaustion (per effect family).
  - optional: include “applied downsample scale” summaries for blur/drop shadow.
  Done (initial): `RenderPerfSnapshot.effect_degradations` with per-family `requested/applied` + counters.
  Done (initial): `RenderPerfSnapshot.effect_blur_quality` with downsample/iteration summaries for blur/shadow.
  Anchors: `crates/fret-render-wgpu/src/renderer/types.rs`,
  `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`,
  `crates/fret-render-wgpu/src/renderer/render_scene/plan_reporting.rs`.

P2 — Higher-end visuals without core bloat:

- **Optional float intermediate for explicit output transfer**:
  - when output transfer is required, render into a float intermediate (e.g. `RGBA16Float`)
    if budgets and adapter capabilities permit, then encode once to the output.
  - keep deterministic fallback to the current unorm intermediate.
  Anchors: `crates/fret-render-wgpu/src/renderer/frame_targets.rs`,
  `crates/fret-render-wgpu/src/renderer/render_plan_compiler.rs`.

- **Bounded custom effect ABI (wgpu-only MVP)**:
  - versioned, fixed bind shapes (params-only; params + 1 catalog texture; params + 1 user texture).
  - explicit cost model so the plan can reject/degrade deterministically.
  - capability discovery surfaced through the renderer context (not ad-hoc env vars).
  - close semantics with a stable `render_space` meaning (effect-local origin/size) and a small deterministic
    renderer-owned pattern atlas for grain/dither-like recipes (no user textures in v1).
  Anchors: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/todo.md`,
    `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`.

## “Missing semantics” candidates (intentionally deferred)

- Variable-length effect graphs (beyond `EffectChain::MAX_STEPS`): valuable, but contract-heavy.
  Prefer bounded custom effects first.
- End-to-end HDR / wide-gamut correctness: requires a broader contract and platform integration.
  Keep hooks (`RenderTargetColorEncoding`) but defer full pipeline work.
