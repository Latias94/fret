---
title: Custom Effect V2 (TODO)
status: draft
date: 2026-02-26
scope: renderer, effects, extensibility, abi
---

# TODO (ordered)

## P0 — Decide the “one extra input” story

- [ ] Pick one (and only one) additional input to unlock most high-end recipes:
  - Option A: a single **user texture** input (bound via an existing portable handle such as `MaterialId`), or
  - Option B: a single **renderer-owned catalog** input that can be extended over time (noise/LUT atlas), or
  - Option C: a “small resource table” (bounded length, e.g. up to 2 textures) with strict capability gating.

Constraints:

- Must work under existing budgeting semantics (no implicit allocations).
- Must be expressible in `fret-core::EffectStep` without leaking backend handles.

## P1 — Versioned ABI and capability discovery

- [ ] Define `CustomEffectDescriptorV2` + `EffectStep::CustomV2 { ... }` shape.
- [ ] Extend renderer capabilities to report supported custom effect shapes.
- [ ] Add plan reporting fields (shape + pass count + scratch usage summary).

## P2 — Implementation and conformance

- [ ] Implement v2 registry + pipeline/cache key generation bump (similar to CustomV1).
- [ ] Add conformance tests:
  - effect reads user texture deterministically under scissor,
  - chain padding + clip coverage semantics remain correct,
  - deterministic degradation paths under budget exhaustion.

## P3 — Ecosystem authoring ergonomics

- [ ] Provide `fret-ui-kit` helper(s) for registering and caching CustomV2 programs.
- [ ] Provide “recipe templates” for:
  - acrylic (blur + tint + grain),
  - cyberpunk postprocess (scanlines + chromatic + vignette),
  - glass highlight overlay (separate chrome layer, not in the postprocess shader).

