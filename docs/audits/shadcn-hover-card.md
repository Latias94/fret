# shadcn/ui v4 Audit - Hover Card (new-york)

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `HoverCard` against the upstream shadcn/ui v4 docs
surfaces, the `new-york-v4` registry implementation in `repo-ref/ui`, and the Radix/Base UI
headless references used to validate semantics and mechanism choices.

## Upstream references (source of truth)

- Docs page (base): `repo-ref/ui/apps/v4/content/docs/components/base/hover-card.mdx`
- Docs page (radix): `repo-ref/ui/apps/v4/content/docs/components/radix/hover-card.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/hover-card.tsx`
- Radix primitive: `repo-ref/primitives/packages/react/hover-card/src/hover-card.tsx`
- Base UI preview card root: `repo-ref/base-ui/packages/react/src/preview-card/root/PreviewCardRoot.tsx`
- Base UI preview card trigger: `repo-ref/base-ui/packages/react/src/preview-card/trigger/PreviewCardTrigger.tsx`

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
- Pass: `HoverCard::new(cx, trigger, content)` already acts as the recipe-level composition entry point,
  with `HoverCardTrigger` / `HoverCardContent` preserving shadcn-style part naming at call sites.
- Pass: `HoverCardContent::new([...])` and `HoverCardContent::build(cx, ...)` already cover the
  composable children lane for the content slot.
- Note: Fret intentionally does not add a separate generic `compose()` builder for `HoverCard`
  today. Unlike modal overlays that need explicit portal/overlay/content slot assembly, hover card
  authoring is already expressed by the root's two required slots plus root-owned hover/anchor
  policy (`open_delay`, `close_delay`, `anchor_element`). An extra builder would mostly duplicate
  the current contract without improving semantics.
- Note: For the same reason, a heterogeneous root `children([...])` API is not currently warranted.
  The root only owns trigger/content pairing plus hover/anchor policy, while the content slot
  already has an explicit composable children surface.

### Open/close behavior

- Pass: Hover open/close is implemented via `HoverRegion` + `HoverIntentState`, with a non-zero
  close delay by default to allow moving from trigger to content.
- Pass: Controlled/uncontrolled open state parity is available via
  `HoverCard::new_controllable(cx, open, default_open, trigger, content)`
  (Base UI / Radix `open` + `defaultOpen`).
- Pass: Open lifecycle callbacks are available via `HoverCard::on_open_change` and
  `HoverCard::on_open_change_complete` (Base UI `onOpenChange` + `onOpenChangeComplete`).
- Note: Fret keeps delay ownership on the root (`HoverCard::open_delay(...)` /
  `HoverCard::close_delay(...)`) to match the Radix/shadcn docs surface. Base UI's trigger-owned
  `delay` / `closeDelay` props remain a useful mechanism cross-check, but they do not define the
  Fret public recipe API.

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

### Docs / teaching surface parity

- Pass: The UI Gallery page order now mirrors the upstream docs flow:
  `Demo`, `Usage`, `Trigger Delays`, `Positioning`, `Basic`, `Sides`, `RTL`, `API Reference`.
- Pass: `Basic` is treated as an upstream docs example, not a Fret-only follow-up.
- Pass: `Children (Fret)` now exists as an explicit post-docs follow-up that demonstrates
  `HoverCardContent::new([...])` for caller-owned panel composition without widening the root
  recipe surface.
- Pass: The `Children (Fret)` section now keeps section-scaffold selectors and preview selectors
  disjoint (`ui-gallery-hover-card-children-*` for the section, `ui-gallery-hover-card-children-demo-*`
  for the preview), so diag automation can target the teaching surface without duplicate `test_id`
  collisions.
- Pass: Source attribution now names both shadcn docs surfaces (base + radix), plus Radix/Base UI
  references, so the distinction between public recipe API and mechanism cross-checks is explicit.
- Pass: UI Gallery now also locks the docs surface with a hover-card-specific docs-surface text test
  plus broader docs-smoke selectors for each docs-path section, so docs drift is separated from
  runtime parity.

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn hover_card::tests`
- `cargo test -p fret-ui-gallery --test hover_card_docs_surface`
- Contract test: `hover_card_open_change_events_emit_change_and_complete_after_settle`
- Contract test: `hover_card_open_change_events_complete_without_animation`
- Web placement gate (layout engine v2): `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
- UI Gallery docs smoke gate:
  `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-docs-smoke.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
- UI Gallery interaction gates:
  `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-click-leave-closes.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
  `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/hover-card/ui-gallery-hover-card-trigger-delays.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
- Underlay scroll anchor stability gate: when the trigger lives inside a scrolling underlay, the
  hover card panel tracks the trigger after wheel-driven scroll updates (validated in
  `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` via
  `fret_hover_card_tracks_trigger_when_underlay_scrolls`).
