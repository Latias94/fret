# M2 Consumer Audit — 2026-04-11

Status: active audit note

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `M1_CONTRACT_FREEZE_2026-04-11.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## Question

Does the repo already have a second real editor-rail consumer, or does it only have repeated inner
editor content reuse so far?

## Audited candidates

### 1) `workspace_shell_demo`

- Evidence:
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
- Verdict:
  - Yes, this is the current first real editor-rail proof.
- Why:
  - It mounts a rail through `WorkspaceFrame.right(...)`.
  - It composes a concrete right-side shell region rather than only reusing inner editor widgets.

### 2) `editor_notes_demo`

- Evidence:
  - `apps/fret-examples/src/editor_notes_demo.rs`
- Verdict:
  - Not a second rail consumer yet.
- Why:
  - It reuses `InspectorPanel` and `PropertyGroup`, but the audited surface is an inner editor
    panel, not a shell-mounted left/right rail recipe.

### 3) `imui_editor_proof_demo`

- Evidence:
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
- Verdict:
  - Not a second rail consumer yet.
- Why:
  - It proves shared editor content/composite reuse across authoring modes, but it does not mount
    that content through `WorkspaceFrame.left/right` or another equivalent shell seam.

## Decision

The lane should currently treat the extraction threshold as **not yet met**.

What exists today is:

- one real editor-rail consumer (`workspace_shell_demo`),
- plus multiple examples of reusable inner editor content (`InspectorPanel`, `PropertyGroup`,
  `PropertyGrid`) that still do not prove a second shared shell-level rail recipe.

## Consequence

This lane should not promote a public `PanelRail` / `InspectorSidebar` type yet.

The next meaningful execution options are now reduced to:

1. promote the panel-resize diagnostic proof into the active gate set for this lane,
2. or land a second app-local shell-mounted rail consumer through the existing workspace shell seam.
