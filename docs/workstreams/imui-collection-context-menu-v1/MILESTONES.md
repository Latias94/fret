# ImUi Collection Context Menu v1 - Milestones

Status: closed closeout record
Last updated: 2026-04-23

Status note (2026-04-23): this file now records the closed collection context-menu verdict only.
Active implementation should move to a different narrow lane if fresh first-party evidence exceeds
this closeout.

## M0 - Baseline and owner freeze

Exit criteria:

- the repo explicitly states why collection context-menu breadth is a new narrow follow-on instead
  of a reopened delete-action or generic menu/key-owner lane,
- the proof-budget rule remains explicit,
- and the owner split keeps the slice app-owned.

Primary evidence:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-23.md`
- `WORKSTREAM.json`

Current status:

- Completed on 2026-04-23.

## M1 - Land the bounded slice

Exit criteria:

- the collection-first proof demo now supports app-owned context-menu quick actions,
- the implementation remains app-owned,
- and the slice is covered by focused source-policy plus unit-test gates.

Primary evidence:

- `M1_APP_OWNED_CONTEXT_MENU_SLICE_2026-04-23.md`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_context_menu_surface.rs`
- `apps/fret-examples/src/lib.rs`

Current status:

- Completed on 2026-04-23.

## M2 - Closeout or split again

Exit criteria:

- the lane closes with explicit reopen policy,
- or another narrower follow-on is created instead of widening this folder.

Primary evidence:

- `CLOSEOUT_AUDIT_2026-04-23.md`
- `WORKSTREAM.json`
- `EVIDENCE_AND_GATES.md`

Current status:

- Closed on 2026-04-23.
