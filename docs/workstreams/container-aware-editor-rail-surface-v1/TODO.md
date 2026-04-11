# Container-Aware Editor Rail Surface v1 — TODO

Status: Active
Last updated: 2026-04-11

Companion docs:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-11.md`
- `TARGET_INTERFACE_STATE.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## M0 — Baseline

- [x] CAER-001 Record why this must be a new follow-on instead of reopening the closed adaptive
  lane.
- [x] CAER-002 Audit current owner split across `Sidebar`, `WorkspaceFrame`, `fret-ui-editor`, and
  `fret-docking`.
- [x] CAER-003 Freeze the smallest open question for this lane: extraction threshold and top-level
  interface shape for a future reusable editor rail.

## M1 — Target Freeze

- [x] CAER-010 Freeze the target interface state for outer shell seam vs inner editor content.
- [x] CAER-011 Freeze the adaptive-axis rule: container-first for the rail, device-shell at the
  outer shell boundary.
- [x] CAER-012 Freeze the promotion threshold for any future public `PanelRail` /
  `InspectorSidebar` candidate.

## M2 — Proof and Gap Audit

- [x] CAER-020 Refresh the current proof set for:
  - `Sidebar` staying app-shell only,
  - `WorkspaceFrame.left/right` staying the shell seam,
  - and `workspace_shell_demo` remaining the first reviewable rail composition.
- [x] CAER-021 Decide whether the next executable slice is:
  - a second app-local rail consumer,
  - a panel-resize diagnostic promotion,
  - or a small editor-owned helper below any future public extraction.
  - Decision on 2026-04-11: land the second app-local rail consumer through
    `apps/fret-examples/src/editor_notes_demo.rs` and lock it with
    `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`.

## M3 — Closeout or Follow-on Split

- [x] CAER-030 Close this lane if the target state and promotion threshold are frozen.
  - Closed on 2026-04-11 via `CLOSEOUT_AUDIT_2026-04-11.md`.
- [x] CAER-031 If implementation work is warranted, split a narrower follow-on instead of letting
  this lane drift into a broad editor-shell refactor.
  - Current closeout verdict: not warranted yet; keep any future helper/extraction work in a new
    narrower follow-on.
