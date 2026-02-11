---
title: "ADR 0082: RenderTransform (Paint + Hit-Testing + Event Coordinates)"
---

# ADR 0082: RenderTransform (Paint + Hit-Testing + Event Coordinates)

Status: Accepted

## Context

Fret has a scene-level transform stack (ADR 0019 / ADR 0078) that renderers must honor for correct
clip/transform composition. However, scene transforms alone are not sufficient for UI authoring:

- Components often need to animate/transform subtrees (fade/scale/rotate) while keeping input
  targeting and pointer-local coordinates consistent with the rendered output.
- If transforms are paint-only, hit-testing and pointer-driven widgets (sliders, drags, selection)
  become incorrect (“looks rotated but still clicks at the old location”).

We want a minimal runtime mechanism that:

- stays inside `crates/fret-ui` (mechanism, not component policy; ADR 0066),
- supports full 2D affine transforms (`Transform2D`),
- keeps pointer routing and per-widget pointer coordinates consistent with rendering.

## Decision

### 1) Widget contract

`Widget` may provide an optional render transform:

- `Widget::render_transform(bounds) -> Option<Transform2D>`

Semantics:

- This is a **render transform** (not a layout transform):
  - Layout bounds remain authoritative for measurement and positioning.
  - The transform affects paint, hit-testing, and pointer event coordinates.
- The transform is expressed in **logical pixels** in the same coordinate space as `bounds`
  (window-local).
- Only **invertible** transforms participate. Non-invertible transforms are treated as `None` to
  preserve paint/input consistency.

### 2) Paint semantics

During paint, the runtime wraps the widget subtree in scene transform ops:

- `PushTransform { transform }`
- paint subtree
- `PopTransform`

This lets renderers apply the same affine transform stack semantics as the rest of the scene
pipeline (ADR 0078).

### 3) Hit-testing semantics

Hit-testing traverses the UI tree while mapping the pointer position through inverse transforms:

- Before testing a node (and its children), the runtime applies the inverse of that node’s
  `render_transform` to the pointer position.
- All existing hit-test clipping rules continue to apply in the node’s untransformed layout space
  (rectangular clip and rounded-rect clip).

### 4) Event coordinate semantics

For pointer-position-bearing events (`PointerEvent`, `ExternalDragEvent`, `InternalDragEvent`):

- Each widget receives an event whose `position` is mapped into that widget’s untransformed layout
  space using the same inverse-transform traversal used for hit-testing.
- For `PointerEvent::Wheel`, `delta` is mapped as a vector (translation ignored) through the same
  inverse transform matrix.

### 5) Cross-frame geometry queries (anchored overlays)

Anchored overlays (popover/menu/tooltip) must be able to track the **visual** geometry of an anchor
element across frames, including any ancestor `render_transform` effects, without converting
`render_transform` into a layout concern.

The runtime therefore exposes **two** distinct geometry queries for declarative elements:

- **Layout bounds** (authoritative for layout): `bounds_for_element(...)`
- **Visual bounds** (post-transform AABB, for overlays): `visual_bounds_for_element(...)`

Rules:

- `visual_bounds_for_element` MUST include the **composed** render transforms along the path from
  the element node to the window root (i.e. ancestor transforms apply to descendants).
- Visual bounds MUST be an **axis-aligned bounding box** (AABB) in window-local logical pixels.
- `bounds_for_element` MUST remain layout-only (untransformed) and MUST NOT be repurposed to “track
  transforms”.

This makes the contract explicit:

- Layout stays stable and cacheable.
- Overlays can anchor to what the user actually sees (ADR 0064) without introducing layout-time
  transform semantics.

## Consequences

- Components can safely use transforms for interactive content without breaking hit-testing or
  pointer-local behavior.
- Scene-level transform semantics remain authoritative; renderers do not need component-specific
  special-casing.

### Implementation notes

- Paint caching may be disabled for nodes that return a `render_transform`, because cached scene ops
  are currently replayed via translation (ADR 0055) which can break transforms whose meaning depends
  on position/time.

- If paint caching remains translation-only, it must not replay subtrees in a way that invalidates
  `visual_bounds_for_element` (and thus anchored overlays). If necessary, the runtime may conservatively
  disable caching for subtrees that contain render transforms until a transform-aware replay strategy
  is introduced.

## References

- Scene state stack: `docs/adr/0019-scene-state-stack-and-layers.md`
- Transform + clip semantics: `docs/adr/0078-scene-transform-and-clip-composition.md`
- Runtime contract surface: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Overlay placement contract: `docs/adr/0064-overlay-placement-contract.md`
