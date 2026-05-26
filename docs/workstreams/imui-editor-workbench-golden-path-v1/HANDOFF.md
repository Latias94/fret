# IMUI Editor Workbench Golden Path v1 - Handoff

Status: Closed
Last updated: 2026-05-25

Current slice: closed on 2026-05-25.

EWG-010 status: complete. The canonical route exists, the direct demo binary is wired, and the focused gates have fresh evidence.

EWG-020 status: complete. Cookbook/docs, `list tool-apps`, DevTools GUI, DevTools MCP, and product-chain discovery now promote `imui_editor_workbench_demo` as the canonical workbench while keeping older proof demos as supporting surfaces.

EWG-030 status: complete. `imui_editor_workbench_demo` now owns the app shell and mounts the reusable `editor_notes_demo::EditorNotesDemoView` workflow directly, so the canonical route is no longer just a shell forwarder.

EWG-040 status: complete. `docs/workstreams/imui-demo-metrics-debug-devtools-v1/` now owns the
DevTools/diagnostics product surface for the Fret equivalent of Dear ImGui `ShowDemoWindow` /
Metrics / Debug. CLI, DevTools GUI, and MCP discovery expose the same owner doc and keep the
canonical editor workbench first.

EWG-050 status: complete. Docking and runner hand-feel remain owned by
`docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`. The Demo/Metrics/Debug
discovery route now links to that owner, the bounded `imui-p3-multiwindow-parity` campaign validate
command, and the real-host Wayland acceptance runbook while keeping local policy-skip evidence
separate.

EWG-060 status: complete. ListBox container, plot adapter, and style/theme preset picker work moved
to narrow owner lanes. `docs/workstreams/imui-list-box-container-proof-v1/` owns only the
BeginListBox-style container, while collection-helper widening stays deferred because the
collection-helper closeout found no repeated reusable-helper pressure.
`docs/workstreams/imui-plot-adapter-proof-v1/` keeps `fret-plot/imui` optional, declarative-only,
and outside `fret-imui` / `fret-ui-kit::imui`.
`docs/workstreams/imui-style-theme-editor-proof-v1/` owns the editor theme preset picker and the
canonical editor-notes inspector now exposes it through `editor-notes-demo.inspector.theme-preset`.
Porting sugar stays deferred until two product routes need the same shorthand. The child lane gates
now pass.

EWG-070 status: complete. The next high-payoff private split is
`docs/workstreams/imui-table-header-owner-split-v1/`. `table_controls/header.rs` now owns IMUI
table header trigger/sort/resize behavior while public table authoring names stay stable, and the
child lane gates pass.

Closeout:

1. The canonical workbench route and EWG-010 through EWG-070 evidence are closed.
2. Future workbench depth, helper sugar, DevTools execution controls, perf attribution, or
   runner/backend work should start in the relevant owner lane or a new proof-led follow-on.
3. Real Wayland compositor acceptance remains open only in `docking-multiwindow-imgui-parity`.
