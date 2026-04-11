# Container-Aware Editor Rail Surface v1 — Milestones

Status: Active
Last updated: 2026-04-11

## M0 — Baseline

Exit criteria:

- The lane clearly states why it is a new follow-on.
- Existing owner split evidence is summarized with assumptions-first confidence.
- The lane names one smallest open question instead of reopening broad adaptive cleanup.

Primary evidence:

- `DESIGN.md`
- `M0_BASELINE_AUDIT_2026-04-11.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/EDITOR_PANEL_SURFACE_AUDIT_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/WORKSPACE_RAIL_SEAM_AUDIT_2026-04-10.md`

Current status:

- Active.
- Baseline recorded on 2026-04-11 via `M0_BASELINE_AUDIT_2026-04-11.md`.

## M1 — Target Freeze

Exit criteria:

- `TARGET_INTERFACE_STATE.md` freezes the shell seam, inner-content owner, adaptive-axis rule, and
  promotion threshold.
- The lane explicitly states how mobile/device-shell behavior composes with a future editor rail.

Primary evidence:

- `TARGET_INTERFACE_STATE.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

Current status:

- Closed on 2026-04-11 via `M1_CONTRACT_FREEZE_2026-04-11.md`.

## M2 — Proof and Gap Audit

Exit criteria:

- The active gate set for sidebar boundary + workspace rail seam is refreshed.
- The lane decides whether a second real consumer exists yet.
- The next executable slice is reduced to one narrow implementation or proof action.

Primary evidence:

- `EVIDENCE_AND_GATES.md`
- `apps/fret-ui-gallery/tests/sidebar_docs_surface.rs`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `M2_CONSUMER_AUDIT_2026-04-11.md`
- `M2_PANEL_RESIZE_GATE_ADOPTION_2026-04-11.md`
- `M2_SECOND_CONSUMER_PROOF_2026-04-11.md`

Current status:

- Closed on 2026-04-11.
- Initial audit: only `workspace_shell_demo` counted as a real shell-mounted editor-rail consumer
  before this slice.
- Current proof result: `editor_notes_demo` now provides the second app-local shell-mounted rail
  consumer through the same `WorkspaceFrame.left/right` seam while keeping reusable inner content
  on `fret-ui-editor`.
- The fixed-window panel-resize diagnostic gate remains part of the active proof set for future
  extraction decisions.

## M3 — Closeout / Follow-on Split

Exit criteria:

- The lane either closes on a frozen promotion threshold,
- or spins a narrower implementation follow-on with one explicit owner and gate set.

Primary evidence:

- future closeout note or follow-on lane docs

Current status:

- Closed on 2026-04-11 via `CLOSEOUT_AUDIT_2026-04-11.md`.
- The lane now closes on a frozen owner split plus two shell-mounted rail consumers.
- Public extraction remains deferred until a narrower follow-on owns shared container-aware
  behavior explicitly.
