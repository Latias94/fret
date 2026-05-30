# IMUI Demo Metrics Debug Action Metadata v1 - Closeout Audit - 2026-05-30

Status: closed
Last updated: 2026-05-30

## Objective

Close the narrow follow-on that made the `demo-metrics-debug` first-open route consumable by richer
DevTools GUI controls and future command-palette work without reopening the closed route owner lane
or moving diagnostics UI into `fret-imui`.

## Completion Checklist

| Requirement | Evidence |
| --- | --- |
| Route exposes an action metadata owner | `apps/fretboard/src/demos.rs`, `docs/workstreams/imui-demo-metrics-debug-action-metadata-v1/WORKSTREAM.json` |
| CLI JSON action objects expose stable metadata | `apps/fretboard/src/demos.rs` |
| DevTools GUI and MCP first-open text surface matching metadata | `apps/fret-devtools/src/native.rs`, `apps/fret-devtools-mcp/src/native.rs` |
| DevTools GUI projects bundle-backed action readiness from selected bundle state | `apps/fret-devtools/src/native.rs` |
| Product-chain and first-open gates enforce the route and source markers | `tools/diag_gate_imui_product_chain.py`, `tools/diag_gate_imui_p2_devtools_first_open.py`, `tools/gate_imui_workstream_source.py` |
| Focused verification passed | `docs/workstreams/imui-demo-metrics-debug-action-metadata-v1/EVIDENCE_AND_GATES.md` |

## Residual Boundaries

- This lane did not add a command runner, shell launcher, or shared command palette contract.
- If command-palette integration is needed, start a separate follow-on with its own owner boundary.
- `fret-imui` remains thin; diagnostics product UI stays in DevTools/diagnostics.
- Real-host Wayland compositor acceptance remains owned by `docking-multiwindow-imgui-parity`.

## Outcome

The action metadata and readiness projection are complete. The lane is closed; future work should
start as narrower DevTools command-palette or action-execution follow-ons.
