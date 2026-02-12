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
  - Includes `ShiftOptions` (Floating UI `shift()` axis configuration).
- Utility for collision padding (caller-provided “outer inset”):
  `crates/fret-ui/src/overlay_placement/util.rs` (`inset_rect`)

### Radix-named facade (ecosystem)

- Radix-shaped popper helpers + transform-origin math:
  `ecosystem/fret-ui-kit/src/primitives/popper.rs`
- Popper wrapper/panel skeleton (absolute layout + arrow protrusion hit-test expansion):
  `ecosystem/fret-ui-kit/src/primitives/popper_content.rs`
- Overlay helpers that prefer visual bounds (render-transform aware anchors, ADR 0082):
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
- Pass: `hideWhenDetached` / “referenceHidden” outcome.
  - Fret exposes `PopperContentPlacement::with_hide_when_detached(true)` and
    `PopperContentPlacement::reference_hidden(...)` for Radix-like `hide({ strategy:
    'referenceHidden' })`.
  - Recipes typically keep the content mounted, but gate opacity + interactivity when
    `reference_hidden` is true (matching Radix `hideWhenDetached`).
  - Implementation: `ecosystem/fret-ui-kit/src/primitives/popper.rs`
  - Usage examples: `ecosystem/fret-ui-shadcn/src/popover.rs`,
    `ecosystem/fret-ui-shadcn/src/tooltip.rs`
- Pass: Wiring “available height” into Select popper sizing.
  - Radix Select recipes use `--radix-select-content-available-height`; shadcn v4 relies on it for
    default max-height.
  - Fret computes the same concept from `popper_available_metrics(...)` via
    `select_popper_available_height(...)` and uses it to size `SelectPosition::Popper`.
  - The “available height/width” metrics are computed against the *effective collision boundary*
    (after collision padding + boundary intersection), matching Radix/Floating `size()` behavior.
  - Implementation: `ecosystem/fret-ui-kit/src/primitives/select.rs`
  - Usage example: `ecosystem/fret-ui-shadcn/src/select.rs`

## Gaps / intentional differences

- Partial: `collisionBoundary` as a list of DOM elements / alternative boundaries.
  - Fret supports a single additional `boundary: Rect` (intersected with `outer`) via
    `CollisionOptions`, but does not model a full boundary list nor `altBoundary` semantics.
- Pass: Radix `sticky="partial" | "always"` semantics (`limitShift()`).
  - `StickyMode::Partial` emulates Floating `limitShift()` by allowing alignment-axis overflow to
    keep the panel attached to the anchor (prevent detachment), when alignment-axis shifting is
    enabled.
  - `StickyMode::Always` keeps the “clamp into boundary” behavior as configured by `ShiftOptions`.
- Pass: Floating `shift({ crossAxis: false })` is modeled explicitly.
  - Fret exposes `ShiftOptions { main_axis, cross_axis }` on `AnchoredPanelOptions`.
  - The Radix-shaped popper facade defaults to `cross_axis=false` to match Radix’s typical
    `shift({ crossAxis: false })` usage.
  - Implementation: `ecosystem/fret-ui-kit/src/primitives/popper.rs`
- Pass: Exposed “available width/height” metrics.
  - `popper_available_metrics(...)` returns structured `available_width/available_height` and
    `anchor_width/anchor_height` for recipes (Radix uses CSS vars).
  - Wired into Radix-shaped recipe sizing helpers:
    - Select: `ecosystem/fret-ui-kit/src/primitives/select.rs`,
      `ecosystem/fret-ui-shadcn/src/select.rs`
    - DropdownMenu: `ecosystem/fret-ui-kit/src/primitives/dropdown_menu.rs`,
      `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
    - ContextMenu: `ecosystem/fret-ui-kit/src/primitives/context_menu.rs`,
      `ecosystem/fret-ui-shadcn/src/context_menu.rs`
    - Menubar: `ecosystem/fret-ui-kit/src/primitives/menubar.rs`,
      `ecosystem/fret-ui-shadcn/src/menubar.rs`
    - HoverCard: `ecosystem/fret-ui-kit/src/primitives/hover_card.rs`
    - Popover: `ecosystem/fret-ui-kit/src/primitives/popover.rs`
    - Tooltip: `ecosystem/fret-ui-kit/src/primitives/tooltip.rs`
- Pass: Arrow “cannot center” signal.
  - Fret exposes `ArrowLayout.center_offset` (Floating `centerOffset`), which callers can use to
    implement Radix `shouldHideArrow` (`center_offset != 0`).

## Follow-ups (recommended)

If we want to move closer to 1:1 Radix/Floating outcomes while keeping the solver pure and
deterministic:

1. Extend `collisionBoundary` modeling beyond a single rect (boundary lists + `altBoundary`
   semantics), if we need full Radix parity for complex scrollers/portals.
2. Consider exposing `limitShift({ offset })`-style tuning.
   - Fret currently models the default `limitShift()` behavior; it does not expose the optional
     limiter offset configuration.
3. Consider wiring `popper_available_metrics(...)` into more size-limited recipes (menus, combobox,
   etc.) instead of relying on fixed theme caps.

## Validation anchors

- Placement solver tests: `crates/fret-ui/src/overlay_placement/tests.rs`
- Popper arrow/wrapper tests: `ecosystem/fret-ui-kit/src/primitives/popper.rs`,
  `ecosystem/fret-ui-kit/src/primitives/popper_content.rs`
- Select sizing (available height): `ecosystem/fret-ui-kit/src/primitives/select.rs`,
  `ecosystem/fret-ui-shadcn/src/select.rs`
