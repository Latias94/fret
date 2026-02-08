# ADR 0030: Shape Rendering and Analytic SDF Semantics (Rounded Rects, Borders, Shadows)

Status: Accepted

## Context

Fret uses GPU rendering (`wgpu`) and already implements rounded rectangles via an analytic SDF quad shader.

- Current Fret quad shader (analytic rounded-rect SDF):
  - `crates/fret-render-wgpu/src/renderer/mod.rs`
- GPUI’s reference implementation (quad SDF + borders + shadows + clip/transforms):
  - `repo-ref/zed/crates/gpui/src/platform/blade/shaders.wgsl`

For an editor UI, we need scalable, consistent primitives:

- rounded rectangles, borders, separators,
- overlays, popups, drag previews (layering correctness),
- eventual drop shadows/blur and soft/rounded clipping.

We must keep `fret-core` backend-agnostic: the scene describes *what* to draw; SDF is only *how* `fret-render` draws it.

Important: “SDF” in this ADR refers to **analytic SDF for shape primitives** (rounded rects, borders, shadows).
Text rendering follows a separate glyph atlas pipeline and should not be conflated with shape SDF (see ADR 0029).

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

Implementation note:

- The quad shader should use an `fwidth`-derived width (e.g. `aa = fwidth(sdf)`) and a smooth transition
  (e.g. `1 - smoothstep(-aa, aa, sdf)`) instead of a constant like `0.5`.

Reference note:

- GPUI’s quad shader uses a constant edge threshold (`antialias_threshold = 0.5`) in its implementation.
  That can be acceptable when the SDF is defined in device-pixel units and the pipeline avoids transforms
  that distort derivatives. Fret’s contract assumes we will need transforms/animation/variable DPI and thus
  requires derivative-based AA to keep semantics stable across backends.

### 3) Border semantics are standardized

`SceneOp::Quad` borders must have stable semantics:

- alignment policy (inside/center/outside) is explicitly defined (**default: inside**),
- per-edge widths are supported,
- corner joins are consistent with rounded corners.

Locked P0 border rules:

- **Default alignment: inside**. Borders never extend outside the quad’s bounds.
- Corner radii are treated as outer radii. The inner radii are derived by subtracting the adjacent border widths
  and clamping at 0 (implementation detail, but the visual expectation is stable).

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

### 6) Dashed borders are deferred (P0) and treated as an interaction primitive

Dashed borders are common in editor UX (selection rectangles, docking drop-zone highlights, drag previews),
but they are also a high-entropy feature: once “dashed border” becomes a general `Quad` capability, we must lock
edge cases early (rounded corners, per-edge widths, dash phase, pixel snapping, transforms, clip interaction).

Therefore, **P0 does not standardize dashed borders as part of `SceneOp::Quad` semantics**.

Recommended P0 approach for editor interactions:

- implement dashed rectangles as a **component-level overlay primitive** that expands into multiple `SceneOp::Quad`
  segments (short solid rectangles) along each edge.
- semantics are explicitly interaction-oriented and stable:
  - dash pattern is defined in device-pixel terms (or equivalently scale-aware logical pixels),
  - the pattern **restarts per edge** (not perimeter-continuous),
  - no “evenly divide the perimeter” adjustment (to avoid resizing-induced pattern jumps).

Future (P1+) options (not locked by this ADR):

- add a dedicated stroke/path primitive (preferred) instead of bloating `Quad`, e.g. `StrokeRect`/`StrokePath`,
  so dashed semantics live with stroke semantics rather than fill semantics.
- implement dashed borders in the renderer (shader or geometry), inspired by GPUI’s `fs_quad` perimeter parameterization.

## Consequences

- We can grow from “simple MVP quads” to a full editor visual language without reworking `SceneOp` contracts.
- Rendering correctness (layering and order) remains guaranteed by ADR 0009, independent of batching optimizations.
- UI components can rely on stable border/shadow semantics early, avoiding visual and behavioral churn.

## Open Questions (To Decide Before Implementation)

1) **Additional border alignments (beyond inside)**:
   - do we support center/outside alignment modes in `SceneOp::Quad`, and if so, how are they expressed without
     coupling to a specific shader technique?
2) **Shadow model**:
   - do we support multiple shadows per quad, and how do we batch them?
3) **Transform interaction**:
   - how do transforms affect SDF AA and clip/shadow sampling?
4) **Performance targets**:
   - expected primitive counts for a full editor UI and required batching strategy.
5) **Dashed border semantics**:
   - if we later support dashed borders as a renderer feature, do we choose:
     - per-edge restart vs perimeter-continuous mode,
     - phase anchoring rules (stable under resize vs evenly distributed),
     - constraints for rounded corners and per-edge widths.
