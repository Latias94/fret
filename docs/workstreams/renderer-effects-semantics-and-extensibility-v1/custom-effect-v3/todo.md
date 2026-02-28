---
title: Custom Effect V3 (TODO)
status: draft
date: 2026-02-28
scope: renderer, effects, extensibility, abi, budgeting
---

# TODO (ordered)

This TODO tracks the V3 work as landable steps. It intentionally starts with “dual-source only” to keep risk low.

## P0 — Lock the contract

- [ ] Review ADR 0301 for boundedness, WebGPU constraints, and cache-key implications.
- [ ] Decide the exact portable surface names and sanitization rules:
  - `CustomEffectSourcesV3` fields (raw request + pyramid request),
  - pyramid bounds (`max_levels`, `max_radius_px`) and clamp constants.
- [ ] Add an explicit “degradation vocabulary” for v3 sources in diagnostics:
  - `raw_aliased_to_src`,
  - `pyramid_unavailable`,
  - `pyramid_budget_insufficient`,
  - `user1_unsupported`,
  - etc.

## P1 — M0: Dual-source (`src_raw`) plumbing

- [ ] `fret-core`: add `EffectStep::CustomV3` + fingerprint/validate.
- [ ] `fret-render-wgpu`: preserve chain root for the v3 effect pass, and bind it as `src_raw`.
- [ ] Add conformance test: `src_raw` differs from `src` when a prior step modifies the chain (e.g. blur),
      and the shader can sample both deterministically under scissor/mask.

## P2 — M1: Optional bounded pyramid (`src_pyramid`)

- [ ] Define pyramid generation strategy (bounded, deterministic):
  - fixed max levels,
  - deterministic downsample/upsample passes,
  - stable clamping for sampling outside bounds.
- [ ] `fret-render-wgpu`: implement pyramid allocation and populate mip levels under budgets.
- [ ] Add plan dump reporting:
  - requested vs applied pyramid levels,
  - bytes allocated per pyramid,
  - degradation reasons.
- [ ] Add conformance:
  - correct dimensions per level,
  - deterministic sampling clamps and alias behavior when pyramid is missing.

## P3 — Authoring demos (apps only)

- [ ] Add a minimal “liquid glass v3” demo template that demonstrates:
  - crisp edge refraction from `src_raw`,
  - frosted center from `src` (blurred),
  - optional level-based sampling from `src_pyramid` (when supported).

## Deferred — Group sharing / caching (M2)

- [ ] Define a mechanism-level “glass group” concept (scene-level, not custom WGSL):
  - cache key and invalidation,
  - compositing/ordering semantics,
  - determinism under partial reuse.
- [ ] Only pursue after M0/M1 are stable and diagnosable.

