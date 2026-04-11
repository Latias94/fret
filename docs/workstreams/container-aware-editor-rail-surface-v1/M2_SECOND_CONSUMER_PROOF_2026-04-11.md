# M2 Second Consumer Proof — 2026-04-11

Status: landed proof note

Related:

- `DESIGN.md`
- `TARGET_INTERFACE_STATE.md`
- `M2_CONSUMER_AUDIT_2026-04-11.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`

## What changed

- `apps/fret-examples/src/editor_notes_demo.rs` now composes a left outline rail and a right
  inspector rail through `WorkspaceFrame.left(...)` and `WorkspaceFrame.right(...)`.
- `apps/fret-examples/tests/editor_notes_editor_rail_surface.rs` now locks that composition as a
  source-level gate.

## Why this now counts as a real shell-mounted rail consumer

Before this slice, `editor_notes_demo` reused `InspectorPanel` and `PropertyGroup`, but it still
stopped at inner editor-content reuse.

After this slice:

- the demo keeps reusable editor content on `fret-ui-editor`,
- it mounts that content through the existing workspace shell seam,
- and it does so without widening `Sidebar` or moving rail recipe policy into `fret-docking`.

That means the repo now has a second reviewable shell-mounted editor-rail composition that is
independent from `workspace_shell_demo`.

## Current verdict

The repo now has two real shell-mounted editor-rail consumers:

1. `apps/fret-examples/src/workspace_shell_demo.rs`
2. `apps/fret-examples/src/editor_notes_demo.rs`

This is enough to prove repeated composition through the existing shell slots.

It is not yet enough to promote a public `PanelRail` / `InspectorSidebar` surface, because the
lane still intentionally requires:

- shared container-aware behavior rather than only repeated fixed-slot composition,
- outer-shell ownership for any mobile/device-shell downgrade,
- and a narrower follow-on if extraction work is actually warranted.

## Focused gate

```bash
cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --no-fail-fast
```
