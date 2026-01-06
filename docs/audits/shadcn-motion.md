# shadcn/ui v4 Audit — Overlay Motion Taxonomy

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

- Presence/timing driver: `ecosystem/fret-ui-kit/src/primitives/presence.rs`
- Transform math helpers: `ecosystem/fret-ui-kit/src/declarative/overlay_motion.rs`
- Per-component wiring in `ecosystem/fret-ui-shadcn/src/*`.

### Motion primitives (math)

- Side-based popper slide-in (open only):
  - `shadcn_enter_slide_transform(side, opacity, opening)`
  - `SHADCN_SLIDE_PX = 8px` (matches `slide-in-from-*-2`)
- Modal sheet-like slide (open + close):
  - `shadcn_modal_slide_transform(side, distance, opacity)`
- Zoom-in/zoom-out around a popper transform origin:
  - `shadcn_zoom_transform(origin, opacity)` (95% -> 100%)

### Timing presets

`overlay_motion.rs` provides frame-count presets assuming ~60fps:

- `SHADCN_MOTION_TICKS_100` (~100ms)
- `SHADCN_MOTION_TICKS_200` (~200ms)
- `SHADCN_MOTION_TICKS_300` (~300ms)
- `SHADCN_MOTION_TICKS_500` (~500ms)

## Current parity notes

- Pass: Popover/Tooltip/HoverCard use fade + zoom + side-based slide-in keyed off popper `Side`.
- Pass: Sheet-like modal panels can slide in/out via `shadcn_modal_slide_transform`.
- Intentional difference: Fret's presence driver is deterministic and tick-based; it does not inspect
  CSS `animation-name` nor listen to animation lifecycle events (Radix DOM behavior).

## Follow-ups (recommended)

- Add a reusable cubic-bezier easing helper so motion can match shadcn's `ease-[cubic-bezier(...)]`
  recipes where used (e.g. NavigationMenu). (`ecosystem/fret-ui-kit/src/headless/easing.rs`)
- Consider a shared "timeline" headless helper (progress + easing + open/close durations) if more
  components need coordinated multi-property transitions beyond opacity.
  (`ecosystem/fret-ui-kit/src/headless/transition.rs`, `ecosystem/fret-ui-kit/src/declarative/transition.rs`)
