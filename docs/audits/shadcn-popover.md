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

### Placement & collision

- Pass: Supports `side`, `align`, and `side_offset` (default 4px) similar to Radix/shadcn.
- Pass: Performs deterministic flip + clamp within window bounds via
  `anchored_panel_bounds_sized(...)` (ADR 0064).
- Pass: Optional arrow is supported via `Popover::arrow(true)` (default is `false`).

### Dismissal behavior

- Pass: Outside-press dismiss is implemented via the click-through observer pass (ADR 0069) and the
  window overlay layer.
- Pass: Escape dismiss is handled by the shared dismissible overlay policy layer.

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
- Partial: Upstream content has open/close + side-based slide/zoom animations; Fret includes
  fade + zoom (best-effort), including a geometry-driven transform origin aligned to the
  anchor/arrow, but does not yet model side-based slide.

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn popover::tests`

## Follow-ups (recommended)

- Add side-based slide/zoom transitions (optional) to better match upstream motion.
