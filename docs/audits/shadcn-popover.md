# shadcn/ui v4 Audit - Popover (new-york)


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Popover` surface against the upstream shadcn/ui v4
documentation and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/popover.mdx`
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

- Pass: Fret exposes a typed recipe-level root constructor `Popover::new(cx, trigger, content)`,
  while keeping an explicit advanced seam (`Popover::from_open(...).into_element_with(...)`) for
  callers that already own the open model.
- Pass: Trigger/content composition matches the shadcn mental model: trigger element + portal-like
  content element.
- Pass: The typed compound-parts lane now covers both sides of that composition:
  `PopoverTrigger::build(...)` for the trigger and `PopoverContent::build(cx, ...)` for
  late-landing content children.
- Pass: `PopoverTrigger` now toggles `open` by default (shadcn/Radix trigger-like behavior), and
  supports opt-out via `PopoverTrigger::auto_toggle(false)` for controlled/manual flows.
- Pass: Upstream exports `PopoverAnchor`; Fret provides `PopoverAnchor` and supports custom anchor
  wiring via `Popover::anchor_element(...)`.
  (`Popover::into_element_with_anchor(...)` passes the resolved anchor rect to the content closure,
  which covers common sizing recipes like "content width follows trigger".)
- Pass: Default root authoring no longer requires a separate `.build(...)` step; the common recipe
  path is `Popover::new(cx, trigger, content)`.
- Note: Fret intentionally does not add a separate generic `children([...])` / `compose()` root
  builder for `Popover` today. Unlike modal overlays that need explicit portal/overlay slot
  assembly, popover authoring is already fully expressed by the root's two required slots plus the
  explicit advanced seams for managed-open and anchor-aware flows. Another root builder would
  mostly duplicate the current contract without improving semantics.
- Note: Unlike modal overlays such as `Dialog` / `AlertDialog` / `Sheet` / `Drawer`, Popover still
  keeps anchor-aware content authoring as a first-class advanced path (`from_open(...).into_element_with_anchor(...)`).
  The sibling `from_open(...).into_element_with(...)` path keeps the managed-open but non-anchor-aware
  case explicit. Those seams preserve the anchor-rect contract for sizing recipes such as
  "content width follows trigger".
- Pass: Anchor overrides are treated as dismissable branches, so interacting with the anchor does
  not trigger outside-press dismissal.
- Pass: Detached trigger wiring is available via `Popover::trigger_element(...)` (Base UI-like
  trigger association when the logical trigger is outside the local composition closure).

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

- Pass: Default auto-focus policy now follows trigger contract:
  - `PopoverTrigger` composition defaults to focus-inside-on-open.
  - Manual/custom trigger wiring preserves previous behavior unless overridden.
- Pass: Optional "focus inside on open" is supported via `Popover::auto_focus(true)` and can be
  disabled via `Popover::auto_focus(false)`.
- Pass: Explicit focus target is supported via `Popover::initial_focus(...)`.
- Pass: A trap-focus-only mode is available via `Popover::modal_trap_focus(true)` (focus trap
  without installing a modal barrier; outside pointer interactions remain enabled).
- Pass: Base UI modal enum parity is exposed via `Popover::modal_mode(PopoverModalMode::...)`
  (`NonModal | Modal | TrapFocus`) in addition to boolean/trap convenience methods.

### Base UI parity extensions

- Pass: Hover-open policy (`openOnHover`) is available via `Popover::open_on_hover(true)` with
  delay controls:
  - `Popover::hover_open_delay_frames(...)`
  - `Popover::hover_close_delay_frames(...)`
- Pass: Open lifecycle callbacks are available via `Popover::on_open_change` and
  `Popover::on_open_change_complete` (Base UI `onOpenChange` + `onOpenChangeComplete`).
- Pass: `forceMount` naming parity is exposed via `Popover::force_mount(...)` as an alias of
  `Popover::keep_mounted(...)`.
- Pass: Advanced placement knobs are exposed:
  - `Popover::collision_padding(...)`
  - `Popover::collision_boundary(...)`
  - `Popover::sticky(...)`
  - `Popover::shift_cross_axis(...)` (default aligned to `false`).

### Visual parity (new-york)

- Pass: Default `PopoverContent` sizing matches `w-72` (`288px`) and padding matches `p-4`.
- Pass: `PopoverContent` keeps the panel root fill-width for text wrapping but no longer stretches
  inline-sized children across the cross axis by default, which better matches the upstream content
  `div` contract and avoids accidental full-width button hit boxes.
- Pass: UI Gallery keeps the official docs section order intact and appends a Fret-only regression
  section afterwards so intrinsic-width child behavior can be gated without mutating the upstream
  `Basic` / `Align` / `With Form` examples.
- Pass: Default border/background uses popover tokens (`popover` / `popover.background`, `border`)
  and shadow matches the design-system "md" shadow.
- Pass: Popover title text defaults to `popover.foreground` / `popover-foreground` (best-effort),
  matching `text-popover-foreground` semantics.
- Pass: Upstream content has open/close + side-based slide/zoom animations; Fret matches the same
  motion taxonomy (fade + zoom + side-based slide) on both enter and exit, including a
  geometry-driven transform origin aligned to the anchor/arrow.
- Pass: UI Gallery now also locks the docs/public surface with a popover-specific docs-surface
  text test plus a docs smoke script promoted in the shadcn conformance suite, so docs drift is
  separated from the existing runtime placement, dismissal, and inline-width gates.

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn popover::tests`
- `cargo test -p fret-ui-gallery --test popover_docs_surface`
- Contract test: `popover_content_does_not_stretch_children_by_default`
- UI Gallery diag: `tools/diag-scripts/ui-gallery/popover/ui-gallery-popover-inline-children-button-not-stretched.json`
- Contract test: `popover_modal_mode_alias_sets_expected_mode`
- Contract test: `popover_open_change_events_emit_change_and_complete_after_settle`
- Contract test: `popover_open_change_events_complete_without_animation`
- Underlay scroll anchor stability gate: when the trigger lives inside a scrolling underlay, the
  popover panel tracks the trigger after wheel-driven scroll updates (validated in
  `ecosystem/fret-ui-shadcn/tests/web_vs_fret_overlay_placement.rs` via
  `fret_popover_tracks_trigger_when_underlay_scrolls`).

## Follow-ups (recommended)

- Fine-tune duration/easing values if strict motion parity is required for demos.
