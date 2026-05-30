# Material 3 Snackbar Parts Selector Packet v1

Status: closed
Date: 2026-05-28

## Truth

- Toast action, cancel, and close affordances are separate headless parts in the shared toast layer.
- Material Snackbar should inherit action/close selectors by forwarding its root id into
  `ToastRequest`.
- Live-region, dismissal, and action routing remain kit policy.

## Artifacts

- `ecosystem/fret-ui-kit/src/window_overlays/render.rs`
- `ecosystem/fret-ui-kit/src/window_overlays/tests/toast.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-snackbar-parts-selector-packet-v1/`

## Wiring

- `ToastRequest::test_id("toast")` derives:
  - `toast.action` for the rendered action button,
  - `toast.cancel` for the rendered cancel button,
  - `toast.close` for the rendered close affordance.
- `Snackbar::test_id("snackbar")` still only sets the toast root id; the kit renderer supplies the
  subpart ids.

## Proof

- `cargo nextest run -p fret-ui-kit toast_action_cancel_and_close_test_ids_derive_from_root_test_id`
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tooltip_and_snackbar_expose_stable_part_test_ids`

## Residual Risk

No message/supporting-text toast part selectors were added. That should be split only if a consumer
needs subpart automation for toast text content.
