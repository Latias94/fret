---
title: Custom Effect V2 (TODO)
status: draft
date: 2026-02-26
scope: renderer, effects, extensibility, abi
---

# TODO (ordered)

## P0 — Decide the “one extra input” story

- [x] Decision locked: add a single **user-provided image texture** input referenced by `ImageId`.
  - Rationale: unlocks LUT/noise/normal-map recipes without growing a renderer-owned catalog into an implicit “asset system”.
  - Boundedness: exactly one extra sampled image (+ sampler) with fixed bind shape; no resource tables in v2.
  - See: `docs/adr/0300-custom-effect-v2-user-image-input.md` and `README.md`.

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
