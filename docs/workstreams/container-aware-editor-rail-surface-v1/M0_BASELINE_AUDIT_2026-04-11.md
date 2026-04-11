# M0 Baseline Audit — 2026-04-11

Status: active baseline note

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/EDITOR_PANEL_SURFACE_AUDIT_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/WORKSPACE_RAIL_SEAM_AUDIT_2026-04-10.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/M3_EDITOR_RAIL_COMPOSITION_2026-04-10.md`

## Assumptions-first baseline

### 1) This must be a new follow-on, not a reopened adaptive lane

- Area: lane state
- Assumption: the broad adaptive-layout lane is already closed and explicitly says future editor
  rail work should start as a narrower follow-on.
- Evidence:
  - `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`
- Confidence: Confident
- Consequence if wrong: new work would blur a threshold/surface question back into a closed lane.

### 2) `Sidebar` is already frozen as app-shell/device-shell policy

- Area: existing sidebar surface
- Assumption: `SidebarProvider::is_mobile(...)` / `is_mobile_breakpoint(...)` are acceptable only
  as app-shell vocabulary and must not become the generic editor-rail answer.
- Evidence:
  - `docs/audits/shadcn-sidebar.md`
  - `apps/fret-ui-gallery/src/ui/pages/sidebar.rs`
  - `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`
- Confidence: Confident
- Consequence if wrong: this follow-on would start from the wrong owner layer and repeat the same
  query-axis drift.

### 3) The outer shell seam already exists

- Area: shell ownership
- Assumption: `WorkspaceFrame.left/right` already provides the reusable outer placement seam for a
  future editor rail.
- Evidence:
  - `docs/workstreams/adaptive-layout-contract-closure-v1/WORKSPACE_RAIL_SEAM_AUDIT_2026-04-10.md`
  - `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- Confidence: Confident
- Consequence if wrong: the lane would spend time inventing a second shell slot abstraction that
  the repo does not need.

### 4) Reusable inner rail content already belongs above shadcn recipes

- Area: inner editor content ownership
- Assumption: reusable inspector/property-grid content should stay on `fret-ui-editor`, not on a
  widened shadcn sidebar family.
- Evidence:
  - `docs/workstreams/adaptive-layout-contract-closure-v1/EDITOR_PANEL_SURFACE_AUDIT_2026-04-10.md`
  - `docs/workstreams/adaptive-layout-contract-closure-v1/M3_EDITOR_RAIL_COMPOSITION_2026-04-10.md`
- Confidence: Confident
- Consequence if wrong: the lane would duplicate editor content policy across mismatched layers.

### 5) The smallest open question is extraction threshold, not missing mechanism

- Area: next executable slice
- Assumption: the repo already has the mechanism/seam proof it needs; what is still missing is a
  frozen threshold for when a public reusable editor-rail surface is warranted.
- Evidence:
  - `docs/workstreams/adaptive-layout-contract-closure-v1/M3_EDITOR_RAIL_COMPOSITION_2026-04-10.md`
  - `docs/workstreams/adaptive-layout-contract-closure-v1/WORKSPACE_RAIL_SEAM_AUDIT_2026-04-10.md`
- Confidence: Likely
- Consequence if wrong: the lane might stay too abstract and miss a real implementation bottleneck.

## Immediate implication

This lane should not begin by adding a new public API.

It should begin by freezing three things:

1. shell seam stays `WorkspaceFrame.left/right`,
2. container-aware adaptive behavior belongs to the inner rail recipe, not to app-shell mobile
   inference,
3. and any future `PanelRail` / `InspectorSidebar` promotion needs a second real consumer plus a
   panel-resize proof.
