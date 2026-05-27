# Material 3 Tooltip Rich Parts Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

Task IDs use `M3TT-*`.

## M0 - Workstream Setup

- [x] M3TT-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-tooltip-rich-parts-packet-v1]
  Goal: Open the narrow tooltip follow-on and define the recipe/mechanism boundary.
  Validation: JSON and catalog gates.
  Review: DONE. The lane scope is limited to rich tooltip parts and wiring de-duplication; action
  interactivity remains a separate mechanism/ADR follow-on.
  Evidence: `DESIGN.md`.
  Handoff: Continue with M3TT-020.

## M1 - Rich Parts And Wiring

- [x] M3TT-020 [owner=codex] [deps=M3TT-010] [scope=ecosystem/fret-ui-material3/src/tooltip.rs,ecosystem/fret-ui-material3/tests/automation_surface.rs]
  Goal: Add `RichTooltip` title/supporting-text selectors and de-duplicate root/chrome semantics
  wiring shared with `PlainTooltip`.
  Validation: Focused automation-surface test proving plain and rich selector contracts.
  Review: DONE. RichTooltip text mode now exposes live `title` and `supporting-text` parts; a
  no-title rich tooltip does not synthesize a title part. PlainTooltip now reuses the shared tooltip
  policy root and old root/chrome selectors still pass.
  Evidence: `ecosystem/fret-ui-material3/src/tooltip.rs`; `ecosystem/fret-ui-material3/tests/automation_surface.rs`; `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_tooltip_and_snackbar_expose_stable_part_test_ids`.
  Handoff: Continue with M3TT-030.

## M2 - Gate And Matrix Closeout

- [x] M3TT-030 [owner=codex] [deps=M3TT-020] [scope=docs/workstreams/material3-tooltip-rich-parts-packet-v1,docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Update the Material3 matrix/packet evidence and close this lane with fresh gates.
  Validation: focused Rust gates, JSON/catalog gates, check/clippy.
  Review: DONE. Matrix and overlay packet now record the rich tooltip parts follow-on as resolved,
  while rich action interactivity remains a separate mechanism follow-on.
  Evidence: `CLOSEOUT_AUDIT_2026-05-28.md`.
  Handoff: Return to the broader Material3 goal and pick the next follow-on.
