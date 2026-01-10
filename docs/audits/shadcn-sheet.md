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
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) via
  `Sheet::on_dismiss_request(...)`. When set, Escape and overlay-click dismissal route through the
  handler and do not automatically close `open`.

### Focus behavior

- Pass: Modal barrier scoping prevents underlay focus traversal (ADR 0068).
- Pass: On open, initial focus falls back to the first focusable descendant within the modal root
  (via `window_overlays` focus helpers).
- Pass: On close, focus restoration is deterministic to the trigger (modal close unmount path).

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn sheet::tests`

## Follow-ups (recommended)

- None currently tracked.
