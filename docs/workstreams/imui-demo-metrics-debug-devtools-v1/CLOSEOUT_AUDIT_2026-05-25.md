# IMUI Demo Metrics Debug DevTools v1 - Closeout Audit - 2026-05-25

Status: closed
Last updated: 2026-05-25

## Objective

Close the DevTools/diagnostics owner lane for the Fret equivalent of Dear ImGui
`ShowDemoWindow` / Metrics / Debug after proving the route is discoverable across CLI, DevTools GUI,
and MCP without moving diagnostics UI into `fret-imui`.

## Completion Checklist

| Requirement | Evidence |
| --- | --- |
| Shared `demo-metrics-debug` owner route exists | `apps/fretboard/src/demos.rs` |
| DevTools GUI exposes the guide and action bundle | `apps/fret-devtools/src/native.rs` |
| MCP exposes the same first-open route text | `apps/fret-devtools-mcp/src/native.rs` |
| Product-chain scripts enforce the route | `tools/diag_gate_imui_product_chain.py`, `tools/diag_gate_imui_p2_devtools_first_open.py` |
| Docking handoff remains explicit | `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`, `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md` |
| Focused gates passed | `docs/workstreams/imui-demo-metrics-debug-devtools-v1/EVIDENCE_AND_GATES.md` |

## Residual Boundaries

- Command palette integration remains deferred until DevTools has a shared command palette contract.
- Real Wayland compositor acceptance remains in the docking lane and cannot be replaced by local
  policy-skip evidence.
- Reusable IMUI diagnostics widgets should start as proof-led follow-ons; this lane does not widen
  `fret-imui`.

## Outcome

The route productization is closed. Further GUI execution controls or richer diagnostics browsers
should start as focused DevTools/diagnostics follow-ons.
