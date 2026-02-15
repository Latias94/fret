---
title: Image Sampling Hints v1
status: Draft
date: 2026-02-15
---

# ADR 0276: Image Sampling Hints v1

## Context

Fret is a self-drawn UI framework that targets:

- native (wgpu),
- wasm/WebGPU,
- and mobile GPUs.

Image sampling choices (nearest vs linear) are a common portability/perf pressure point:

- they are visually meaningful (pixel-art / canvas / masks vs typical UI content),
- they can split batching (sampler state differences),
- and they must not introduce backend-specific surprises.

We want a minimal, bounded contract that:

- keeps the renderer mechanism surface small,
- preserves strict in-order semantics,
- and yields deterministic behavior under capability limits.

## Decision

### D1 — Add a minimal sampling hint enum (v1)

Introduce `ImageSamplingHint` with a strictly bounded state surface:

- `Default`
- `Linear`
- `Nearest`

`Default` is renderer-chosen, but v1 renderers should treat it as equivalent to `Linear` unless
explicitly documented otherwise.

No address mode controls are added in v1 (clamp-to-edge remains the baseline).
No mip/anisotropy controls are added in v1.

### D2 — Hints live on image sampling sites, not on `Paint`/`Material`

Sampling hints are attached to the ops/values that *actually sample a texture*:

- `SceneOp::Image { sampling, .. }`
- `SceneOp::ImageRegion { sampling, .. }`
- `SceneOp::MaskImage { sampling, .. }`
- `Mask::Image { sampling, .. }` (mask-image sources)

This keeps `Paint` and `MaterialId` as the controlled extensibility seam, without turning them into
a high-entropy “everything can configure everything” surface.

### D3 — Deterministic fallbacks are mandatory

Renderers must implement deterministic fallbacks:

- If `Nearest` is unsupported (capability, feature policy, or backend limitation), it must degrade
  to `Default` (v1: effectively `Linear`) deterministically.
- Degradation must not reorder draws. Only sampler/bind-group selection may change.

## Contract (v1)

### Filtering

- `Nearest`: nearest-neighbor sampling.
- `Linear`: linear sampling.
- `Default`: renderer-chosen; v1 should match `Linear` unless otherwise stated.

### Addressing

v1 does not expose address mode:

- baseline is clamp-to-edge for image sampling.

### Scope

Sampling hints apply only to image texture sampling.
They do not affect:

- geometry, scissoring, or clipping,
- hit-testing semantics,
- or the public `DrawOrder` contract (scene op order remains authoritative).

## Renderer requirements

- Sampling hints must be applied without introducing new ordering semantics.
- State splits caused by hints must be bounded and observable (e.g. bind-group switches).
- Backends should prefer a small set of sampler objects (e.g. one linear + one nearest per relevant
  bind-group layout), rather than an unbounded sampler configuration space.

## Conformance gates

At least one GPU readback conformance test must cover:

- nearest vs linear producing measurably different output for the same source image,
- and mixed-primitive ordering invariants when sampling modes differ.

Evidence (wgpu default renderer):

- `crates/fret-render-wgpu/tests/image_sampling_hint_conformance.rs`

## Alternatives considered

### A1 — Put sampling controls on `Paint` / `MaterialId`

Rejected for v1:

- would increase the high-entropy state surface of `Paint`,
- encourages per-op sampler configuration (batching hazards),
- and blurs the mechanism vs policy boundary (defaults belong in ecosystem policy layers).

### A2 — Put hints on a renderer-only “image style” layer

Deferred:

- likely useful long-term, but v1 wants the smallest contract change that unblocks correctness and
  portability.

## Consequences

- The public scene contract grows by one minimal enum and a small number of fields on image/mask
  sampling sites.
- Renderer implementations must be careful to avoid unbounded sampler permutations and to keep
  sampler selection deterministic across backends.

