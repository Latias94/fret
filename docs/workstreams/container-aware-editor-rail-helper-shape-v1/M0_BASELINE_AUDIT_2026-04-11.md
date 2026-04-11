# M0 Baseline Audit — 2026-04-11

Status: baseline audit

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `docs/workstreams/container-aware-editor-rail-surface-v1/CLOSEOUT_AUDIT_2026-04-11.md`

## Assumptions-first read

### 1) The shared shell seam is already extracted

- Evidence:
  - `ecosystem/fret-workspace/src/frame.rs`
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - this lane would be auditing the wrong layer, because the repo would still be missing the real
    shell-placement abstraction.

### 2) The repeated inner editor content already has an owner

- Evidence:
  - `fret_ui_editor::composites::InspectorPanel`
  - `fret_ui_editor::composites::PropertyGroup`
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would need to revisit `fret-ui-editor` ownership instead of auditing wrapper shape.

### 3) The remaining repeated layer is mostly wrapper chrome, not shared behavior

- Evidence:
  - `workspace_shell_demo` uses a fixed-width bordered container around the right rail.
  - `editor_notes_demo` uses a left card rail plus a separate right rail width.
  - no shared collapse, resize policy, or container-class decision helper appears in either demo.
- Confidence:
  - Confident
- Consequence if wrong:
  - a real helper may already be visible and this lane should freeze its shape explicitly.

## Audited consumers

### 1) `workspace_shell_demo`

- What repeats:
  - `WorkspaceFrame.right(...)`
  - `InspectorPanel`
  - `PropertyGroup` / `PropertyGrid`
- What is still local:
  - right-rail width (`320px`)
  - outer container border/background
  - selection/shell group content

### 2) `editor_notes_demo`

- What repeats:
  - `WorkspaceFrame.left(...)` / `WorkspaceFrame.right(...)`
  - right-side `InspectorPanel`
  - editor-owned metadata content
- What is still local:
  - left-rail structure and outline buttons
  - left/right widths (`256px` / `360px`)
  - center card content
  - no shared outer border/chrome pattern with `workspace_shell_demo`

## Baseline verdict

The current repo evidence does not show a reusable helper shape yet.

What repeats is already split cleanly between:

- `WorkspaceFrame` for shell placement,
- and `fret-ui-editor` for reusable inner editor content.

What remains divergent is exactly the layer that a new helper would need to own:

- slot width policy,
- chrome/border treatment,
- left-vs-right asymmetry,
- and app-local content composition.
