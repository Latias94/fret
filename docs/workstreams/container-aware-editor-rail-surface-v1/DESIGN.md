# Container-Aware Editor Rail Surface v1

Status: Closed closeout record
Last updated: 2026-04-11

Related:

- `CLOSEOUT_AUDIT_2026-04-11.md`
- `M0_BASELINE_AUDIT_2026-04-11.md`
- `TARGET_INTERFACE_STATE.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/EDITOR_PANEL_SURFACE_AUDIT_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/WORKSPACE_RAIL_SEAM_AUDIT_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/M3_EDITOR_RAIL_COMPOSITION_2026-04-10.md`
- `docs/audits/shadcn-sidebar.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

This follow-on exists because the closed adaptive-layout lane already answered two prerequisite
questions:

- app-shell `Sidebar` remains viewport/device-shell policy and must not widen into the editor-panel
  story,
- and the outer shell seam for editor rails already exists via `WorkspaceFrame.left(...)` /
  `WorkspaceFrame.right(...)`.

What remains open is narrower:

> if Fret wants a reusable editor rail / inspector sidebar later, what should the top-level
> interface actually look like, how should mobile vs desktop adaptation be split, and what proof is
> required before a new reusable public surface is promoted?

## In scope

- Freeze the owner split for a future reusable editor rail surface.
- Record the correct adaptive axis for editor rails: container/panel width first, not viewport
  width first.
- Record how outer app-shell mobile behavior should compose with an inner editor rail surface.
- Name the proof threshold required before a reusable public `PanelRail` / `InspectorSidebar`
  candidate is justified.
- Leave a runnable gate set for the current shell seam and boundary evidence.

## Out of scope

- Reopening shadcn `Sidebar` semantics or API growth.
- Adding a new public `PanelRail` / `InspectorSidebar` type in this lane before proof exists.
- Inventing a second shell-placement API beyond `WorkspaceFrame.left/right`.
- Moving dock topology ownership into shell chrome or recipe policy.
- Forcing mobile-sheet behavior into a generic editor rail component.

## Current hypothesis

The correct near-term design is:

- keep shell placement on `fret-workspace::WorkspaceFrame`,
- keep reusable inner panel content on `fret-ui-editor`,
- keep concrete rail recipe/state app-local until a second real consumer exists,
- keep container queries / panel adaptive classification as the primary adaptive axis,
- and let outer app-shell/device-shell surfaces decide whether the rail is mounted, collapsed, or
  replaced by a mobile sheet/drawer.

This means the lane closes without another `Sidebar` API or immediate public extraction.
Any future implementation work should reopen as a narrower follow-on only when shared
container-aware behavior has an explicit owner and proof plan.
