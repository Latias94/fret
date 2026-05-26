# IMUI Demo Metrics Debug DevTools v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## DMD-010 - Owner Lane And Discovery Contract

Goal: make the Demo/Metrics/Debug route owner explicit and machine-checkable.

Acceptance:

- `demo-metrics-debug` has an owner doc in `fretboard-dev list tool-apps --json`.
- DevTools GUI first-open lines expose the same owner doc.
- MCP first-open resource text exposes the same owner doc.
- Source gates reject moving this product surface into `fret-imui`.

Result 2026-05-25:

- Complete. The CLI, DevTools GUI, and MCP first-open route now expose
  `docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json` as the route owner.
- Complete. The route lists `imui_editor_workbench_demo` first and keeps `imui_editor_proof_demo`
  as a supporting proof surface.
- Complete. Source gates now require the owner lane markers and keep the productized diagnostics
  route outside `fret-imui`.

## DMD-020 - Persistent DevTools Product Surface

Goal: turn the current guide text into a more actionable DevTools product surface without changing
IMUI runtime contracts.

Acceptance:

- The canonical editor workbench is the first action.
- Metrics/debug follow-up commands are reachable from current artifact state.
- The route remains based on shared diagnostics artifacts.
- Docking hand-feel remains owned by `docking-multiwindow-imgui-parity`.
- Local Wayland policy-skip evidence is not recorded as real compositor acceptance.

Result 2026-05-25:

- Complete. CLI human output, CLI JSON, DevTools GUI first-open lines, and MCP first-open resource
  text expose the same docking owner doc, Wayland acceptance runbook, and bounded handoff commands.
- Complete. The route keeps `imui_editor_workbench_demo` first and keeps metrics/debug commands
  backed by shared diagnostics artifacts.
- Complete. The source gates now reject losing the docking handoff owner, bounded campaign
  validate command, or local policy-skip boundary.

## DMD-030 - Product Diagnostics Closure

Goal: make the diagnostics route feel like a maintained product surface instead of a loose list of
links.

Acceptance:

- DevTools GUI exposes a dedicated guide panel for the action bundle.
- MCP mirrors the same action bundle in first-open text.
- CLI JSON exposes the same actions as machine-readable route data.
- Command palette work is deferred until a shared palette contract exists.

Result 2026-05-25:

- Complete. `fretboard-dev list tool-apps` now carries `action_commands` for the route.
- Complete. DevTools GUI exposes a copyable `Demo/Metrics/Debug` action bundle inside the
  dedicated route panel.
- Complete. MCP first-open text mirrors the same action bundle.
- Complete. The source gates now require the action surface markers and the deferred palette
  decision.
