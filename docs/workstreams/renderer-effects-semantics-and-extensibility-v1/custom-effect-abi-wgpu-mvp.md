---
title: Custom Effect ABI (wgpu-only MVP)
status: draft
date: 2026-02-25
scope: renderer, effects, extensibility, capabilities, budgeting
---

# Custom Effect ABI (wgpu-only MVP)

## Goal

Enable ecosystem/component authors to ship **high-end visual effects** (e.g. acrylic/glass/refraction stacks)
without forking the renderer, while preserving:

- **boundedness**: fixed binding shapes + fixed parameter sizes,
- **determinism**: explicit inputs only (no hidden time), deterministic degradation,
- **diagnosability**: per-effect counters + plan dump visibility,
- **layering**: no `wgpu` leakage into contract crates (`fret-core`, `fret-ui`).

This is explicitly **wgpu-only** for the MVP. Other backends may degrade the effect deterministically.

## Non-goals (v1)

- Unbounded effect graphs or user-provided arbitrary render graphs.
- Backend-agnostic shader portability (this is wgpu-only to start).
- Full HDR/wide-gamut correctness end-to-end.
- Allowing effects to allocate arbitrary GPU resources outside the renderer’s budgeting model.

## Problem statement

The current `EffectChain` covers the core primitives needed to reproduce most “UI standard” looks
(`GaussianBlur`, `ColorAdjust`, `BackdropWarpV2`, `NoiseV1`, etc.). That is great for *replication*.

What’s missing is an **escape hatch with a ceiling**:

- app/ecosystem authors want to author a small fullscreen shader (or a tiny fixed multi-pass bundle),
  while still respecting Fret’s ordering, scissoring, masking, and budgeting semantics.

## Options

### Option A (recommended): core-level `EffectId` + renderer-owned registration service

Add a small, portable handle and param surface in `fret-core`, similar to `MaterialId`:

- `EffectId`: opaque ID (slotmap key) in `fret-core`.
- `EffectParamsV1`: fixed-size params (e.g. 16 floats, like `MaterialParams`).
- `EffectStep::CustomV1 { id: EffectId, params: EffectParamsV1 }`.

Renderers expose a runtime service for registration:

- `EffectService::register_effect(desc) -> EffectId`
- `EffectService::unregister_effect(id) -> bool`

The **renderer** owns:

- pipeline creation,
- any catalog textures it exposes,
- capability gating,
- deterministic fallbacks (no-op / degraded variant / substitute built-in steps).

Why this fits Fret:

- `Scene` remains the single ordering contract.
- The effect is still bounded: fixed params + fixed bind shapes.
- Authors can ship high-end looks as ecosystem policy, but the mechanism stays small.

### Option B: renderer-only custom pass API (no core changes)

Not viable for effects applied via `SceneOp`, because `SceneOp` is defined in `fret-core`.
Anything that needs to be expressed in the display list needs a portable handle.

### Option C: “everything is a material”

Useful for many visual recipes, but insufficient for backdrop sampling / multi-pass cases.
Materials are draw-time; effects are plan-time (ordering + offscreen/backdrop semantics).

## Proposed ABI surface (MVP)

### 1) Fixed binding shapes

The MVP supports **one-pass fullscreen effects** that read from the current effect source
(`src_texture`) and write to the current destination (`dst_texture`), under scissor/mask.

Binding shapes are versioned and strictly limited:

- **Shape v1 (ParamsOnly)**:
  - `src_texture: texture_2d<f32>`
  - `params: uniform` (fixed-size, e.g. 64 bytes)
  - optional: mask binding via the existing renderer mask path (uniform mask image or mask texture)

Future shapes (v2+) may add:

- one renderer-owned catalog texture (e.g. blue noise),
- one user-provided sampled texture (for normal maps / displacement fields),
- explicit sampler controls (bounded).

### 2) Capability gating

Renderers expose support via capabilities:

- `RendererCapabilities.custom_effects: bool`
- supported binding shapes list
- max custom effects per frame / per app (bounded)

### 3) Cost model + deterministic degradation

Custom effects must participate in the `RenderPlan` budgeting model.

MVP rule:

- Custom effect pass uses the same budget gate as other single-scratch in-place passes:
  requires `full * 2` bytes available (source + scratch) unless it can be proven in-place safe.
- On insufficient budget or target exhaustion:
  - degrade deterministically to no-op (tracked in counters),
  - optionally allow a renderer-provided fallback chain (pre-registered).

### 4) Diagnostics hooks

Per-effect counters should answer:

- requested vs applied,
- degraded (budget zero / insufficient / target exhausted),
- applied binding shape + variant.

Plan dumps should include:

- effect ID,
- shape,
- pass count,
- degradation reason.

## Integration points (implementation sketch)

- `fret-core`:
  - add `EffectId` + `EffectParamsV1`
  - extend `EffectStep` with `CustomV1`
  - extend scene fingerprint mixing so cache keys remain correct

- `fret-render-wgpu`:
  - add an effect registry (slotmap) keyed by `EffectId`
  - compile `CustomV1` into a `RenderPlanPass::CustomEffect`
  - executor records the fullscreen pass using the same masking/scissor helpers

## Open questions

- Do we allow custom effects in both `FilterContent` and `Backdrop` modes?
  - MVP: yes, but they operate on the current effect source (content or backdrop).
- Do we allow multi-pass custom bundles?
  - MVP: no (start with one pass). If needed later, it must declare pass count and scratch usage
    so budgeting stays deterministic.

