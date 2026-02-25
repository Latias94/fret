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

- [ ] Document and enforce intermediate color rules:
  - [ ] Define whether intermediates are always linear (recommended) and how output encoding is handled.
  - [ ] Ensure effect passes are consistent with the rule (blur/color adjust/backdrop warp/composite).
  - [ ] Add a small conformance test or a diag script to catch regressions (sRGB vs linear surprises).

- [ ] Unify blur implementation into a shared “blur primitive” module:
  - [ ] Single place that maps `(radius_px, quality, budgets, viewport_size)` → passes + downsample strategy.
  - [ ] Shared degradation counters + reasons used by `GaussianBlur`, `DropShadow`, and future effects.

- [ ] Improve diagnostics for degradations:
  - [ ] Add per-effect degradation counters (budget zero, insufficient budget, target exhaustion).
  - [ ] Include “requested vs applied” summaries in perf snapshots where applicable.

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
