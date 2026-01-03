# shadcn/ui v4 Audit - Hover Card (new-york)

This audit compares Fret's shadcn-aligned `HoverCard` against the upstream shadcn/ui v4 docs and
the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/hover-card.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/hover-card.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/hover_card.rs`
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

### Placement & sizing

- Pass: Anchored placement uses deterministic flip/clamp behavior via
  `overlay_placement::anchored_panel_bounds_sized(...)`.
- Pass: Optional arrow is supported via `HoverCard::arrow(true)` (default is `false`).

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn hover_card::tests`
