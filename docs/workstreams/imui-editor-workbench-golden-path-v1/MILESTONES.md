# IMUI Editor Workbench Golden Path v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Canonical Route Exists

Exit criteria:

- `imui_editor_workbench_demo` exists in `apps/fret-examples`.
- `fret-demo` exposes a direct `imui_editor_workbench_demo` binary.
- The route delegates to the current workspace shell proof until a bounded convergence task moves
  content.
- Focused source-policy tests pass.

## M1 - First-Open Docs And Discovery

Exit criteria:

- Docs name `imui_editor_workbench_demo` as the canonical editor workbench route.
- Cookbook and examples guidance classify old demos as focused/supporting proof surfaces.
- Product-chain discovery can find the canonical route.

## M2 - Coherent Workbench Workflow

Exit criteria:

- The canonical route demonstrates a connected editor workflow across shell, center content,
  inspector/editor controls, collection-like navigation, and diagnostics-friendly test ids.
- Focused proof demos remain smaller supporting surfaces.

## M3 - Diagnostics And Perf Discoverability

Exit criteria:

- Demo/Metrics/Debug discovery can launch or point to the canonical route.
- Perf and diagnostics evidence stays in `fret-diag` / DevTools owner lanes.

## M4 - Completion Read

Exit criteria:

- A closeout audit proves which parts of the Dear ImGui-class objective are closed by this lane and
  which remain in owner follow-ons.
- The lane does not claim real-host Wayland, broad DevTools productization, or broad perf smoothness
  unless their owner evidence proves completion.

## M5 - Proof-Led API Follow-On Split

Exit criteria:

- ListBox, plot adapter, style/theme editor, and porting sugar are re-evaluated from current proof
  pressure.
- Accepted ListBox container, plot adapter, and style/theme preset picker candidates move to narrow
  owner lanes with gates.
- Deferred collection-helper widening and porting sugar keep explicit evidence-backed reasons
  instead of becoming speculative public IMUI API.
- The canonical workbench inspector exposes the accepted editor-owned style/theme preset picker.
