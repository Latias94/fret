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

## M3 — Intermediate color rule + conformance

Exit criteria:

- A written rule exists (linear intermediates recommended).
- Effect passes behave consistently with the rule.
- At least one targeted test/diag gate catches sRGB/linear mismatches for a representative effect chain.

## M4 — Bounded custom effect design (wgpu-only MVP)

Exit criteria:

- A design for a bounded, capability-gated custom effect extension point exists and is reviewed.
- A minimal MVP can render one custom effect (e.g. “glass tint + subtle blur + warp”) without touching core contracts.
- Budgeting/degradation is deterministic and diagnosable.

