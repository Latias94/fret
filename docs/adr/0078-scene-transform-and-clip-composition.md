# ADR 0078: Scene Transform and Clip Composition (Affine v1)

Status: Accepted

## Context

ADR 0019 introduces a state-stack model for `Scene`:

- `PushTransform / PopTransform`
- `PushOpacity / PopOpacity`
- `PushClipRect / PushClipRRect / PopClip`
- `PushLayer / PopLayer` (reserved)

The repository currently implements the **transform stack plumbing** end-to-end (core contract → UI
paint-cache replay → renderer MVP), but the exact semantics for how transforms compose with clipping
(especially rounded clipping) are not fully locked.

If transform/clip semantics are left implicit, component work will quickly diverge into “works for
scrolling” hacks and renderer-side approximations that later require large rewrites (especially for
viewport overlays, zoomable canvases, and editor tools).

This ADR locks the v1 semantics so:

- UI code can reliably express “clip then scroll/zoom inside” composition.
- Renderer implementations can evolve (scissor → shader clip → mask pass) without changing UI behavior.
- Caching/replay (ADR 0055) has a stable contract for “moving subtrees” via transforms.

## Decision

### 1) Coordinate spaces

- All geometry carried by `SceneOp` (`Rect`, `Point`, `Corners`, `Edges`) is expressed in **logical pixels**
  in the **current local coordinate space**.
- The **cumulative transform stack** maps local coordinates to the scene’s root coordinate space.
- Presentation-scale conversion (`scale_factor`) remains a renderer concern (ADR 0002).

### 2) Transform stack semantics

- `PushTransform { transform }` pushes a new local-to-parent transform.
- Cumulative composition is **left-multiplication**:
  - If `current` is the current cumulative transform and we push `t`,
    the new cumulative transform is `current * t`.
  - This means: apply `t` first, then apply `current`.
- The identity transform is the implicit base state.

### 3) Clip stack semantics are “captured at push time”

`PushClipRect` / `PushClipRRect` push a clip entry that consists of:

- the clip geometry (`rect` (+ `corner_radii`)),
- the **cumulative transform in effect at the time of the push** (“clip-local → world”),
- and the previously active clip stack (nested clips intersect).

Crucially:

- Subsequent `PushTransform` operations do **not** retroactively move/scale existing clip entries.
- If code wants a clip to move with content, it must push the transform first, then push the clip.
- If code wants content to move under a fixed clip (scrolling), it must push the clip first, then
  push the transform, then draw content.

This matches the common editor/UI mental model:

- `clip(viewport)` + `transform(scroll)` + `draw(children)` => children scroll under a fixed viewport.

### 4) Renderer obligations (v1)

Renderers must preserve ordering semantics (ADR 0009) while honoring the transform + clip model above.

For rectangular clip entries, a renderer may use:

- scissor rectangles (fast path), *if* the captured clip transform maps the clip rect to an axis-aligned
  rectangle in world space (i.e. translation and scale only; no rotation or shear).

For rounded clip entries, a renderer must provide soft/AA clipping (ADR 0063). The recommended v1
implementation approach is:

- carry an inverse transform per clip entry (“world → clip-local”), and
- evaluate the rounded-rect SDF (or equivalent) in clip-local space.

If a renderer cannot represent the current clip stack correctly via scissor rectangles, it must fall back
to a shader-based clip evaluation or a mask pass. “Over-clipping” or “under-clipping” is not conformant.

### 5) Opacity semantics (clarification)

`PushOpacity { opacity }` is a **multiplicative opacity multiplier** applied to subsequent draw ops.

- It is *not* an “isolated opacity group” (no offscreen group compositing is implied).
- An isolated group (where overlapping children are composited first, then the group is faded) would
  require a different op (e.g. `PushOpacityGroup`) and is out of scope for this ADR.

## Consequences

- UI code can build correct scroll/zoom/tool overlays with explicit, stable ordering and stack semantics.
- Renderer implementations can start with approximations but must converge on full affine + correct clip
  composition without changing UI behavior.
- Replay caching (ADR 0055) can safely use “wrap in `PushTransform`” as the canonical translation strategy.

## Future Work

- Define a conformance test harness that exercises:
  - clip-before-transform scrolling,
  - transform-before-clip moving masks,
  - rounded clip under rotation,
  - and viewport overlays under zoom.

Current status:

- `crates/fret-render/tests/affine_clip_conformance.rs` provides a GPU-level conformance check for
  affine transform + clip-local evaluation (may skip in environments without a usable adapter).
- `fret-render` encodes the active clip stack into a GPU storage buffer (no fixed `MAX_CLIPS` limit),
  and uses a linked-list node encoding so `PushClip*` can append a single node (no per-op stack
  snapshot copies) while preserving clip-local SDF evaluation for rounded clips.
- `fret-render` uses a scissor-only fast path for axis-aligned rectangular clips; non-axis-aligned or
  rounded clips fall back to shader-based clip evaluation.
- Decide how `PushLayer` should interact with batching and potential offscreen composition.
