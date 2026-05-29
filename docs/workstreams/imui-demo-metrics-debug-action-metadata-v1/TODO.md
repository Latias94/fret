# IMUI Demo Metrics Debug Action Metadata v1 - TODO

Status: Closed
Last updated: 2026-05-30

## Tasks

- [x] DMDA-010 - Add shared route action metadata.
  - Scope: CLI JSON, DevTools GUI lines, MCP first-open lines, source gates, and first-open docs.
  - Status: Completed on 2026-05-30.
  - Validation:
    - `cargo nextest run -p fretboard-dev demos::tests::tool_apps_list_names_first_open_routes demos::tests::tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast`
    - `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates --no-fail-fast`
    - `cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast`
    - `python tools/diag_gate_imui_product_chain.py --only discovery`
    - `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`
    - `python tools/gate_imui_workstream_source.py`

- [x] DMDA-020 - Surface DevTools GUI action readiness from metadata.
  - Scope: only after DMDA-010 proves action shape. Prefer a small DevTools-side control that uses
    `requires_bundle` to disable bundle-backed actions when no bundle is selected.
  - Status: Completed on 2026-05-30 as a readiness projection in the Demo/Metrics/Debug guide.
    No new command runner was added.
  - Validation: add or update a focused DevTools unit test before adding broader GUI behavior.

- [x] DMDA-030 - Split command-palette contract work if needed.
  - Scope: start a separate follow-on if the action metadata needs a generalized command registry,
    command palette UI, or cross-route action availability contract.
  - Status: Completed on 2026-05-30 by closing this metadata/readiness lane and deferring real
    command-palette/action-execution controls to a future narrower DevTools follow-on.
  - Validation: new lane must name its owner crate and avoid `fret-imui` widening.
