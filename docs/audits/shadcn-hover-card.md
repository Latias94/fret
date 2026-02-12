# shadcn/ui v4 Audit - Hover Card (new-york)

This audit compares Fret's shadcn-aligned `HoverCard` against the upstream shadcn/ui v4 docs and
the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/hover-card.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/hover-card.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/hover_card.rs`
- Radix facade: `ecosystem/fret-ui-kit/src/primitives/hover_card.rs`
- Hover intent state machine: `ecosystem/fret-ui-kit/src/headless/hover_intent.rs`
- Placement helpers: `ecosystem/fret-ui-kit/src/overlay.rs`
- Per-window hover overlay policy: `ecosystem/fret-ui-kit/src/window_overlays/mod.rs`

## Audit checklist

### Composition surface

- Pass: `HoverCard`, `HoverCardTrigger`, `HoverCardContent` exist and are declarative-only.
- Pass: Content is rendered via a per-window overlay root (portal-like), so it is not clipped by
  ancestor `overflow: Clip`.
- Pass: Supports an optional custom placement anchor via `HoverCardAnchor` +
  `HoverCard::anchor_element(...)` (anchor can be separate from the trigger).

### Open/close behavior

- Pass: Hover open/close is implemented via `HoverRegion` + `HoverIntentState`, with a non-zero
  close delay by default to allow moving from trigger to content.
- Pass: Controlled/uncontrolled open state parity is available via
  `HoverCard::new_controllable(cx, open, default_open, trigger, content)`
  (Base UI / Radix `open` + `defaultOpen`).
- Pass: Open lifecycle callbacks are available via `HoverCard::on_open_change` and
  `HoverCard::on_open_change_complete` (Base UI `onOpenChange` + `onOpenChangeComplete`).

### Placement & sizing

- Pass: Anchored placement uses deterministic flip/clamp behavior via the Radix-shaped popper facade.
  - Placement policy: `fret_ui_kit::primitives::popper::PopperContentPlacement`
  - Solver: `crates/fret-ui/src/overlay_placement/solver.rs`
  - Layout-driven placement: `AnchoredProps`.
- Pass: Optional arrow is supported via `HoverCard::arrow(true)` (default is `false`).

### Visual parity (new-york)

- Pass: Default content sizing matches `w-64` (`256px`) and padding matches `p-4`.
- Pass: Default background/border follow popover tokens (`popover` / `popover.background`, `border`).
- Pass: Upstream includes open/close animations (fade + zoom + side-based slide) keyed off
  `data-state` and `data-side`. Fret matches the same motion taxonomy on both enter and exit, using a
  geometry-driven transform origin aligned to the anchor/arrow.

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn hover_card::tests`
- Contract test: `hover_card_open_change_events_emit_change_and_complete_after_settle`
- Contract test: `hover_card_open_change_events_complete_without_animation`
- Web placement gate (layout engine v2): `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
- Underlay scroll anchor stability gate: when the trigger lives inside a scrolling underlay, the
  hover card panel tracks the trigger after wheel-driven scroll updates (validated in
  `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` via
  `fret_hover_card_tracks_trigger_when_underlay_scrolls`).
