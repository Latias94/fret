# IMUI Demo Metrics Debug DevTools v1 - Handoff

Status: Closed
Last updated: 2026-05-25

Current slice: closed on 2026-05-25.

This lane owns the DevTools/diagnostics productization surface for the Fret equivalent of Dear
ImGui `ShowDemoWindow` / Metrics / Debug. The route is `demo-metrics-debug`.

DMD-010 status: complete. The owner lane exists and CLI, DevTools GUI, and MCP discovery expose the
same owner doc:

`docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json`

DMD-020 status: complete. The route keeps the canonical editor workbench first, keeps the proof
demo as a supporting surface, and now exposes docking handoff metadata consistently across CLI,
DevTools GUI, and MCP:

- docking owner: `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- Wayland acceptance runbook:
  `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`
- bounded campaign: `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`
- local policy-skip: `python tools/diag_gate_docking_wayland_policy_skip.py`

Productized diagnostics still live in DevTools/diagnostics surfaces, not `fret-imui`. Local
Wayland policy-skip evidence must stay separate from real-host Wayland compositor acceptance.

DMD-030 status: complete. The route now has a product action surface:

- CLI JSON exposes route-level `action_commands`.
- DevTools GUI keeps the dedicated `Demo / Metrics / Debug Routes` guide panel and adds a copyable
  action command bundle.
- MCP `fret-diag://first-open.md` mirrors the same action list.
- Command palette work is explicitly deferred until DevTools has a shared command palette contract.

Closeout:

1. Keep `cargo run -p fret-demo --bin imui_editor_workbench_demo` as the first action.
2. Keep reusable IMUI helper pressure in proof-led follow-on lanes instead of widening this lane.
3. Start a new DevTools/diagnostics follow-on for richer GUI execution controls or command palette
   integration.
