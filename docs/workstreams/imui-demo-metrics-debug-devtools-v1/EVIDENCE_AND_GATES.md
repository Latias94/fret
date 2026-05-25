# IMUI Demo Metrics Debug DevTools v1 - Evidence & Gates

Status: Closed
Last updated: 2026-05-25

## Evidence Anchors

- Owner lane:
  - `docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-demo-metrics-debug-devtools-v1/DESIGN.md`
  - `docs/workstreams/imui-demo-metrics-debug-devtools-v1/TODO.md`
  - `docs/workstreams/imui-demo-metrics-debug-devtools-v1/MILESTONES.md`
  - `docs/workstreams/imui-demo-metrics-debug-devtools-v1/EVIDENCE_AND_GATES.md`
- Discovery surfaces:
  - `apps/fretboard/src/demos.rs`
  - `apps/fret-devtools/src/native.rs`
  - `apps/fret-devtools-mcp/src/native.rs`
  - `docs/diagnostics-first-open.md`
  - `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
  - `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`
  - `tools/diag_gate_imui_product_chain.py`
  - `tools/diag_gate_imui_p2_devtools_first_open.py`

## Focused Gates

```powershell
cargo nextest run -p fretboard-dev demos::tests::tool_apps_list_names_first_open_routes demos::tests::tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast
cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates --no-fail-fast
cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

## 2026-05-25 DMD-030 Product Diagnostics Closure

- `apps/fretboard/src/demos.rs` now exposes a machine-readable `action_commands` group for the
  `demo-metrics-debug` route, with the workbench, discovery gate, metrics, debug, and docking
  validation actions promoted to first-class route data.
- `apps/fret-devtools/src/native.rs` now renders a dedicated `Demo / Metrics / Debug Routes`
  action row with a copyable command bundle, and its first-open lines explicitly defer command
  palette work until a shared contract exists.
- `apps/fret-devtools-mcp/src/native.rs` now mirrors the same action bundle in `first-open.md`.
- `tools/diag_gate_imui_product_chain.py` and `tools/diag_gate_imui_p2_devtools_first_open.py`
  now source-check the action bundle and the deferred palette decision.

Fresh gates:

- `cargo fmt --check -p fretboard-dev -p fret-devtools -p fret-devtools-mcp` passed.
- `cargo nextest run -p fretboard-dev demos::tests::tool_apps_list_names_first_open_routes demos::tests::tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast` passed with 2 tests.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates --no-fail-fast` passed with 2 tests.
- `cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast` passed with 1 test.
- `python tools/diag_gate_imui_product_chain.py --only discovery` passed.
- `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python tools/check_workstream_catalog.py` passed and validated 440 dedicated directories plus 47 standalone markdown files.
- `python -m json.tool docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json` passed.
- `git diff --check` passed; Git reported CRLF/LF working-copy warnings for `Cargo.lock` and
  `apps/fret-examples/src/lib.rs`, but no whitespace errors.

Notes:

- Discovery builds emitted existing `crates/fret-ui` warnings for `unexpected cfg:
  unstable-retained-bridge` and `current_effective_opacity`; they are outside this slice.
- No screenshot was added because this slice changed the DevTools first-open guide/action command
  surface and CLI/MCP text contracts; the action surface is covered by source tests and discovery
  gates rather than a new visual layout capture.

## 2026-05-25 DMD-010 Owner Lane And Discovery Contract

- Created `docs/workstreams/imui-demo-metrics-debug-devtools-v1/` as the owner lane for the Fret
  equivalent of Dear ImGui `ShowDemoWindow` / Metrics / Debug.
- `apps/fretboard/src/demos.rs` now exposes the `demo-metrics-debug` owner doc in human
  `list tool-apps` output and JSON `first_open_routes[].owner_doc`.
- `apps/fret-devtools/src/native.rs` and `apps/fret-devtools-mcp/src/native.rs` now surface the
  same route owner and canonical/supporting demo split.
- `docs/diagnostics-first-open.md` now documents the owner lane, canonical editor workbench route,
  supporting proof demo, and shared metrics/debug command chain.
- `tools/diag_gate_imui_product_chain.py`, `tools/diag_gate_imui_p2_devtools_first_open.py`, and
  `tools/gate_imui_workstream_source.py` now enforce the owner-lane route contract.

Fresh gates:

- `cargo fmt --check -p fretboard-dev -p fret-devtools -p fret-devtools-mcp` passed.
- `cargo nextest run -p fretboard-dev demos::tests::tool_apps_list_names_first_open_routes demos::tests::tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast` passed with 2 tests.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes --no-fail-fast` passed with 1 test.
- `cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast` passed with 1 test.
- `python tools/diag_gate_imui_product_chain.py --only discovery` passed.
- `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python tools/check_workstream_catalog.py` passed and validated 438 dedicated directories plus 47 standalone markdown files.
- `git diff --check` passed; Git reported a CRLF-to-LF working-copy warning for
  `apps/fret-examples/src/lib.rs`, but no whitespace errors.

Notes:

- `python tools/diag_gate_imui_product_chain.py --only discovery` emitted existing warnings from
  `crates/fret-ui` (`unexpected cfg: unstable-retained-bridge` and
  `current_effective_opacity` dead code). They are outside this slice.

## 2026-05-25 DMD-020 Discovery Route Handoff Metadata

- CLI human and JSON `first_open_routes` now expose `docking_owner_doc`,
  `wayland_acceptance_doc`, and `handoff_commands` for `demo-metrics-debug`.
- DevTools GUI first-open lines and MCP first-open resource text expose the same docking owner,
  Wayland acceptance runbook, and bounded handoff commands.
- `docs/diagnostics-first-open.md` now documents that local Wayland policy-skip evidence is not
  real-host compositor acceptance.
- `tools/diag_gate_imui_product_chain.py`, `tools/diag_gate_imui_p2_devtools_first_open.py`, and
  `tools/gate_imui_workstream_source.py` now enforce the route handoff metadata.

Fresh gates:

- `cargo fmt --check -p fretboard-dev -p fret-devtools -p fret-devtools-mcp` passed.
- `cargo nextest run -p fretboard-dev demos::tests::tool_apps_list_names_first_open_routes demos::tests::tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast` passed with 2 tests.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes --no-fail-fast` passed with 1 test.
- `cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast` passed with 1 test.
- `python -m py_compile tools/gate_imui_workstream_source.py tools/diag_gate_imui_p2_devtools_first_open.py tools/diag_gate_imui_product_chain.py` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` passed after rebuilding the stale `fretboard-dev` binary.
- `python tools/diag_gate_imui_product_chain.py --only discovery` passed.
- `python -m json.tool docs/workstreams/imui-demo-metrics-debug-devtools-v1/WORKSTREAM.json` passed.
- `python -m json.tool docs/workstreams/imui-editor-workbench-golden-path-v1/WORKSTREAM.json` passed.
- `python tools/check_workstream_catalog.py` passed and validated 438 dedicated directories plus
  47 standalone markdown files.
- `git diff --check` passed; Git reported a CRLF-to-LF working-copy warning for
  `apps/fret-examples/src/lib.rs`, but no whitespace errors.

Notes:

- Discovery builds emitted existing `crates/fret-ui` warnings for `unexpected cfg:
  unstable-retained-bridge` and `current_effective_opacity`; they are outside this slice.
