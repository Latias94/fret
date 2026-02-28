---
title: Custom Effect V3 (Renderer-provided sources)
status: draft
date: 2026-02-28
scope: renderer, effects, extensibility, abi, budgeting
---

# Custom Effect V3 (Renderer-provided sources)

CustomV1 (ADR 0299) and CustomV2 (ADR 0300) intentionally stay small: a single-pass effect with a fixed parameter
payload and (in v2) one optional user image input.

This is a strong baseline for bounded extensibility, but the “high fidelity liquid glass” ceiling often needs:

- access to both a **raw** (unmodified) backdrop and a **processed** (blurred/adjusted) backdrop, and/or
- a bounded **blur pyramid** so custom shaders can sample different blur scales without embedding heavy blur loops.

CustomV3 is the next bounded ceiling bump: it introduces **renderer-provided sources** (raw + optional pyramid)
with explicit budgeting and deterministic degradation.

## Status

- M0 (dual-source `src_raw`): implemented in `fret-core` + `fret-render-wgpu` with conformance coverage.
- M1 (bounded pyramid `src_pyramid`): implemented in `fret-render-wgpu` with plan dump reporting and conformance.
- M2 (sharing/caching): deferred (requires an explicit mechanism-level design).

## Diagnostics vocabulary (wgpu)

CustomV3 source outcomes are visible in two places:

- RenderPlan dump summaries (per-effect): requested vs distinct/aliased raw, and requested vs degraded pyramid.
- Per-frame perf counters (aggregate): `RenderPerfSnapshot.effect_degradations.custom_effect_v3_sources.*`.

Counters:

- `raw_requested`, `raw_distinct`, `raw_aliased_to_src`
- `pyramid_requested`, `pyramid_applied_levels_ge2`
- `pyramid_degraded_to_one_budget_zero`, `pyramid_degraded_to_one_budget_insufficient`

## Design anchor

Normative contract:

- `docs/adr/0301-custom-effect-v3-renderer-provided-sources.md`

This folder tracks the fearless refactor plan to land that ADR in the wgpu backend while keeping WebGPU/WASM
constraints visible.

## Proposed shape (summary)

- New portable step: `EffectStep::CustomV3 { ... }`
- Fixed bind shape includes:
  - `src` (current chain input),
  - `src_raw` (chain root),
  - `src_pyramid` (mipped raw; optional, bounded),
  - `user0` and optional `user1` (portable `ImageId` inputs).

## Sequencing (recommended)

1) **M0: dual-source only** (`src_raw`):
   - land `src_raw` plumbing + conformance without introducing pyramid generation.
2) **M1: bounded pyramid** (`src_pyramid`):
   - add mip-chain allocation + deterministic downsample/upsample strategy under budgets.
3) **M2: optional sharing** (deferred):
   - consider group-level reuse/caching once the per-layer pyramid path is correct and diagnosable.

## Implementation hazards (wgpu)

- `src_raw` must be a **read-only** source. If the chain’s destination texture is also the chain root, the backend
  cannot sample `srcdst` while writing to it in the same pass; it must either:
  - evaluate the final pass into a different target and then composite back, or
  - preserve a scratch copy of the raw chain root.
  If neither is possible under budgets/targets, V3 must deterministically alias `src_raw == src` and report it.

- The pyramid (when requested) is expected to be derived from `src_raw` and may require additional scratch targets.
  Degrade deterministically to `levels = 1` when budgets are insufficient.

## Deliverables

- ADR implementation in `fret-core` + `fret-render-wgpu`.
- Capability discovery for v3 subfeatures (raw, pyramid, user image count).
- Render-plan dump additions (explicit reporting of pyramid levels and degradation reasons).
- Conformance tests:
  - raw-vs-src correctness,
  - pyramid sampling determinism (level dimensions and sampling clamps),
  - deterministic degradation under budget pressure.
- Authoring demo templates (apps only; no ecosystem recipes in this workstream).
