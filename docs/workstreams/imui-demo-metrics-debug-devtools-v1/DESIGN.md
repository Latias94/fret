# IMUI Demo Metrics Debug DevTools v1 - Design

Status: Closed
Last updated: 2026-05-25

## Problem

Dear ImGui makes its capability surface discoverable through always-available demo, metrics, and
debug windows. Fret should have the same operational experience, but the owner is different:

- `imui_editor_workbench_demo` is the product workbench route.
- `fretboard-dev list tool-apps` is the CLI discovery index.
- `apps/fret-devtools` is the human GUI owner.
- `apps/fret-devtools-mcp` is the AI/client automation owner.
- `fret-imui` stays a thin facade and must not gain a product diagnostics workspace.

This lane owns the Fret equivalent of the Dear ImGui `ShowDemoWindow` / Metrics / Debug route as a
diagnostics product surface.

## Owner Boundary

In scope:

- Keep the `demo-metrics-debug` route visible from CLI, GUI, and MCP.
- Make the route point at the canonical editor workbench first.
- Keep supporting proof demos visible but clearly supporting.
- Keep metrics and debug commands aligned with shared diagnostics artifacts.
- Surface the owner workstream in discovery so the route is maintainable.
- Surface the docking handoff owner without reopening docking hand-feel work here.

Out of scope:

- Moving diagnostic panels, metrics browsers, or debug drill-down UI into `fret-imui`.
- Widening IMUI public helpers without two proof surfaces and a separate owner lane.
- Replacing `fretboard-dev` diagnostics commands with app-local ad hoc command strings.

## Current Route Contract

The product route is `demo-metrics-debug`.

Action surface decision:

- DevTools GUI owns a dedicated `Demo / Metrics / Debug Routes` guide panel with a
  copyable action command bundle.
- MCP mirrors the same action list in `fret-diag://first-open.md`.
- `fretboard-dev list tool-apps --json` exposes the same actions as machine-readable
  `action_commands`.
- A command palette is deferred until DevTools has a shared command palette contract; this lane
  should not create a one-off palette just for IMUI diagnostics.

Demo commands:

- `demo editor workbench`: `cargo run -p fret-demo --bin imui_editor_workbench_demo`
- supporting demos stay discoverable only as supporting evidence.

Metrics commands:

- `diag stats`
- `diag layout-perf-summary`
- `diag memory-summary`

Debug commands:

- `diag triage`
- `diag hotspots`
- `diag trace`

Docking handoff commands:

- `docking arbitration supporting`: `cargo run -p fret-demo --bin docking_arbitration_demo`
- `docking campaign validate`: `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`
- `docking policy-skip local`: `python tools/diag_gate_docking_wayland_policy_skip.py`

The route owner doc is:

- `docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json`

The docking handoff owner is:

- `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`

The Wayland acceptance runbook remains:

- `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`

Local Wayland policy-skip evidence must not be recorded as compositor acceptance.

## Invariant

If a future slice adds a richer ShowDemoWindow-style GUI, it belongs in DevTools/diagnostics first.
Only reusable low-level mechanisms that are proven by multiple app surfaces should move toward
`fret-ui-kit::imui`, and `fret-imui` remains the thin public facade.
