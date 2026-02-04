# shadcn/ui v4 Audit - Popover (new-york)

This audit compares Fret's shadcn-aligned `Popover` surface against the upstream shadcn/ui v4
documentation and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/popover.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/popover.tsx`
- Registry demo: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/popover-demo.tsx`
- Underlying primitive concept: Radix `@radix-ui/react-popover` (portal + anchored content + dismiss)

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/popover.rs`
- Depends on overlay policy/infra:
  - `ecosystem/fret-ui-kit/src/window_overlays/*` (dismissible overlays, focus rules)
  - `ecosystem/fret-ui-kit/src/overlay_controller.rs` (overlay requests + presence)
  - `ecosystem/fret-ui-kit/src/primitives/popover.rs` (Radix-aligned a11y + request facade)
  - `crates/fret-ui/src/overlay_placement/solver.rs` (anchored placement + flip/clamp)

## What upstream exports (new-york)

Upstream shadcn/ui exports a thin wrapper around Radix:

- `Popover` (root)
- `PopoverTrigger`
- `PopoverContent` (defaults: `align="center"`, `sideOffset={4}`, `w-72`, `p-4`, `rounded-md`,
  `border`, `shadow-md`, `z-50`, open/close animations keyed off `data-state` and `data-side`)
- `PopoverAnchor` (optional custom anchor)

## Audit checklist

### API & composition

- Pass: Fret exposes a `Popover` recipe with a `Model<bool>` open state.
- Pass: Trigger/content composition matches the shadcn mental model: trigger element + portal-like
  content element.
- Pass: Upstream exports `PopoverAnchor`; Fret provides `PopoverAnchor` and supports custom anchor
  wiring via `Popover::anchor_element(...)`.
  (`Popover::into_element_with_anchor(...)` passes the resolved anchor rect to the content closure,
  which covers common sizing recipes like "content width follows trigger".)
- Pass: Anchor overrides are treated as dismissable branches, so interacting with the anchor does
  not trigger outside-press dismissal.

### Placement & collision

- Pass: Supports `side`, `align`, and `side_offset` (default 4px) similar to Radix/shadcn.
- Pass: Uses deterministic flip/clamp placement via the Radix-shaped popper facade.
  - Placement policy: `fret_ui_kit::primitives::popper::PopperContentPlacement`
  - Solver: `crates/fret-ui/src/overlay_placement/solver.rs`
  - Layout-driven placement: `AnchoredProps` (no re-entrant “measure triggers layout” loops).
  - Default shift axis matches Radix (`shift({ crossAxis: false })`) via `ShiftOptions` in the
    popper facade.
- Pass: Optional arrow is supported via `Popover::arrow(true)` (default is `false`).

### Dismissal behavior

- Pass: Outside-press dismiss is implemented via the click-through observer pass (ADR 0069) and the
  window overlay layer.
- Pass: Escape dismiss is handled by the shared dismissible overlay policy layer.
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) via
  `Popover::on_dismiss_request(...)`. When set, Escape/outside-press dismissal route through the
  handler and do not automatically close `open`. For modal popovers, the barrier press also routes
  through the same handler.

### Focus behavior

- Pass: Default behavior preserves trigger focus (close to Radix default focus restore behavior).
- Pass: Optional "focus inside on open" is supported via `Popover::auto_focus(true)`.
- Pass: Explicit focus target is supported via `Popover::initial_focus(...)`.

### Visual parity (new-york)

- Pass: Default `PopoverContent` sizing matches `w-72` (`288px`) and padding matches `p-4`.
- Pass: Default border/background uses popover tokens (`popover` / `popover.background`, `border`)
  and shadow matches the design-system "md" shadow.
- Pass: Popover title text defaults to `popover.foreground` / `popover-foreground` (best-effort),
  matching `text-popover-foreground` semantics.
- Pass: Upstream content has open/close + side-based slide/zoom animations; Fret matches the same
  motion taxonomy (fade + zoom + side-based slide) on both enter and exit, including a
  geometry-driven transform origin aligned to the anchor/arrow.

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn popover::tests`
- Underlay scroll anchor stability gate: when the trigger lives inside a scrolling underlay, the
  popover panel tracks the trigger after wheel-driven scroll updates (validated in
  `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` via
  `fret_popover_tracks_trigger_when_underlay_scrolls`).

## Follow-ups (recommended)

- Fine-tune duration/easing values if strict motion parity is required for demos.
