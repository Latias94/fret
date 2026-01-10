# Radix Primitives Audit — Popper

This audit compares Fret's floating placement substrate against the upstream Radix
`@radix-ui/react-popper` implementation pinned in `repo-ref/primitives`.

The goal is **behavioral parity** (placement + collision avoidance + arrow alignment), not React
API parity.

## Upstream references (source of truth)

- Radix implementation: `repo-ref/primitives/packages/react/popper/src/popper.tsx`
- Floating UI middlewares used by Radix:
  - `repo-ref/floating-ui/packages/core/src/middleware/offset.ts`
  - `repo-ref/floating-ui/packages/core/src/middleware/shift.ts`
  - `repo-ref/floating-ui/packages/core/src/middleware/flip.ts`
  - `repo-ref/floating-ui/packages/core/src/middleware/size.ts`
  - `repo-ref/floating-ui/packages/core/src/middleware/arrow.ts`
  - `repo-ref/floating-ui/packages/core/src/middleware/hide.ts`

Key upstream behaviors/surfaces:

- `side`, `align`, `sideOffset`, `alignOffset`.
- `avoidCollisions` + `collisionBoundary` + `collisionPadding`.
- `sticky` (Radix uses `limitShift()` when `sticky="partial"`).
- Arrow positioning with padding; arrow can shift the panel for aligned placements when the anchor
  is too small (`alignmentOffset`), and may expose "cannot center" (`centerOffset`).
- `hideWhenDetached` (Radix uses `hide({ strategy: 'referenceHidden' })`).
- Computed “available” metrics for recipes (`--radix-popper-available-*`, `--radix-popper-anchor-*`).

## Fret mapping

Fret models Radix/Floating outcomes with a deterministic, pure solver and thin wrappers:

### Placement solver (mechanism)

- Deterministic anchored placement + flip + clamp + size clamp + RTL-aware alignment:
  `crates/fret-ui/src/overlay_placement/solver.rs`
- Public API:
  `crates/fret-ui/src/overlay_placement/mod.rs`
- Types:
  `crates/fret-ui/src/overlay_placement/types.rs`
- Utility for collision padding (caller-provided “outer inset”):
  `crates/fret-ui/src/overlay_placement/util.rs` (`inset_rect`)

### Radix-named facade (ecosystem)

- Radix-shaped popper helpers + transform-origin math:
  `ecosystem/fret-ui-kit/src/primitives/popper.rs`
- Popper wrapper/panel skeleton (absolute layout + arrow protrusion hit-test expansion):
  `ecosystem/fret-ui-kit/src/primitives/popper_content.rs`
- Overlay helpers that prefer visual bounds (render-transform aware anchors, ADR 0083):
  `ecosystem/fret-ui-kit/src/overlay.rs` (`anchor_bounds_for_element`, `outer_bounds_with_window_margin`)

### Recipe usage (examples)

shadcn wrappers use the Radix-named facade to compute `placed` rect, wrapper insets, arrow layout,
and transform origin:

- `ecosystem/fret-ui-shadcn/src/popover.rs`
- `ecosystem/fret-ui-shadcn/src/tooltip.rs`
- `ecosystem/fret-ui-shadcn/src/hover_card.rs`
- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- `ecosystem/fret-ui-shadcn/src/context_menu.rs`
- `ecosystem/fret-ui-shadcn/src/menubar.rs`
- `ecosystem/fret-ui-shadcn/src/select.rs`

## Current parity notes

- Pass: `side` + `align` placement and deterministic flip behavior.
  - Fret flips when the preferred side overflows on the **side axis** without clamping, then
    retries on the opposite side; if neither fits, it selects a best-fit side and clamps.
  - Implementation: `anchored_panel_layout_ex(...)` / `anchored_panel_layout_sized_ex(...)` in
    `crates/fret-ui/src/overlay_placement/solver.rs`.
- Pass: `sideOffset` and `alignOffset` are represented as `Offset`:
  - `Offset.main_axis` (added to `side_offset`)
  - `Offset.cross_axis` (skidding)
  - `Offset.alignment_axis` (Radix/Floating `alignmentAxis`; sign flips for `End`, and for vertical
    placements it flips under RTL).
  - Implementation: `Offset` + `apply_cross_axis_offset(...)` in
    `crates/fret-ui/src/overlay_placement/solver.rs`.
- Pass: RTL-aware logical alignment for vertical placements (`Top`/`Bottom`):
  - Implementation: `anchored_origin_ex(...)` in `crates/fret-ui/src/overlay_placement/solver.rs`.
- Pass: Arrow positioning with padding, and aligned-placement panel shift when needed:
  - Fret computes `ArrowLayout { offset, alignment_offset }` and may shift the panel to preserve
    arrow pointing intent.
  - Implementation: `apply_arrow_layout(...)` in `crates/fret-ui/src/overlay_placement/solver.rs`.
- Pass: “Arrow protrusion” is modeled explicitly so recipes can expand wrapper hit-test bounds:
  - `default_arrow_protrusion(...)`, `diamond_arrow_options(...)`,
    `wrapper_insets_for_arrow(...)` in `ecosystem/fret-ui-kit/src/primitives/popper.rs`.
- Pass: Transform origin for shadcn overlay motion is computed from geometry:
  - `popper_content_transform_origin(...)` in `ecosystem/fret-ui-kit/src/primitives/popper.rs`.
- Pass: Collision padding can be expressed by insetting the `outer` rect prior to solving:
  - Fret supports structured collision padding/boundary via `AnchoredPanelOptions.collision`.
  - `crates/fret-ui/src/overlay_placement/types.rs` (`CollisionOptions`)
  - `crates/fret-ui/src/overlay_placement/solver.rs` (applies `CollisionOptions` to `outer`)

## Gaps / intentional differences

- Partial: `collisionBoundary` as a list of DOM elements / alternative boundaries.
  - Fret supports a single additional `boundary: Rect` (intersected with `outer`) via
    `CollisionOptions`, but does not model a full boundary list nor `altBoundary` semantics.
- Pass: Radix `sticky="partial" | "always"` semantics (`limitShift()`).
  - `StickyMode::Partial` emulates Floating `limitShift()` by allowing alignment-axis overflow to
    keep the panel attached to the anchor (prevent detachment).
  - `StickyMode::Always` keeps the current “fully clamp into boundary” behavior.
- Missing: `hideWhenDetached` / “referenceHidden” outcome.
  - Fret clamps/fits instead of hiding the overlay when the anchor is detached from the boundary.
- Missing: Exposed “available width/height” metrics.
  - Radix writes CSS variables for available width/height and anchor width/height; Fret currently
    returns the placed rect (and optional arrow layout), but does not expose those extra numbers as
    a structured output for recipes.
- Partial: Arrow “cannot center” signal.
  - Radix exposes `centerOffset` and uses it to drive `shouldHideArrow`. Fret computes arrow
    clamping and optional `alignment_offset`, but does not explicitly return a `center_offset` /
    `should_hide_arrow` boolean.

## Follow-ups (recommended)

If we want to move closer to 1:1 Radix/Floating outcomes while keeping the solver pure and
deterministic:

1. Add a collision options struct that can be applied to `outer` consistently (padding per side)
   and optionally supports “boundary composition” (e.g. intersect multiple rects).
2. Consider exposing `limitShift({ offset })`-style tuning.
   - Fret currently models the default `limitShift()` behavior; it does not expose the optional
     limiter offset configuration.
3. Add an optional `reference_hidden` / `should_hide` output bit (Radix `hideWhenDetached`).
4. Extend `ArrowLayout` to expose `center_offset` (and derive a `should_hide_arrow` helper).
5. Expose Radix-like “available” metrics as structured outputs for recipes (instead of CSS vars).

## Validation anchors

- Placement solver tests: `crates/fret-ui/src/overlay_placement/tests.rs`
- Popper arrow/wrapper tests: `ecosystem/fret-ui-kit/src/primitives/popper.rs`,
  `ecosystem/fret-ui-kit/src/primitives/popper_content.rs`
