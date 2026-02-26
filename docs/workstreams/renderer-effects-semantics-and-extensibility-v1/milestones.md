---
title: Renderer Effects Semantics + Extensibility v1 (Milestones)
status: draft
date: 2026-02-25
scope: renderer, effects, caching, portability, diagnostics
---

# Milestones

Each milestone is intended to be shippable on its own. “Done” means tests + diagnostics evidence exist.

## M0 — Baseline inventory

Exit criteria:

- Documented the affected contract surfaces and code anchors (done in `README.md`).
- A minimal “smoke” scenario exists (manual or diag) that uses:
  - `GaussianBlur`
  - `DropShadowV1`
  - `Backdrop` mode effect
  - `NoiseV1`
  - Evidence: `crates/fret-render-wgpu/tests/effect_backdrop_acrylic_recipe_conformance.rs`.

## M1 — Scene encoding cache correctness

Exit criteria:

- `SceneEncodingCacheKey` includes:
  - material registry generation (or equivalent),
  - encode config key (budgets + relevant knobs),
  - updated miss reasons and surfacing.
- A regression test demonstrates that changing budgets or registering/unregistering a material invalidates the cache.

## M2 — Blur radius semantics closure

Exit criteria:

- `GaussianBlur.radius_px` affects plan compilation and output.
- `DropShadowV1.blur_radius_px` affects plan compilation and output.
- Deterministic degradation rules are defined and observable in perf/diagnostics.

## M2.1 — Chain clip coverage semantics

Exit criteria:

- Clip/mask coverage is applied exactly once for multi-step effect chains (final step only), preventing `clip^2`
  edge darkening.
- A unit test locks this behavior for representative chains (e.g. blur → custom effect).

## M3 — Intermediate color rule + conformance

Exit criteria:

- A written rule exists (linear intermediates recommended).
- Effect passes behave consistently with the rule.
- Output transfer behavior is explicit and deterministic for non-sRGB 8-bit output formats:
  - render into an extra color intermediate (`PlanTarget::Intermediate3`),
  - apply a single final explicit sRGB transfer blit when writing to `Rgba8Unorm` / `Bgra8Unorm`.
- At least one targeted test/diag gate catches regressions (explicit output transfer + representative effects).

## M4 — Bounded custom effect design (wgpu-only MVP)

Exit criteria:

- A design for a bounded, capability-gated custom effect extension point exists and is reviewed.
- A minimal MVP can render one custom effect (e.g. “glass tint + subtle blur + warp”) without touching core contracts.
- Budgeting/degradation is deterministic and diagnosable.
 - Evidence: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-abi-wgpu-mvp.md`.
