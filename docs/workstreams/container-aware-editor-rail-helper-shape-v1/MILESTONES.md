# Container-Aware Editor Rail Helper Shape v1 — Milestones

Status: Closed closeout lane
Last updated: 2026-04-11

## M0 — Baseline

Exit criteria:

- The two current shell-mounted rail consumers are audited side by side.
- The lane states which parts are already owned and which parts would need a new helper owner.

Primary evidence:

- `M0_BASELINE_AUDIT_2026-04-11.md`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `ecosystem/fret-workspace/src/frame.rs`

Current status:

- Closed on 2026-04-11 via `M0_BASELINE_AUDIT_2026-04-11.md`.

## M1 — Verdict

Exit criteria:

- The lane decides whether a shared helper shape exists yet.
- The lane closes if the current answer is still "no new helper yet".

Primary evidence:

- `CLOSEOUT_AUDIT_2026-04-11.md`
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs`

Current status:

- Closed on 2026-04-11.
- Verdict: no new helper yet. The repeated reusable pieces are already owned by
  `WorkspaceFrame.left/right` and `fret-ui-editor`, while the remaining wrapper layer is still too
  policy-specific and divergent.
