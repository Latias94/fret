# Material 3 Snackbar Parts Selector Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

Task IDs use `M3SPS-*`.

## M0 - Workstream Setup

- [x] M3SPS-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-snackbar-parts-selector-packet-v1]
  Goal: Open the narrow Snackbar follow-on and define the kit/recipe boundary.
  Validation: JSON and catalog gates.
  Review: DONE. The work is a kit-level toast selector addition with Material-facing proof; no
  Material foundation or mechanism change is required.
  Evidence: `DESIGN.md`.
  Handoff: Continue with M3SPS-020.

## M1 - Kit Toast Part Selectors

- [x] M3SPS-020 [owner=codex] [deps=M3SPS-010] [scope=ecosystem/fret-ui-kit/src/window_overlays/render.rs,ecosystem/fret-ui-kit/src/window_overlays/tests/toast.rs]
  Goal: Derive action/cancel/close test ids from the toast root test id in the shared toast
  renderer.
  Validation: Focused kit toast semantics snapshot test.
  Review: DONE. The renderer now derives `<root>.action`, `<root>.cancel`, and `<root>.close` only
  for rendered affordances.
  Evidence: `cargo nextest run -p fret-ui-kit toast_action_cancel_and_close_test_ids_derive_from_root_test_id`.
  Handoff: Continue with M3SPS-030.

## M2 - Material Automation Proof And Matrix Closeout

- [x] M3SPS-030 [owner=codex] [deps=M3SPS-020] [scope=ecosystem/fret-ui-material3/tests/automation_surface.rs,docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Prove Material Snackbar root ids surface live action/close ids and close the matrix
  follow-on.
  Validation: Focused Material automation-surface test plus docs/matrix gates.
  Review: DONE. Material Snackbar remains a request skin; the shared kit renderer owns the action
  and close selectors.
  Evidence: `CLOSEOUT_AUDIT_2026-05-28.md`.
  Handoff: Return to the broader Material3 alignment goal.
