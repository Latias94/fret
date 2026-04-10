# M3 Editor-Rail Composition — 2026-04-10

Status: accepted execution note
Last updated: 2026-04-10

This note records the first reviewable editor-rail composition that follows the owner split frozen
earlier in this lane:

- outer shell placement uses `fret-workspace::WorkspaceFrame.right(...)`,
- inner panel content uses `fret-ui-editor` composites,
- and no new public `PanelRail` / `InspectorSidebar` primitive is introduced.

## Goal

Complete `ALC-044` with the smallest landable proof that the repo can already compose an
editor-grade side rail without widening shadcn `Sidebar` or adding a new shell primitive.

## Applied slice

The `workspace_shell_demo` now mounts a right-side editor rail through the existing shell seam.

The rail composition is intentionally narrow:

- shell slot and width ownership stay in `WorkspaceFrame.right(...)`,
- the inner recipe uses `InspectorPanel + PropertyGroup + PropertyGrid`,
- and the content reads current workspace-shell state such as active pane, active tab, dirty-tab
  count, tabstrip mode, and dirty-close prompt state.

This keeps the demo reviewable while proving the owner split in running code.

## Commands used

```bash
cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --no-fail-fast
cargo check -p fret-demo --bin workspace_shell_demo --message-format short
```

## Result

Promotion succeeded.

Passing evidence:

- `apps/fret-examples/src/workspace_shell_demo.rs`
  - mounts the rail through `WorkspaceFrame.right(...)` and keeps the inner content on editor
    composites.
- `apps/fret-examples/tests/workspace_shell_editor_rail_surface.rs`
  - keeps the owner seam explicit at the source level.
- `docs/workstreams/adaptive-layout-contract-closure-v1/WORKSPACE_RAIL_SEAM_AUDIT_2026-04-10.md`
  - remains the audit note that predicted this exact composition path.

## Consequence for this lane

`ALC-044` is now considered complete.

That means the M3 bounded slice queue is closed for this lane:

1. the owner split is documented,
2. the panel-resize and query-axis proof surfaces are active,
3. and one reviewable editor-rail composition now exists on the chosen shell seam.

The next work returns to M4 closeout/follow-on decisions.
