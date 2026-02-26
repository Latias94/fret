---
title: Custom Effect V2 (High-ceiling, bounded)
status: draft
date: 2026-02-26
scope: renderer, effects, extensibility, abi
---

# Custom Effect V2 (High-ceiling, bounded)

CustomV1 intentionally ships as a small “escape hatch with a ceiling” (single pass, params-only, `src_texture` only).
That is enough for many UI looks, but it is not the end-state for “editor-grade” effects such as:

- acrylic / glass variants that want a noise/LUT/normal-map input,
- stylized post-processing themes (cyberpunk/retro) that want a stable pattern atlas or LUT,
- effect stacks that want a small, fixed multi-pass bundle with an explicit cost model.

This folder tracks a **fearless refactor** path to a CustomV2 ABI that raises the ceiling while keeping the
core contract bounded, budgetable, and capability-gated.

Key invariants:

- No `wgpu` handle leakage into `fret-core` / `fret-ui`.
- Fixed, versioned binding shapes.
- Explicit sampling bounds and predictable scratch usage.
- Deterministic degradation with per-effect counters and plan visibility.

See also:

- CustomV1 contract: `docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v1-semantics.md`
- ADR 0299 (CustomV1 MVP): `docs/adr/0299-custom-effect-abi-wgpu-only-mvp.md`

