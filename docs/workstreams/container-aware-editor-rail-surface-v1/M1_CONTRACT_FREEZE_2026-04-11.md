# M1 Contract Freeze — 2026-04-11

Status: accepted decision note

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `M0_BASELINE_AUDIT_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/EDITOR_PANEL_SURFACE_AUDIT_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/WORKSPACE_RAIL_SEAM_AUDIT_2026-04-10.md`

## Question

Before implementation continues, which interface decisions are already stable enough that this lane
should stop revisiting them?

## Decision

Freeze the following rules for this lane:

1. `Sidebar` remains an app-shell / device-shell surface.
2. `WorkspaceFrame.left/right` remains the outer shell seam for editor rails.
3. Reusable inner editor panel content remains `fret-ui-editor` territory.
4. A future reusable editor rail must be container-aware first, not viewport-first.
5. Mobile/device-shell downgrade policy stays at the outer shell boundary, not inside a generic
   reusable editor rail component.
6. No public `PanelRail` / `InspectorSidebar` extraction happens before a second real consumer plus
   a panel-resize proof.

## Immediate consequence

This lane should no longer spend time on:

- widening `SidebarProvider::is_mobile(...)`,
- inventing a new shell placement API,
- or debating whether `fret-docking` should own rail recipe policy.

The remaining open work is narrower:

- refresh the current proof set,
- decide whether a second rail consumer exists yet,
- and choose the smallest implementation follow-on only after that threshold is explicit.
