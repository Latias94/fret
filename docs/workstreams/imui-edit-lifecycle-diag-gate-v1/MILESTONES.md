# ImUi Edit Lifecycle Diag Gate v1 Milestones

Status: closed
Last updated: 2026-04-24

## M0 - Gate Shape

Exit criteria:

- The response-signals demo exposes stable lifecycle readouts for slider and text editing.
- The new script drives real edit interactions and asserts counters through `test_id` selectors.
- The script has a dedicated suite that launches `imui_response_signals_demo`.

Result: complete.

## M1 - Editor-Proof Drift Repair

Exit criteria:

- Existing editor-proof drag/text/numeric scripts match current demo behavior.
- Scripts do not click offscreen targets; full-layout transitions scroll targets into view.
- Demo-local semantics are corrected where the script exposed real proof drift.

Result: complete.

## M2 - Closeout

Exit criteria:

- Both promoted suites pass with `--launch`.
- Registry checks include the new scripts/suites.
- Workstream docs name the repro, gates, and evidence.
- Public IMUI/runtime APIs remain unchanged.

Result: complete.
