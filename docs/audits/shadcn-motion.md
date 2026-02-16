# shadcn/ui v4 Audit: Overlay Motion Taxonomy


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit documents how Fret models the shadcn/ui v4 motion taxonomy used across overlay-ish
components (Popover/Tooltip/HoverCard/Sheet/etc.) and highlights remaining gaps.

## Upstream references (source of truth)

shadcn/ui v4 uses Tailwind utility recipes (example: `new-york-v4`) that combine:

- `animate-in` / `animate-out`
- `fade-in-0` / `fade-out-0`
- `zoom-in-95` / `zoom-out-95`
- Side-based slide-in utilities keyed off Radix `data-side`
  (`slide-in-from-top-2`, `slide-in-from-right-2`, ...).

Canonical style bundles:

- `repo-ref/ui/apps/v4/registry/styles/style-vega.css`
- `repo-ref/ui/apps/v4/registry/styles/style-nova.css`

Component examples:

- Popover content: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/popover.tsx`
- Tooltip content: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`
- HoverCard content: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/hover-card.tsx`

## Fret implementation

Fret does not have CSS animation events. Motion is composed from:

- Transition/timing driver: `ecosystem/fret-ui-kit/src/headless/transition.rs`,
  `ecosystem/fret-ui-kit/src/declarative/transition.rs`
- Easing presets (including shadcn cubic-bezier): `ecosystem/fret-ui-kit/src/headless/easing.rs`
- Transform math helpers: `ecosystem/fret-ui-kit/src/declarative/overlay_motion.rs`
- Per-component wiring in `ecosystem/fret-ui-shadcn/src/*`.

### Input semantics (hit testing)

In DOM/CSS, animated transforms affect pointer targeting (clicks land where the content appears).
Fret models this explicitly:

- Use `RenderTransform` for interactive overlay motion so hit-testing and pointer event coordinates
  are mapped through the inverse transform.
- Reserve `VisualTransform` for paint-only transforms (decorations, spinners, arrows) that do not
  need to participate in hit testing.

Evidence:

- Render-time transform primitive: `crates/fret-ui/src/element.rs` (`RenderTransformProps`)
- Hit-testing applies inverse render transform: `crates/fret-ui/src/tree/hit_test.rs`
- shadcn overlay wiring uses the shared `wrap_opacity_and_render_transform(...)` helper for motion
  wrappers (to keep hit testing aligned with the visual transform):
  `ecosystem/fret-ui-kit/src/declarative/overlay_motion.rs`,
  `ecosystem/fret-ui-shadcn/src/overlay_motion.rs`,
  `ecosystem/fret-ui-shadcn/src/dialog.rs`,
  `ecosystem/fret-ui-shadcn/src/popover.rs`, `ecosystem/fret-ui-shadcn/src/tooltip.rs`,
  `ecosystem/fret-ui-shadcn/src/hover_card.rs`, `ecosystem/fret-ui-shadcn/src/sheet.rs`,
  `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`, `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`,
  `ecosystem/fret-ui-shadcn/src/context_menu.rs`, `ecosystem/fret-ui-shadcn/src/menubar.rs`,
  `ecosystem/fret-ui-shadcn/src/select.rs`.

### Motion primitives (math)

- Side-based popper slide-in (open only):
  - `shadcn_enter_slide_transform(side, opacity, opening)`
  - `SHADCN_SLIDE_PX = 8px` (matches `slide-in-from-*-2`)
- Modal sheet-like slide (open + close):
  - `shadcn_modal_slide_transform(side, distance, opacity)`
- Zoom-in/zoom-out around a popper transform origin:
  - `shadcn_zoom_transform(origin, opacity)` (95% -> 100%)
- Shared popper overlay transform assembly (enter-only slide + zoom):
  - `shadcn_popper_presence_transform(side, origin, opacity, scale, opening)`

### Timing presets

`overlay_motion.rs` provides wall-clock duration presets (theme overridable):

- `SHADCN_MOTION_DURATION_100` (100ms)
- `SHADCN_MOTION_DURATION_200` (200ms)
- `SHADCN_MOTION_DURATION_300` (300ms)
- `SHADCN_MOTION_DURATION_500` (500ms)

Duration-based drivers map wall-time to the nearest 60Hz tick budget via
`ticks_60hz_for_duration(...)`, then apply refresh-rate scaling in the runtime driver so perceived
timings remain stable on 60/120/144Hz displays.

Theme override keys:

- `duration.shadcn.motion.{100,200,300,500}`
- `easing.shadcn.motion`

## Current parity notes

- Pass: Popover/Tooltip/HoverCard use fade + zoom + side-based slide-in keyed off popper `Side`.
- Pass: Sheet-like modal panels can slide in/out via `shadcn_modal_slide_transform`.
- Pass: Motion progress is driven by a shared transition timeline with shadcn-aligned easing.
- Pass: NavigationMenu directional content switching matches shadcn's `data-motion` semantics via
  `navigation_menu_content_transition(...)` + `navigation_menu_content_switch(...)`.
- Intentional difference: Fret's presence driver is deterministic and tick-based; it does not inspect
  CSS `animation-name` nor listen to animation lifecycle events (Radix DOM behavior).

## Follow-ups (future)

- Extend motion taxonomy coverage to remaining overlay-ish components as they are added.
