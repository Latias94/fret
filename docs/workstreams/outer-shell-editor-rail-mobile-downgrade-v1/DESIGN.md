# Outer-Shell Editor Rail Mobile Downgrade v1

Status: closed closeout record
Last updated: 2026-04-11

Related:

- `M0_BASELINE_AUDIT_2026-04-11.md`
- `CLOSEOUT_AUDIT_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/device-shell-strategy-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/workstreams/container-aware-editor-rail-helper-shape-v1/CLOSEOUT_AUDIT_2026-04-11.md`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

This follow-on owns one narrow question:

> when an editor-oriented shell that mounts a container-aware rail also targets mobile or compact
> devices, which layer should own the downgrade from desktop rail to drawer/sheet/route, and
> should that downgrade be extracted into a shared helper yet?

## In scope

- Freeze the owner split for editor-rail mobile downgrade.
- Audit whether the current repo evidence justifies a shared downgrade helper.
- Record the correct relationship between:
  - outer-shell device/mobile branching,
  - shell-mounted rail seams,
  - and inner container-aware rail content.

## Out of scope

- Public `PanelRail` / `InspectorSidebar` extraction.
- Reopening `Sidebar` into a generic editor-rail API.
- Moving device-shell strategy into `crates/fret-ui`.
- Picking one mandatory downgrade presentation (`Drawer`, `Sheet`, route, or stack page) for all
  apps.

## Current hypothesis

The correct answer is still explicit outer-shell ownership, not a shared helper.

Why:

- `device_shell_switch(...)` already owns the generic desktop/mobile branch helper at the strategy
  layer when apps want it,
- `WorkspaceFrame.left/right` already owns desktop shell mounting,
- and no current editor-rail proof surface shows repeated mobile downgrade behavior that is more
  specific than "the outer shell remounts or replaces the rail when the device shell changes."
