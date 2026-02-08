# shadcn/ui v4 Audit - Tooltip

This audit compares Fret's shadcn-aligned `Tooltip` against the upstream shadcn/ui v4 docs and
examples in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/tooltip.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/tooltip.tsx`
- Underlying primitive: Radix `@radix-ui/react-tooltip`

Key upstream notes:

- Tooltip should open on **hover** and on **keyboard focus**.
- v4 updated tooltip colors to `bg-foreground text-background` (2025-09-22 changelog).
- Content includes an arrow and supports rich children (icons, `<Kbd />`, formatted blocks).

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/tooltip.rs`
- Hover intent state machine: `ecosystem/fret-ui-kit/src/headless/hover_intent.rs`
- Overlay placement helpers: `ecosystem/fret-ui-kit/src/overlay.rs`
- Per-window tooltip overlay policy: `ecosystem/fret-ui-kit/src/window_overlays/mod.rs`

## Audit checklist

### Composition surface

- Pass: `Tooltip`, `TooltipTrigger`, `TooltipContent` exist and are declarative-only.
- Pass: `TooltipContent` now supports rich children (`Vec<AnyElement>`), matching upstream examples.
- Pass: `TooltipProvider` exists and provides a shared delay group for consistent open delays.
- Pass: Fret additionally supports an optional custom placement anchor via `TooltipAnchor` +
  `Tooltip::anchor_element(...)` (anchor can be separate from the trigger).
- Pass: Base UI cursor-tracking policy is supported via `Tooltip::track_cursor_axis(...)` with
  `None | X | Y | Both` behavior.

### Open/close behavior

- Pass: Hover open/close is implemented via `HoverRegion` + `HoverIntentState`.
- Pass: Open-on-keyboard-focus is implemented via `ElementContext::is_focused_element(trigger_id)`.
- Pass: Provider-level close delay is supported via `TooltipProvider::close_delay_duration_ms(...)`
  / `TooltipProvider::close_delay_duration_frames(...)` (Base UI `closeDelay` parity).

### Placement & portal behavior

- Pass: Renders into a per-window overlay root (portal-like), not clipped by ancestor overflow.
- Pass: Supports `side` and `align` placement options.
- Pass: Default `side_offset` aligns with upstream's default (`0`) and can be overridden.
- Pass: Placement anchors to **visual bounds** when available (render-transform aware) via
  `fret-ui-kit::overlay::anchor_bounds_for_element`.

### Visual defaults (shadcn parity)

- Pass: Default tooltip background aligns with upstream (`foreground`).
- Pass: Default rounding and padding match the wrapper (`rounded-md`, `px-3`, `py-1.5`).
- Pass: `TooltipContent::text(...)` defaults to `text-xs`-like sizing via `component.tooltip.text_px`
  (fallback `12px`) and a `16px` line height.
- Pass: Tooltip content applies the `text-background` foreground to descendant text/icon primitives
  by default when colors are not specified.
- Pass: Arrow is implemented and enabled by default (can be disabled via `Tooltip::arrow(false)`).
- Pass: Upstream includes zoom/slide animations keyed off `data-state` and `data-side`; Fret matches
  the same motion taxonomy (fade + zoom + side-based slide) on both enter and exit, including a
  geometry-driven transform origin aligned to the anchor/arrow.

## Follow-ups (recommended)

- Consider expanding `TooltipProvider` knobs (e.g. disable-hoverable-content) if parity needs it.

## Validation

- Web placement gate (layout engine v2): `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
- Underlay scroll anchor stability gate: when the trigger lives inside a scrolling underlay, the
  tooltip content tracks the trigger after wheel-driven scroll updates (validated in
  `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` via
  `fret_tooltip_tracks_trigger_when_underlay_scrolls`).
- Contract test: `tooltip_track_cursor_axis_projects_anchor_center_as_expected`.
- Contract test: `tooltip_aliases_map_to_existing_fields`.
