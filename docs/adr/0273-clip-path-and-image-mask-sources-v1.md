---
title: ClipPath and Image Mask Sources v1
status: Draft
date: 2026-02-13
---

# ADR 0273: ClipPath and Image Mask Sources v1

## Context

Fret’s scene contract currently supports:

- rectangular clip stacks (`PushClipRect`),
- rounded-rect soft clips (`PushClipRRect`),
- a paint-only mask stack with gradient sources (ADR 0239).

Modern UI ecosystems and editor tooling frequently need additional primitives:

1) **Clip paths** (clip-path / arbitrary shape clipping)
   - must affect hit-testing when used as an overflow/visibility mechanism,
   - must compose correctly with transforms and nested clip stacks,
   - must preserve strict in-order semantics.

2) **Image masks** (mask-image)
   - commonly used for fades, spotlights, reveal effects, and stylized visuals,
   - should remain paint-only by default (hit-testing unchanged),
   - must be bounded and degrade deterministically for wasm/mobile and downlevel GPUs.

If these are not standardized, ecosystems will either:

- approximate with many quads (op explosion and drift), or
- escalate to heavy external pipelines for routine UI looks.

## Decision

### D1 — Add a clip-path entry type (v1)

Extend the scene clip stack with a path-based clip:

- `SceneOp::PushClipPath { bounds: Rect, path: PathId }`
- `SceneOp::PopClip`

Notes:

- `bounds` is a computation bound, not an implicit clip; it limits the cost of the clip
  implementation and enables budgeting.
- The clip geometry comes from a prepared path handle (`PathId`), produced by the existing path
  service. The path is interpreted as a filled shape (v1; stroke-based clips are deferred).

### D2 — Extend mask sources with an image mask (v1)

Extend `Mask` with an image-backed source:

- `Mask::Image { image: ImageId, uv: UvRect }`

The image mask is evaluated as a coverage function `m(x, y) in [0, 1]` in the mask’s local
coordinate space. v1 evaluation uses the image’s coverage channel (renderer-defined; recommended:
red for `R8Unorm` masks, alpha for RGBA), clamped to `[0, 1]`.

Sampling policy is intentionally minimal in v1:

- v1 defines linear sampling + clamp addressing as the baseline.
- A future ADR may introduce an explicit sampling hint (nearest/linear/mip) with bounded state
  splits.

## Semantics (normative)

### Ordering and transform interaction

Clip-path entries follow the same ordering and capture rules as other clip entries:

- Clip entries are captured at push time (including the cumulative transform in effect).
- Subsequent transforms do not retroactively move existing clip entries.
- Scene operation order remains authoritative.

### Hit-testing

- `PushClipPath` affects hit-testing in the same way as `PushClipRect/PushClipRRect` when used by
  the UI runtime for overflow clipping: descendants outside the effective clip are not hit-testable.
- `Mask::Image` remains paint-only by default (hit-testing unchanged), consistent with ADR 0239.

### Budgets and deterministic degradation

Clip-path and image masks must be implementable in at least two cost models:

1) shader evaluation (per-pixel coverage),
2) cached mask textures (generate once, reuse for a segment).

Renderers may choose either, but must remain bounded and deterministic:

- Both `PushClipPath.bounds` and `PushMask.bounds` are computation bounds used for budgeting.
- Under budget pressure or missing capabilities, renderers must degrade deterministically:
  - prefer lowering mask resolution / disabling expensive mask generation first,
  - and if a clip-path cannot be represented, fall back to a conservative behavior:
    - recommended v1 fallback: treat `PushClipPath` as `PushClipRect { rect: bounds }`.

Degradation must be visual-only and must not affect layout.

## Consequences

- Ecosystem crates gain a stable path to “clip-path” and “mask-image” style visuals without Tier A
  pipelines.
- Mobile/wasm have explicit, deterministic fallback behavior rather than silent divergence.
- Renderer internals have a clear caching target (mask textures keyed by path/image + transform +
  scale factor + bounds tier).

## Acceptance criteria (recommended gates)

- Add at least one GPU conformance test that:
  - validates clip-before-transform vs transform-before-clip behavior for path clips,
  - validates nested clips (rect/rrect/path) + masks,
  - and validates deterministic degradation under forced budget failure.

## Related

- Rounded clipping and soft clip masks: `docs/adr/0063-rounded-clipping-and-soft-clip-masks.md`
- Scene transform and clip composition: `docs/adr/0078-scene-transform-and-clip-composition.md`
- Mask layers and alpha masks: `docs/adr/0239-mask-layers-and-alpha-masks-v1.md`
- Renderer budgets and deterministic degradation: `docs/adr/0118-renderer-intermediate-budgets-and-effect-degradation-v1.md`

