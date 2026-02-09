# shadcn/ui v4 Audit - Sheet (new-york)

This audit compares Fret's shadcn-aligned `Sheet` surface against the upstream shadcn/ui v4 docs
and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/sheet.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/sheet.tsx`
- Underlying primitive concept: Radix `@radix-ui/react-dialog` (Sheet is a styled modal dialog)

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/sheet.rs`
- Depends on overlay policy/infra:
  - `ecosystem/fret-ui-kit/src/window_overlays/*` (modal overlays, focus restore/initial focus)
  - `ecosystem/fret-ui-kit/src/overlay_controller.rs` (overlay requests + presence)
  - `ecosystem/fret-ui-kit/src/primitives/dialog.rs` (Radix-aligned modal request + barrier helpers)

## What upstream exports (new-york)

Upstream shadcn/ui exports a thin wrapper around Radix Dialog:

- `Sheet`
- `SheetTrigger`
- `SheetContent`
- `SheetHeader`
- `SheetFooter`
- `SheetTitle`
- `SheetDescription`

## Audit checklist

### Composition surface

- Pass: Fret exposes `Sheet` as a recipe driven by a `Model<bool>` open state.
- Pass: Trigger/content composition matches the shadcn mental model.
- Pass: Content is rendered via a per-window overlay root (portal-like), so it is not clipped by
  underlay layout/overflow.

### Placement & sizing

- Pass: Supports `side` (`Left`/`Right`/`Top`/`Bottom`) and a per-side `size` override (width or
  height), matching the upstream intent.
- Pass: `Left`/`Right` default width is token-driven (e.g. `component.sheet.size` /
  `component.sheet.width`).
- Pass: `Top`/`Bottom` default to auto height (upstream uses `h-auto`), unless an explicit
  `Sheet::size(...)` override is provided.
- Pass: Default overlay color matches the upstream `bg-black/50` intent.

### Dismissal behavior

- Pass: Escape dismiss is handled by the shared dismissible root (Radix-aligned outcome).
- Pass: Overlay click-to-dismiss is implemented by rendering a full-window barrier behind the
  content (default on, configurable via `Sheet::overlay_closable(...)`).
- Pass: Base UI-compatible convenience alias `disable_pointer_dismissal(bool)` is provided and
  maps directly to `overlay_closable(!disable)`.
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) via
  `Sheet::on_dismiss_request(...)`. When set, Escape and overlay-click dismissal route through the
  handler and do not automatically close `open`.
- Pass: Open lifecycle callbacks are available via `Sheet::on_open_change` and
  `Sheet::on_open_change_complete` (Base UI `onOpenChange` + `onOpenChangeComplete`).

### Focus behavior

- Pass: Modal barrier scoping prevents underlay focus traversal (ADR 0068).
- Pass: On open, initial focus falls back to the first focusable descendant within the modal root
  (via `window_overlays` focus helpers).
- Pass: On close, focus restoration is deterministic to the trigger (modal close unmount path).

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn sheet::tests`
- Contract test: `sheet_disable_pointer_dismissal_alias_maps_overlay_closable`
- Contract test: `sheet_open_change_events_emit_change_and_complete_after_settle`
- Contract test: `sheet_open_change_events_complete_without_animation`
- Shadcn Web chrome gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_sheet_demo_panel_chrome_matches`, `web_vs_fret_sheet_side_panel_chrome_matches`, `web_vs_fret_sheet_side_right_panel_chrome_matches`, `web_vs_fret_sheet_side_bottom_panel_chrome_matches`, `web_vs_fret_sheet_side_left_panel_chrome_matches`).
- Shadcn Web placement gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_sheet_demo_overlay_insets_match`, `web_vs_fret_sheet_side_top_overlay_insets_match`, `web_vs_fret_sheet_side_right_overlay_insets_match`, `web_vs_fret_sheet_side_bottom_overlay_insets_match`, `web_vs_fret_sheet_side_left_overlay_insets_match`).

## Follow-ups (recommended)

- None currently tracked.
