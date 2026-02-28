---
title: Custom Effect V3 (TODO)
status: draft
date: 2026-02-28
scope: renderer, effects, extensibility, abi, budgeting
---

# TODO (ordered)

This TODO tracks the V3 work as landable steps. It intentionally starts with “dual-source only” to keep risk low.

## P0 — Lock the contract

- [x] Review ADR 0301 for boundedness, WebGPU constraints, and cache-key implications.
- [x] Decide the exact portable surface names and sanitization rules:
  - `CustomEffectSourcesV3` fields (raw request + pyramid request),
  - pyramid bounds (`max_levels`, `max_radius_px`) and clamp constants.
- [x] Add an explicit “degradation vocabulary” for v3 sources in diagnostics (beyond plan dumps):
  - `raw_aliased_to_src` (counter: `custom_effect_v3_sources.raw_aliased_to_src`)
  - `pyramid_degraded_to_one_budget_zero` / `pyramid_degraded_to_one_budget_insufficient`
  - Note: `user1_unsupported` remains TODO (capability gating vocabulary is not finalized).

- [x] Define cache-key and generation implications:
  - ensure v3 registration generations and any source-prep knobs that affect encode output contribute to
    the scene encoding cache key.

## P1 — M0: Dual-source (`src_raw`) plumbing

- [x] `fret-core`: add `EffectStep::CustomV3` + fingerprint/validate.
- [x] `fret-render-wgpu`: introduce a “source preparation” step for V3:
  - ensure `src_raw` is read-only (cannot alias a render attachment being written this pass),
  - decide whether to preserve a scratch copy or render into an intermediate then composite back,
  - add deterministic degradation when scratch/budgets are insufficient (`src_raw == src`).
- [x] `fret-render-wgpu`: add render-plan reporting for v3 source prep outcomes:
  - whether raw is distinct or aliased,
  - requested/applied pyramid levels,
  - degradation reasons surfaced in plan dumps (counters remain TODO).
- [x] `fret-render-wgpu`: add per-frame counters for v3 source outcomes:
  - requested vs applied raw distinctness,
  - requested vs applied pyramid levels (>=2),
  - deterministic degradation reasons.
- [x] Add conformance test: `src_raw` differs from `src` when a prior step modifies the chain (e.g. blur),
      and the shader can sample both deterministically under scissor/mask.

## P2 — M1: Optional bounded pyramid (`src_pyramid`)

- [x] Define pyramid generation strategy (bounded, deterministic):
  - fixed max levels,
  - deterministic downsample/upsample passes,
  - stable clamping for sampling outside bounds.
- [x] `fret-render-wgpu`: implement pyramid allocation and populate mip levels under budgets.
- [x] Add plan dump reporting:
  - requested vs applied pyramid levels,
  - degradation reasons.
- [x] Add conformance:
  - correct dimensions per level,
  - deterministic sampling clamps and alias behavior when pyramid is missing.

## P3 — Authoring demos (apps only)

- [ ] Add a minimal “liquid glass v3” demo template that demonstrates:
  - crisp edge refraction from `src_raw`,
  - frosted center from `src` (blurred),
  - optional level-based sampling from `src_pyramid` (when supported).

## Deferred — Group sharing / caching (M2)

- [x] Write down a design space and recommended sequence:
  - `m2-sharing-and-caching-design.md`
- [ ] M2.0 (reversible): implement chain-local pyramid reuse (same frame, same `src_raw`, no intervening writes),
      with per-frame counters.
- [ ] M2.1 (contract): propose an explicit scene-level “glass group” primitive (ADR) to share snapshot/pyramid
      across multiple surfaces deterministically.
