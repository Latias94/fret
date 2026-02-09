# shadcn/ui v4 Audit - Alert Dialog (new-york)

This audit compares Fret's shadcn-aligned `AlertDialog` surface against the upstream shadcn/ui v4
docs and the `new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/alert-dialog.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert-dialog.tsx`
- Underlying primitive concept: Radix `@radix-ui/react-alert-dialog` (dialog + safety defaults)

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`
- Depends on overlay policy/infra:
  - `ecosystem/fret-ui-kit/src/window_overlays/*` (modal overlays, focus restore/initial focus)
  - `ecosystem/fret-ui-kit/src/overlay_controller.rs` (overlay requests)
  - `ecosystem/fret-ui-kit/src/primitives/alert_dialog.rs` (Radix-aligned cancel focus policy)
  - `ecosystem/fret-ui-kit/src/primitives/dialog.rs` (Radix-aligned modal request facade)
  - `crates/fret-ui/src/tree/*` (modal barrier scoping + focus traversal contract, ADR 0068)

## What upstream exports (new-york)

Upstream shadcn/ui exports a thin wrapper around Radix:

- `AlertDialog`
- `AlertDialogTrigger`
- `AlertDialogContent`
- `AlertDialogHeader`
- `AlertDialogFooter`
- `AlertDialogTitle`
- `AlertDialogDescription`
- `AlertDialogAction`
- `AlertDialogCancel`

## Audit checklist

### Composition surface

- Pass: Fret exposes `AlertDialog` as a recipe driven by a `Model<bool>` open state.
- Pass: Trigger/content composition matches the shadcn mental model.

### Dismissal behavior (safety defaults)

- Pass: Escape dismiss is supported (shared dismissible root).
- Pass: Overlay click-to-dismiss is **disabled by default** (Radix/shadcn safety behavior).
- Pass: Open lifecycle callbacks are available via `AlertDialog::on_open_change` and
  `AlertDialog::on_open_change_complete` (Base UI `onOpenChange` + `onOpenChangeComplete`).

### Focus behavior (safety defaults)

- Pass: Modal barrier scoping prevents underlay focus traversal (ADR 0068).
- Pass: On open, default initial focus prefers the first `AlertDialogCancel` button when present
  (Radix outcome).
- Pass: On close, focus restoration is deterministic to the trigger.

### Visual parity (new-york)

- Pass: Motion matches shadcn's `fade` + `zoom-in-95` / `zoom-out-95` outcomes (best-effort, tick
  driven).

## Validation

- `cargo check -p fret-ui-shadcn`
- `cargo nextest run -p fret-ui-shadcn alert_dialog::tests`
- Contract test: `alert_dialog_open_change_events_emit_change_and_complete_after_settle`
- Contract test: `alert_dialog_open_change_events_complete_without_animation`
- Shadcn Web chrome gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`
  (`web_vs_fret_alert_dialog_demo_panel_chrome_matches`).
- Shadcn Web placement gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`
  (`web_vs_fret_alert_dialog_demo_overlay_center_matches`).
- Radix Web overlay geometry gate: `cargo nextest run -p fret-ui-shadcn --test radix_web_overlay_geometry`
  (`radix_web_alert_dialog_open_geometry_matches_fret`).

## Follow-ups (recommended)

- Add optional presence/motion (if desired) while preserving safety semantics.
