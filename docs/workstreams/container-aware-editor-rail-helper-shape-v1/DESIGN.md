# Container-Aware Editor Rail Helper Shape v1

Status: closed closeout record
Last updated: 2026-04-11

Related:

- `M0_BASELINE_AUDIT_2026-04-11.md`
- `CLOSEOUT_AUDIT_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

This follow-on starts from a narrower post-closeout question:

> now that the repo has two shell-mounted editor-rail consumers, is there a real shared helper
> shape below any future public `PanelRail` extraction, or should the current wrapper layer stay
> explicit and app-local?

## In scope

- Audit the repeated shell-mounted rail shape across `workspace_shell_demo` and
  `editor_notes_demo`.
- Separate already-owned shared mechanism from still-divergent wrapper policy.
- Decide whether any new helper owner is justified yet.
- Record reopen criteria if the answer is still "no new helper yet".

## Out of scope

- Public `PanelRail` / `InspectorSidebar` promotion.
- Reopening `Sidebar` semantics or mobile/device-shell ownership.
- Moving rail recipe policy into `fret-docking`.
- Replacing `WorkspaceFrame.left/right` with a new shell seam.

## Current hypothesis

The likely answer is still no new helper yet.

Why:

- the repeated outer seam is already owned by `WorkspaceFrame.left/right`,
- the repeated inner editor content is already owned by `fret-ui-editor`,
- and the remaining wrapper layer still differs in width, chrome, border treatment, and local
  recipe policy across the two current consumers.

If that holds under audit, this lane should close on a no-new-helper verdict rather than forcing a
premature extraction.
