# ADR 0097: Plot3D Rendering Strategy (Correct 3D, Portable UI Contracts)

Status: Proposed

## Context

Fret aims to support both general-purpose apps and editor-grade UX. Plot components are useful in both
domains, and for some products a 3D plot variant (e.g. scatter3d, line3d, surface) is desirable.

For 2D plots, `fret-plot` can emit portable primitives (`SceneOp::{Path,Quad,Text}`) and rely on the
renderer's existing contracts (ADR 0080).

3D changes the constraints:

- Correctness requires depth testing and predictable visibility semantics.
- Performance often requires GPU-friendly representations (instancing, mesh pipelines), especially for
  dense point clouds.
- Fret core contracts must remain wgpu-free (ADR 0004 / ADR 0037 / ADR 0038).

So the key decision is: how do we provide "real 3D" while keeping UI contracts portable?

## Decision

### Preferred (B1): Treat Plot3D as an embedded viewport surface

Plot3D uses the existing engine viewport architecture:

- UI embeds a render target via `SceneOp::ViewportSurface { target: RenderTargetId, ... }` (ADR 0007).
- The host/engine renders the 3D plot into that `RenderTargetId` using the engine render hook pipeline
  (ADR 0038).
- UI forwards input (hover/orbit/pan/zoom) using the viewport input forwarding contract (ADR 0025),
  keeping the interaction data-only and portable across platforms.

This yields:

- **Correctness**: depth test, shading, and visibility are implemented in the engine pipeline.
- **Performance**: the engine can use GPU instancing and specialized render passes.
- **Portability**: `fret-core` remains wgpu-free; UI only handles IDs and events.

#### Implications

- `fret-plot` remains focused on 2D primitives.
- Plot3D becomes either:
  - a separate ecosystem crate (`ecosystem/fret-plot3d`) that is UI-facing and data-only, plus a runner/host
    integration layer that is wgpu-facing, or
  - an app-owned integration using existing viewport APIs (no additional framework code), with optional
    helper glue provided later.

### Alternative (B2): Add first-class mesh/3D draw ops to the scene contract

Define a new portable scene primitive (e.g. `SceneOp::Mesh2D` / `SceneOp::Mesh3D`) and extend the renderer
to support it, including depth and camera conventions.

Pros:

- a fully self-contained Plot3D component can live purely in the UI ecosystem
- unified styling and composition in the scene graph

Cons:

- very high contract cost and long-term compatibility burden (camera, depth, blending, clipping, picking)
- forces early stabilization of high-entropy renderer semantics
- significant implementation complexity and testing requirements

This path should only be taken via a dedicated ADR focused on scene/renderer contracts and conformance tests.

## Consequences

- Plot3D "route B" starts as B1: embedded viewport surfaces, not new scene primitives.
- If/when a self-contained Mesh3D scene primitive is required, it is a separate project milestone with its
  own ADR and conformance suite.

## References

- Viewport surfaces: `docs/adr/0007-viewport-surfaces.md`
- Viewport input forwarding: `docs/adr/0025-viewport-input-forwarding.md`
- Engine render hook / queue ownership: `docs/adr/0038-engine-render-hook-and-submission-coordinator.md`
- Display list ordering: `docs/adr/0002-display-list.md`
- Plot architecture and performance baseline: `docs/adr/0098-plot-architecture-and-performance.md`
