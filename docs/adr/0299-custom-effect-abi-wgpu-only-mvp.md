---
title: ADR 0299: Custom Effect ABI (wgpu-only MVP)
status: Draft
date: 2026-02-25
---

# ADR 0299: Custom Effect ABI (wgpu-only MVP)

## Context

Fret’s scene contract already exposes bounded, portable postprocessing primitives via `SceneOp::PushEffect` +
`EffectChain` (ADR 0117) and deterministic budgeting/degradation (ADR 0118). This supports high-fidelity UI
replication (shadcn, Material 3) and leaves room for higher-end recipes (glass/acrylic) using built-in steps.

However, ecosystem authors also need an **escape hatch with a ceiling**:

- author a small fullscreen shader to implement a bespoke look,
- without forking the renderer,
- while preserving ordering, scissor/mask semantics, and deterministic degradation,
- and without leaking `wgpu` into contract crates (`fret-core`, `fret-ui`).

This is expected to start as a **wgpu-only** extension point; other backends may degrade deterministically.

## Decision

### 1) Add a bounded custom effect handle to `fret-core`

Introduce a portable, opaque handle:

- `EffectId` (slotmap key, similar to `MaterialId`).

Add a fixed-size parameter payload:

- `EffectParamsV1` (64 bytes; 16 floats) with sanitization (non-finite → 0).

Extend the effect chain surface:

- `EffectStep::CustomV1 { id: EffectId, params: EffectParamsV1 }`.

### 2) Registration is renderer-owned and capability-gated

Renderers expose a runtime registration service (wgpu supports; others may return `Unsupported`):

- `CustomEffectService::register_custom_effect_v1(desc) -> EffectId`
- `CustomEffectService::unregister_custom_effect(id) -> bool`

The renderer owns pipeline creation and performs deterministic validation/fallback behavior.

### 3) MVP constraints (wgpu-only)

The MVP is intentionally small and landable:

- single-pass fullscreen effect,
- params-only (fixed 64B payload),
- no user-provided textures in v1,
- expressed only inside `EffectChain` between `PushEffect`/`PopEffect`,
- deterministic degradation to no-op under budget/target exhaustion (tracked in counters).

### 4) Cache correctness

Because `EffectId` is opaque, changing registration state must invalidate scene encoding caches:

- wgpu backend maintains an `effects_generation` counter,
- it is included in the `SceneEncodingCacheKey`.

## Consequences

- Ecosystem authors can build high-end effects as policy/recipes without expanding the portable core with
  backend-specific shader surfaces.
- The contract stays bounded and testable (conformance tests can register a custom effect and assert scissor,
  ordering, and determinism).
- The approach preserves future extensibility: v2 binding shapes may add a renderer-owned catalog texture or
  a single user texture, but only with versioned, fixed bind shapes and explicit cost models.

## References

- ADR 0117: Effect Layers and Backdrop Filters
- ADR 0118: Renderer Intermediate Budgets and Effect Degradation
- Workstream: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-abi-wgpu-mvp.md`

