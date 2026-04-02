# Shadow Renderer Primitive (Fearless Refactor v1) — Design

Status: Complete (v1 primitive lane landed; future backend-specific upgrades need a follow-on workstream or ADR)

Last updated: 2026-04-02

Related:

- Portable softness follow-on: `docs/workstreams/shadow-portable-softness-fearless-refactor-v1/DESIGN.md`
- Shadow surface closure lane: `docs/workstreams/shadow-surface-fearless-refactor-v1/DESIGN.md`
- Blur effect workstream: `docs/workstreams/renderer-drop-shadow-effect-v1/renderer-drop-shadow-effect-v1.md`
- ADR: `docs/adr/0318-rounded-rect-shadow-scene-primitive-v1.md`

## Context

Status note (2026-04-02): the default `ShadowStyle -> SceneOp::ShadowRRect -> analytic wgpu`
path is now landed, the explicit scene-level quad fallback helper is in place, representative
evidence is recorded, and first-party consumer audits no longer assume the old quad-expanded
representation. Future backend-specific adoption can reuse the scene-level fallback helper without
reopening UI-layer default lowering.

Fret now has three distinct shadow realities:

1. `ShadowStyle` is the stable mechanism surface for container and recipe-owned elevation.
2. `DropShadowV1` is a bounded effect-step for content-derived blur under `EffectMode::FilterContent`.
3. The historical fallback in `crates/fret-ui/src/paint.rs` expands each shadow layer into multiple
   `SceneOp::Quad` operations with alpha falloff.

The recent portable-softness lane corrected the alpha budget problem in the quad expansion path, but
that lane intentionally did not claim that the fallback painter was the final renderer-quality
answer.

Pinned GPUI/Zed review confirms the more modern architecture:

- API-level box shadow values lower into a dedicated scene primitive.
- The renderer keeps shadow evaluation in a dedicated shadow pipeline.
- The shader computes rounded-rect shadow coverage directly instead of approximating it with many
  CPU-expanded quads.

Fret is still pre-open-source, so this is the right time to move the default renderer path toward
that shape and delete the wrong layering once the replacement is proven.

## Problem Statement

The main problem is no longer "our shadow presets are wrong."

The workstream started because **box shadow geometry was hidden inside UI-layer quad expansion
instead of being a first-class scene primitive**.

That default-path structural gap is now closed on the integrated wgpu renderer, but the same root
cause still matters for two remaining reasons:

1. non-native backends still need an explicit, documented degradation lane,
2. the repo still needs durable evidence proving why the primitive path is the correct default.

That created five concrete issues:

1. Quality is capped by the fallback algorithm.
   - Large-radius shadows still read more rectangular or harder than a dedicated shader path.
2. Scene size grows with blur.
   - One logical shadow layer becomes many quads before the renderer even sees it.
3. Renderer optimization freedom is limited.
   - Batching, clipping, transform handling, and future quality upgrades are tied to a UI helper.
4. The wrong shadow mechanisms are easy to conflate.
   - `DropShadowV1` is an effect over content coverage, not a geometric box-shadow primitive.
5. Product surfaces amplify the fallback's artifacts.
   - `todo_demo` showed that once local composition gets heavier, the fallback's straight-edge feel
     becomes much easier to notice.

## Goals

1. Add a first-class scene primitive for rounded-rect box shadows.
2. Keep `ShadowStyle` as the mechanism/authoring contract for container chrome.
3. Move the default integrated renderer (`fret-render-wgpu`) to a dedicated renderer-owned path for
   `ShadowStyle`-backed box shadows.
4. Keep a deterministic fallback for backends that cannot or do not yet implement the primitive.
5. Delete UI-layer multi-quad expansion as the default path once the primitive is proven.

## Non-goals

1. Replacing `DropShadowV1` or folding box shadow into the effect chain.
2. Arbitrary path shadows or CSS `filter: drop-shadow(...)` parity.
3. Inner shadows or ambient/spot-light systems.
4. Changing shadcn/Tailwind preset ownership in `ecosystem/*`.
5. Adding interaction policy to `crates/fret-ui`.

## Invariants

1. Mechanism vs policy ownership stays intact.
   - `ShadowStyle` remains a mechanism surface.
   - preset mapping and component recipes remain in `ecosystem/*`.

2. `DropShadowV1` stays explicit and separate.
   - It remains the content-derived blur path for effect-owned surfaces that already manage effect
     bounds and accept ADR 0286 degradation rules.

3. `fret-core` stays semantic and backend-agnostic.
   - The scene contract must describe shadow geometry/parameters, not SDF or WGSL details.

4. Fallback remains deterministic.
   - Unsupported backends may degrade to the existing bounded quad approximation, but the fallback
     must no longer be the only representation of box shadow in the scene.

5. This lane is allowed to delete wrong default paths.
   - Once the primitive ships with gates, keeping UI-side multi-quad expansion as the normal path
     would be compatibility theater.

## Options Considered

### Option A: Keep tuning the quad-expansion fallback

Rejected as the main direction.

- It can improve the fallback but cannot fix the structural issue that box shadow is invisible to
  the renderer as a semantic primitive.
- It keeps scene inflation and quality ceilings in the wrong layer.

### Option B: Implicitly map `ShadowStyle` to `DropShadowV1`

Rejected.

- `DropShadowV1` is defined as a content-derived effect step, not a rounded-rect box-shadow
  primitive.
- It requires effect-owned bounds/intermediate semantics that generic container chrome does not
  currently own.
- Silent promotion would violate the ownership split documented by ADR 0060 and ADR 0286.

### Option C: Add a first-class rounded-rect shadow scene primitive

Chosen direction.

- It matches ADR 0030's long-term direction for first-class shadows.
- It preserves `ShadowStyle` as the public mechanism contract while moving implementation quality to
  the renderer.
- It gives the renderer one shadow op per logical layer instead of many quads.

## Proposed v1 Architecture

### 1. New scene primitive

Add a first-class shadow scene operation in `fret-core`.

Provisional shape:

- `SceneOp::ShadowRRect { order, rect, corner_radii, offset, spread, blur_radius, color }`

The exact name is ADR-owned, but the semantic intent is fixed:

- one op per logical shadow layer,
- rounded-rect geometry is explicit,
- blur/spread/offset are explicit,
- shadow order stays independent from quad fill/border order.

### 2. `ShadowStyle` lowers to shadow ops, not quads

`crates/fret-ui/src/paint.rs` should stop expanding each `ShadowLayerStyle` into many `SceneOp::Quad`
entries on the default path.

Instead:

- `ShadowStyle.primary` lowers to one shadow op,
- `ShadowStyle.secondary` lowers to a second shadow op when present.

This keeps theme/preset semantics unchanged while making the renderer aware of shadow intent.

### 3. Renderer-owned quality path

`fret-render-wgpu` should implement the new op through a dedicated shadow pipeline using the same
analytic rounded-rect mindset already established by ADR 0030.

Expected properties:

- shadow softness computed in shader space, not via CPU-expanded quads,
- rounded corners stay coherent with quad/border geometry,
- clipping and transforms stay renderer-owned,
- future quality/perf work can happen without changing `fret-ui` authoring APIs.

### 4. Backend fallback path

Backends that do not yet implement the primitive may degrade deterministically by replaying the
historical portable quad approximation.

Important ownership rule:

- that fallback must remain explicit and non-default,
- not in the container paint path as the primary representation of shadow.

### 5. Delete-ready cleanup target

Once the new primitive is wired and gated:

- the UI-layer "shadow equals many quads" default path becomes delete-ready,
- docs should stop describing that path as the normal implementation,
- the old approximation remains only as an explicit compatibility fallback where still needed.

## ADR Impact

This lane requires a new ADR or an ADR update before landing implementation because it changes the
`fret-core` scene contract.

Expected ADR scope:

- exact scene-op shape and semantics,
- degradation rules for unsupported backends,
- clipping/transform semantics,
- relationship to ADR 0030, ADR 0060, and ADR 0286.

## Gates Required

1. Core contract coverage
   - scene validation, fingerprinting, and replay support for the new shadow op.
2. Renderer conformance
   - GPU readback or deterministic renderer tests for offset, blur footprint, spread, corner radii,
     and clipping.
3. Screenshot evidence
   - representative elevated surfaces such as Card, Calendar, Sonner, and `todo_demo`.
4. Perf evidence
   - prove scene/op count and renderer cost are at least reviewable, and ideally improved relative
     to quad expansion on shadow-heavy surfaces.

## Evidence Anchors

- Current portable lowering: `crates/fret-ui/src/paint.rs`
- Current container paint entrypoint: `crates/fret-ui/src/declarative/host_widget/paint.rs`
- Current scene contract: `crates/fret-core/src/scene/mod.rs`
- ADR 0030 direction: `docs/adr/0030-shape-rendering-and-sdf-semantics.md`
- ADR 0060 coexistence posture: `docs/adr/0060-shadows-and-elevation.md`
- ADR 0286 effect shadow: `docs/adr/0286-drop-shadow-effect-step-v1.md`
- GPUI/Zed shadow lowering: `repo-ref/zed/crates/gpui/src/window.rs`
- GPUI/Zed scene primitive: `repo-ref/zed/crates/gpui/src/scene.rs`
- GPUI/Zed shader path: `repo-ref/zed/crates/gpui_wgpu/src/shaders.wgsl`

## Deliverables

Minimum deliverables for this lane:

- `DESIGN.md`, `TODO.md`, `MILESTONES.md`
- ADR for the new scene primitive
- core scene + renderer implementation
- conformance gate(s) for the primitive
- screenshot evidence for representative elevated surfaces
- cleanup note proving the UI-layer multi-quad path is no longer the default and now survives only
  as an explicit fallback helper
