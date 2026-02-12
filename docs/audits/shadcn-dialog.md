# shadcn/ui v4 Audit - Dialog (new-york)

This audit compares Fret's shadcn-aligned `Dialog` surface against the upstream shadcn/ui v4 docs
and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/dialog.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/dialog.tsx`
- Underlying primitive concept: Radix `@radix-ui/react-dialog` (portal + dismiss + focus management)

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/dialog.rs`
- Depends on overlay policy/infra:
  - `ecosystem/fret-ui-kit/src/window_overlays/*` (modal overlays, focus restore/initial focus)
  - `ecosystem/fret-ui-kit/src/overlay_controller.rs` (overlay requests + presence)
  - `ecosystem/fret-ui-kit/src/primitives/dialog.rs` (Radix-aligned trigger a11y + request facade)
  - `crates/fret-ui/src/tree/*` (modal barrier scoping + focus traversal contract, ADR 0068)

## What upstream exports (new-york)

Upstream shadcn/ui exports a thin wrapper around Radix:

- `Dialog`
- `DialogTrigger`
- `DialogContent`
- `DialogHeader`
- `DialogFooter`
- `DialogTitle`
- `DialogDescription`

## Audit checklist

### Composition surface

- Pass: Fret exposes `Dialog` as a recipe driven by a `Model<bool>` open state.
- Pass: Trigger/content composition matches the shadcn mental model.
- Pass: Content is rendered via a per-window overlay root (portal-like), so it is not clipped by
  underlay layout/overflow.
- Pass: `DialogClose` is available as an explicit close affordance recipe (close button parity).

### Dismissal behavior

- Pass: Escape dismiss is handled by the shared dismissible root (Radix-aligned outcome).
- Pass: Overlay click-to-dismiss is implemented by rendering a full-window barrier behind the
  content (default on).
- Pass: Base UI-compatible convenience alias `disable_pointer_dismissal(bool)` is provided and
  maps directly to `overlay_closable(!disable)`.
- Pass: Dismissals can be intercepted (Radix `DismissableLayer` "preventDefault" outcome) via
  `Dialog::on_dismiss_request(...)`. When set, Escape and overlay-click dismissal route through the
  handler and do not automatically close `open`.
- Pass: Open lifecycle callbacks are available via `Dialog::on_open_change` and
  `Dialog::on_open_change_complete` (Base UI `onOpenChange` + `onOpenChangeComplete`).

### Focus behavior

- Pass: Modal barrier scoping prevents underlay focus traversal (ADR 0068).
- Pass: On open, initial focus falls back to the first focusable descendant within the modal root
  (via `window_overlays` focus helpers).
- Pass: On close, focus restoration is deterministic to the trigger (modal close unmount path).

### Visual parity (new-york)

- Pass: Motion matches shadcn's `fade` + `zoom-in-95` / `zoom-out-95` outcomes (best-effort, tick
  driven).
- Pass: Default sizing matches the upstream `w-full max-w-[calc(100%-2rem)] sm:max-w-lg` intent via
  a padded center layout + `DialogContent` max-width.

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn dialog::tests`
- Contract test: `dialog_disable_pointer_dismissal_alias_maps_overlay_closable`
- Contract test: `dialog_open_change_events_emit_change_and_complete_after_settle`
- Contract test: `dialog_open_change_events_complete_without_animation`
- Shadcn Web chrome gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_dialog_demo_panel_chrome_matches`).
- Shadcn Web placement gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_dialog_demo_overlay_center_matches`).
- Radix Web overlay geometry gate: `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
  (`radix_web_dialog_open_geometry_matches_fret`).

## Follow-ups (recommended)

- Consider exposing optional per-surface motion variants if recipes need diverging durations/easing.
