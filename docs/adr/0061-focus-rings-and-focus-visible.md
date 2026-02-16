# ADR 0061: Focus Rings (Outline) and Focus-Visible


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Zed: https://github.com/zed-industries/zed

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
## Context

shadcn-style component libraries rely heavily on a consistent focus treatment:

- `focus-visible:ring-*` for a keyboard-focused outline around controls.
- `ring-offset-*` to separate the ring from the control surface.

In early prototypes, Fret widgets often expressed focus by mutating border color or drawing ad-hoc
extra quads. This makes it hard to keep focus affordances consistent across components, and it
encourages per-widget hacks.

## Decision

Introduce a small, renderer-friendly focus ring primitive:

- `RingStyle` (already in `crates/fret-ui/src/element.rs`) defines the ring geometry and colors.
- `fret_ui::paint::paint_focus_ring(scene, order, bounds, ring)` paints the ring using one or two
  `SceneOp::Quad` operations.

This is a *visual decoration* contract that can be applied by:

- Declarative elements (`PressableProps.focus_ring`).
- Retained widgets (e.g. `TextInputStyle.focus_ring`, `TextAreaStyle.focus_ring`).
- Component-layer recipes (shadcn-style wrappers).

Focus rings are shown when the widget decides to paint them and `cx.focus == Some(cx.node)`.

## Semantics

### Geometry

`RingStyle` supports two placements:

- `Inset`: the ring is drawn inside the element bounds, optionally deflating corner radii.
- `Outset`: the ring is drawn outside the element bounds.
  - If `offset > 0` and `offset_color` is set, a "ring offset" stroke is drawn first.
  - Then the ring stroke is drawn at radius `offset + width`.

Because rings are drawn via quads and border strokes, they may be clipped by parent clip rects.
Components that need un-clipped focus affordances should ensure the ring is painted outside any
content clip.

### Ordering

`SceneOp::Quad.order` is carried for future render ordering semantics, but current renderers follow
the scene op sequence for draw order. Call sites should still use a higher `order` than the
control's background/border to preserve intent.

## Theme Tokens

Baseline keys used by component recipes:

- `component.ring.width` (Px)
- `component.ring.offset` (Px)

Semantic colors (via `Theme::color_by_key` alias bridge):

- `ring`
- `ring-offset-background`

## Focus-Visible Heuristic (v1)

Fret tracks a per-window "focus-visible enabled" flag with a minimal input-modality heuristic:

- `PointerDown` disables focus-visible (mouse/touch interaction).
- `KeyDown` of common navigation keys enables focus-visible (e.g. Tab, arrows, Home/End, PageUp/PageDown).

This is intentionally small and deterministic; it is good enough to match common shadcn-like UX
expectations during MVP development, without requiring a full modality state machine.

## Non-Goals

- True blurred glows and shadows (handled separately; see ADR 0060).
- Perfect parity with CSS `:focus-visible` across all edge cases.

## Consequences

- Components can converge on shadcn-like `focus-visible:ring-*` behavior without duplicating drawing
  code.
- Widgets can transition away from "focused border color" as the primary focus affordance, keeping
  borders stable and applying the ring as a consistent decoration.

## Notes (Zed/GPUI reference, non-normative)

- GPUI tracks an `InputModality` (mouse vs keyboard) on the window and uses it as a focus-visible
  style signal (`Window::last_input_was_keyboard`), updated during event dispatch:
  `repo-ref/zed/crates/gpui/src/window.rs` (`InputModality`, `dispatch_event`,
  `last_input_was_keyboard`).
