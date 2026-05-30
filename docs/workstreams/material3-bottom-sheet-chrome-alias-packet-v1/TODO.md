# Material 3 BottomSheet Chrome Alias Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

Task IDs use `M3BS-*`.

## M0 - Source And Gap Packet

- [x] M3BS-010 [owner=codex] [deps=none] [scope=docs/workstreams/material3-bottom-sheet-chrome-alias-packet-v1,docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_overlay_feedback_packet_v1.md]
  Goal: Record why the chrome aliases were withheld and why hidden diagnostic anchors are now the
  right recipe-layer solution.
  Validation: packet artifact and current-state source anchors.
  Review: DONE. Prior visible/layout-sensitive markers changed bottom-sheet sizing; hidden
  diagnostic anchors are the correct recipe/foundation solution.
  Evidence: `artifacts/bottom_sheet_chrome_alias_packet_v1.md`.
  Handoff: Do not add visible/layout-participating markers.

## M1 - Layout-Safe Chrome Aliases

- [x] M3BS-020 [owner=codex] [deps=M3BS-010] [scope=ecosystem/fret-ui-material3/src/bottom_sheet.rs,ecosystem/fret-ui-material3/tests/automation_surface.rs]
  Goal: Add `bottom_sheet.chrome` and `modal_bottom_sheet.sheet.chrome` using hidden diagnostic
  anchors.
  Validation: focused automation-surface red/green test.
  Review: DONE. A single hidden full-region anchor in `DockedBottomSheet` covers docked and modal
  sheet chrome aliases.
  Evidence: `material3_dialog_and_bottom_sheet_expose_stable_part_test_ids`.
  Handoff: Keep modal scrim/sheet/drag-handle ids unchanged.

## M2 - Golden Guard And Closeout

- [x] M3BS-030 [owner=codex] [deps=M3BS-020] [scope=ecosystem/fret-ui-material3/tests/radio_alignment.rs,docs/workstreams/material3-bottom-sheet-chrome-alias-packet-v1]
  Goal: Prove the aliases do not change bottom-sheet scene output and close the lane.
  Validation: bottom-sheet headless golden gate, check/clippy, JSON/catalog.
  Review: DONE. Bottom-sheet headless goldens pass without refresh; focused gates pass.
  Evidence: `CLOSEOUT_AUDIT_2026-05-28.md`.
  Handoff: Return to broader Material3 follow-on selection.
