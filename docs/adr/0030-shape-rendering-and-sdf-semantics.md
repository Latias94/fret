# ADR 0030: Shape Rendering and Analytic SDF Semantics (Rounded Rects, Borders, Shadows)

Status: Proposed

## Context

Fret uses GPU rendering (`wgpu`) and already implements rounded rectangles via an analytic SDF quad shader.

- Current Fret quad shader (analytic rounded-rect SDF):
  - `crates/fret-render/src/renderer.rs`
- GPUI’s reference implementation (quad SDF + borders + shadows + clip/transforms):
  - `repo-ref/zed/crates/gpui/src/platform/blade/shaders.wgsl`

For an editor UI, we need scalable, consistent primitives:

- rounded rectangles, borders, separators,
- overlays, popups, drag previews (layering correctness),
- eventual drop shadows/blur and soft/rounded clipping.

We must keep `fret-core` backend-agnostic: the scene describes *what* to draw; SDF is only *how* `fret-render` draws it.

References:

- Display list ordering and batching constraints:
  - `docs/adr/0002-display-list.md`
  - `docs/adr/0009-renderer-ordering-and-batching.md`
- Scene state stack evolution (transforms/opacity/layers):
  - `docs/adr/0019-scene-state-stack-and-layers.md`

## Decision

### 1) SDF remains a renderer implementation detail

`fret-core` continues to expose semantic primitives (`SceneOp::Quad`, later `Shadow`, etc.) without mentioning SDF.

`fret-render` may implement these primitives using:

- analytic SDF in a fragment shader,
- MSAA geometry,
- mask textures,
- or other approaches,

as long as semantics remain stable.

### 2) Anti-aliasing rule is defined (do not hard-code thresholds)

To avoid later rewrites due to DPI scaling and animation artifacts, we define:

- AA must be derived from the SDF gradient (e.g. `fwidth`) rather than a fixed threshold.

This is necessary for consistent edges across different scale factors and transforms.

### 3) Border semantics are standardized

`SceneOp::Quad` borders must have stable semantics:

- alignment policy (inside/center/outside) is explicitly defined (default: inside or center, TBD),
- per-edge widths are supported,
- corner joins are consistent with rounded corners.

If we do not define these semantics early, higher-level UI components (docking chrome, inspectors) will “bake in”
assumptions that later become incompatible.

### 4) Shadows become a first-class primitive (not a Quad flag)

Drop shadows and blur are common in editor UIs and are difficult to do “as a quad parameter” without coupling:

- shadow geometry,
- blur radius,
- sampling strategy,
- batching boundaries.

Therefore shadows should be modeled as separate scene primitives (exact shape TBD) so they can evolve independently.

### 5) Clip evolution path is explicit

Short term:

- map `PushClipRect/PopClip` to scissor rectangles (fast, portable).

Medium term:

- add shader-based soft/rounded clip as a renderer feature, without changing `fret-core` semantics.

## Consequences

- We can grow from “simple MVP quads” to a full editor visual language without reworking `SceneOp` contracts.
- Rendering correctness (layering and order) remains guaranteed by ADR 0009, independent of batching optimizations.
- UI components can rely on stable border/shadow semantics early, avoiding visual and behavioral churn.

## Open Questions (To Decide Before Implementation)

1) **Border alignment default**:
   - inside vs center vs outside (and interaction with pixel snapping).
2) **Shadow model**:
   - do we support multiple shadows per quad, and how do we batch them?
3) **Transform interaction**:
   - how do transforms affect SDF AA and clip/shadow sampling?
4) **Performance targets**:
   - expected primitive counts for a full editor UI and required batching strategy.

