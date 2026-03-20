# shadcn/ui v4 Audit - Sheet (new-york)

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Sheet` surface against the upstream shadcn/ui v4 docs
and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/sheet.mdx`
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
- `SheetClose`
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
- Pass: `SheetContent` renders a default close affordance and exposes
  `show_close_button(false)`, matching upstream `showCloseButton={false}`.
- Pass: `SheetClose::from_scope()` is available as recipe-layer sugar for additional content-local
  close buttons while preserving `SheetClose::new(open)` as the explicit constructor.
- Pass: `Sheet::compose()` provides a recipe-level builder for part assembly without pushing
  shadcn-specific composition concerns into the lower-level mechanism contract.
- Pass: `SheetContent::build(...)` is the typed content-side companion on that same recipe lane,
  so first-party snippets no longer need to pre-land `SheetHeader` / `SheetFooter` trees into a
  raw `SheetContent::new([...])` array.

### Placement & sizing

- Pass: `Sheet::new(open)` defaults to `Right`, matching the upstream component default.
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

## Authoring note: `compose()`

`Sheet::compose()` is a recipe-layer bridge for authors who want a more composable part-based
style than the raw closure root.

- Scope: ergonomics only; it lowers into `Sheet::into_element_parts(...)`.
- Default teaching path: first-party examples now prefer
  `Sheet::new_controllable(cx, None, false).compose().trigger(...).content_with(...)`.
- Focused follow-up lane: explicit `SheetTrigger` / `SheetPortal` / `SheetOverlay` ownership stays
  documented through a dedicated `Parts` example rather than replacing the default copyable path.
- Layering: it does **not** change the underlying overlay/focus/dismiss mechanism.
- Limitation: this is still not a full React-style nested children API; Fret stores already-built
  elements and assembles them at the final call site.

## Follow-ups (recommended)

- None currently tracked.
