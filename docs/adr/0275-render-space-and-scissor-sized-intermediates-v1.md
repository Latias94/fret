---
title: RenderSpace and Scissor-Sized Intermediates v1
status: Draft
date: 2026-02-14
---

# ADR 0275: RenderSpace and Scissor-Sized Intermediates v1

## Context

Many renderer features require offscreen intermediates:

- isolated opacity groups (`saveLayer(alpha)`, ADR 0272),
- compositing groups (blend modes),
- filter chains (e.g. blur/color adjust),
- and future clip-path / mask-image implementations.

Allocating intermediates at full viewport size is often the dominant cost:

- extra passes + bandwidth,
- larger transient textures (VRAM pressure),
- and avoidable batching flushes around group boundaries.

To make these features viable on wasm/mobile (and on downlevel GPUs), intermediates must be:

- bounded (by explicit computation bounds / scissor),
- budgeted (peak bytes tracked),
- and deterministically degradable (no silent divergence).

However, moving to scissor-sized render targets introduces a correctness hazard:

- scene geometry and scissor rectangles are expressed in **absolute viewport pixel space**,
- but a scissor-sized intermediate has a **local framebuffer origin**.

If shaders and scissor configuration continue to assume “viewport origin is (0, 0)”, then
rendering into a smaller intermediate produces incorrect clip-space projection and incorrect
scissoring.

## Decision

### D1 — Introduce `RenderSpace` as a renderer-internal coordinate contract

For any pass that renders scene geometry in absolute pixel coordinates, the renderer provides a
uniform describing the current render target’s pixel-space mapping:

- `RenderSpace { origin_px, size_px }`

Semantics:

- `origin_px` is the absolute pixel origin of the pass’s framebuffer within the logical viewport.
- `size_px` is the pixel size of the pass’s framebuffer.

Vertex shaders that convert pixel positions to clip space must do so relative to `RenderSpace`,
not relative to the global viewport:

- `local_px = pixel_pos_px - origin_px`
- NDC is derived from `local_px / size_px`.

This allows the scene contract to remain stable (absolute coordinates), while enabling scissor-
sized render targets.

### D2 — Allocate scissor-sized intermediates for bounded groups/effects when possible

For bounded group/effect modes that render into an intermediate and then composite back:

- prefer allocating an intermediate sized to the group/effect scissor bounds,
- apply renderer intermediate budgets during plan compilation,
- deterministically degrade when budgets are insufficient (as defined by existing group/effect
  degradation rules).

### D3 — Guardrail: disable scissor-sized intermediates when Backdrop effects are present

Backdrop-style effects often sample existing scene content and may rely on absolute coordinate
assumptions across multiple passes (including masked variants).

Until Backdrop passes are explicitly made `RenderSpace`-aware end-to-end, the renderer must:

- fall back to full-viewport intermediates for any scene that contains `EffectMode::Backdrop`.

This is a correctness-first constraint and is expected to be lifted in a follow-up ADR once the
required coordinate invariants are fully specified and tested.

## Semantics (normative)

### Coordinate spaces

- Scene geometry positions, clip/mask evaluation positions, and scissor rectangles are expressed
  in absolute viewport pixel space.
- A pass render target defines a local framebuffer space:
  - `local_px = absolute_px - render_space.origin_px`.

### Scissor mapping

When a pass uses a scissor rectangle that was computed in absolute pixel space, the renderer must:

- translate the scissor by `-render_space.origin_px`,
- clamp it to `[0, 0]..[render_space.size_px]`,
- and treat the result as the GPU scissor for that render pass.

### RenderPlan scissor representation

To avoid accidental mixing of coordinate spaces during refactors, any scissor stored in the internal
render plan MUST be explicitly tagged as either:

- **absolute (render-space)**: expressed in viewport pixel space, or
- **local (dst-target space)**: expressed relative to the destination framebuffer origin (`0..dst_size`).

If a pass stores an absolute scissor, it must be mapped through the pass's `RenderSpace` before being
used as a GPU scissor.

### Budgeting and deterministic degradation

- Scissor-sized intermediates participate in the same budgeting surface as full-sized
  intermediates.
- When the renderer cannot allocate the required intermediate size under budget, it must degrade
  deterministically (no best-effort partial allocation).

## Consequences

### Benefits

- Dramatically reduces intermediate fill cost for bounded groups/effects.
- Establishes a portable coordinate contract that supports wasm/mobile constraints.
- Keeps the public scene contract stable (no new external coordinate types).

### Costs

- One additional uniform binding in the renderer’s primary uniform bind group.
- Vertex shaders that use pixel positions must be updated to use `RenderSpace`.
- Some effect families (notably Backdrop) require explicit follow-up work to be safely compatible
  with scissor-sized intermediates.

## Alternatives considered

1) Keep full-viewport intermediates and rely on scissor only.
   - Simple, but too expensive on many targets and blocks predictable budget scaling.

2) Express scene geometry in local intermediate coordinates.
   - Would leak intermediate planning into the scene contract and complicate retained → declarative
     evolution (as well as tooling and conformance).

## Open questions / follow-ups

- How should `EffectQuality` drive downsample tiers for scissor-sized intermediates (not just blur)?
- What additional conformance scenes are required to safely enable scissor-sized Backdrop effects?
- How should clip-path / image-mask generation caches key their render-space / transform capture?

## Evidence anchors

- `crates/fret-render-wgpu/src/renderer/render_plan.rs` (scissor-sized intermediate planning + budget gating; `AbsoluteScissorRect` vs `LocalScissorRect`; debug-only plan validation)
- `crates/fret-render-wgpu/src/renderer/render_scene/render.rs` (RenderSpace uniform updates; absolute→local scissor mapping)
- `crates/fret-render-wgpu/src/renderer/resources.rs` + `crates/fret-render-wgpu/src/renderer/shaders.rs` (binding `@group(0) @binding(5)`)
