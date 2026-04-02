---
title: Rounded-Rect Shadow Scene Primitive v1
status: Proposed
date: 2026-04-01
---

# ADR 0318: Rounded-Rect Shadow Scene Primitive v1

## Context

Fret currently has two different shadow mechanisms with different ownership:

- `fret-ui::element::ShadowStyle` (ADR 0060) for box/container chrome and theme-token-driven
  component elevation,
- `EffectStep::DropShadowV1(DropShadowV1)` (ADR 0286) for effect-owned, content-derived blur under
  `EffectMode::FilterContent`.

That split is intentional and remains correct. The problem is the **current default implementation**
of `ShadowStyle`:

- `crates/fret-ui/src/paint.rs` expands each logical shadow layer into many `SceneOp::Quad`
  operations,
- softness is approximated in the UI layer instead of represented semantically in the scene,
- renderer quality/perf work is therefore constrained by a UI-side fallback representation.

Recent shadow work fixed the alpha-budget drift in that fallback, but it did not remove the deeper
architectural limitation: box shadow is still not a first-class scene primitive.

ADR 0030 already pointed in this direction explicitly:

- `fret-core` should stay semantic,
- shadows should become first-class primitives rather than a `Quad` flag,
- renderers should remain free to implement those semantics using analytic SDF or other techniques.

Pinned GPUI/Zed review reinforces that direction:

- API-level box shadow values lower into a dedicated scene primitive,
- the renderer owns the dedicated shadow pipeline,
- and rounded-rect shadow coverage is evaluated in shader space instead of being approximated by
  many CPU-expanded quads.

We want the same architectural shape in Fret without collapsing `ShadowStyle` into ADR 0286's
effect-owned `DropShadowV1`.

## Goals

- Add a first-class scene primitive for rounded-rect box shadows.
- Keep `ShadowStyle` stable as the authoring/mechanism contract for container chrome.
- Let renderers implement shadow quality in renderer space rather than UI-layer quad expansion.
- Keep deterministic degradation for backends that do not yet implement the primitive.

## Non-goals

- Replacing or broadening `DropShadowV1`.
- CSS `filter: drop-shadow(...)` parity.
- Inner shadows or arbitrary path shadows.
- Policy-level preset mapping (`shadow-sm/md/lg`) in `crates/*`.

## Decision

Add a first-class rounded-rect shadow scene operation to `fret-core::scene::SceneOp`.

Provisional contract shape:

- `SceneOp::ShadowRRect { order, rect, corner_radii, offset, spread, blur_radius, color }`

Field semantics:

- `order`: draw ordering token, following the existing non-semantic `DrawOrder` posture.
- `rect`: the base rounded-rect geometry in logical/layout space.
- `corner_radii`: outer corner radii for the base rounded rect.
- `offset`: shadow translation in logical pixels (pre-scale-factor).
- `spread`: signed logical-pixel expansion/deflation applied before blur.
- `blur_radius`: bounded logical-pixel blur radius.
- `color`: solid shadow color in the existing `Color` contract space.

The primitive is **single-layer**. Multi-layer box shadows remain a higher-level composition story:

- `ShadowStyle.primary` lowers to one shadow op,
- `ShadowStyle.secondary` lowers to a second shadow op when present.

`DropShadowV1` remains a separate mechanism for content-derived shadows under an effect scope and is
**not** the generic lowering target for `ShadowStyle`.

## Semantics (v1)

### 1. Geometry

The primitive represents a rounded-rect box shadow derived from the base `rect` and
`corner_radii`.

The effective source geometry is computed as:

1. apply `spread` to the base rect,
2. expand/deflate `corner_radii` by the same amount and clamp radii at `0`,
3. translate the result by `offset`.

If spread-deflation collapses width or height to `<= 0`, the shadow produces no visible output and
renderers may skip it deterministically.

### 2. Blur

`blur_radius` is expressed in logical pixels and is bounded by the contract.

The exact sampling strategy is renderer-owned, but the semantic outcome is:

- opacity falls off outward from the spread-adjusted rounded rect,
- the shadow remains visually associated with the rounded-rect geometry rather than a plain
  axis-aligned box.

### 3. Ordering

`SceneOp::ShadowRRect` is an ordinary draw primitive:

- it participates in scene order like `Quad`, `Image`, and `Text`,
- it is typically emitted before the container background/border quad,
- it does not implicitly draw the underlying fill or border.

### 4. Clip, transform, opacity, layer, and mask interaction

The primitive obeys the same stack semantics as other scene draw ops:

- current transform stack,
- current opacity stack,
- current layer stack,
- current clip stack,
- current mask/composite/effect scopes.

This ADR does not introduce shadow-specific clip or transform rules.

### 5. Coordinate space

All geometry inputs are expressed in logical/layout space, matching `Quad` and `ShadowStyle`.
Renderers apply device scaling as part of their normal scene encoding path.

## Deterministic degradation (ordered)

Backends MUST NOT silently reinterpret `SceneOp::ShadowRRect` as `DropShadowV1`.

If a backend does not implement a dedicated soft shadow path, degradation is:

1. preferred fallback: replay a bounded rounded-rect quad approximation,
2. if only a harder fallback is viable: render a single spread-adjusted rounded rect with no blur,
3. if geometry is empty after sanitization/deflation: skip.

The degradation must remain deterministic for a given backend/capability set.

## Contract notes

### Why this is not `DropShadowV1`

`DropShadowV1` is defined as:

- a content-derived shadow,
- requiring an effect scope and FilterContent intermediate,
- bounded by explicit effect computation bounds.

`SceneOp::ShadowRRect` is instead:

- geometry-derived,
- not effect-owned,
- intended for ordinary container chrome and theme-token-driven box shadows.

Those are different mechanism surfaces and should remain separate.

### Why this is not a `Quad` flag

Treating shadow as an independent primitive keeps:

- renderer batching/options open,
- clip/transform evolution explicit,
- and box-shadow semantics reviewable without inflating `Quad` itself.

This matches ADR 0030's posture for first-class shadow primitives.

## Consequences

Pros:

- `ShadowStyle` can remain stable while renderer quality improves.
- One logical shadow layer becomes one semantic scene op instead of many quads.
- The renderer can implement a dedicated shadow pipeline with better fidelity and saner perf tradeoffs.

Cons:

- Adds a new scene primitive that all renderers/fallback paths must understand.
- Requires contract, validation, fingerprinting, and conformance work before implementation is
  complete.

## Relationship to Existing ADRs

- ADR 0030: this ADR realizes the “shadows become a first-class primitive” direction for
  rounded-rect box shadows.
- ADR 0060: `ShadowStyle` remains the portable authoring contract, but its default rendering path is
  no longer assumed to be UI-layer multi-quad expansion forever.
- ADR 0286: `DropShadowV1` remains the explicit blur effect step for effect-owned, content-derived
  shadows and is not superseded by this ADR.

## Tracking

- Workstream: `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/DESIGN.md`
- TODO: `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/shadow-renderer-primitive-fearless-refactor-v1/MILESTONES.md`
