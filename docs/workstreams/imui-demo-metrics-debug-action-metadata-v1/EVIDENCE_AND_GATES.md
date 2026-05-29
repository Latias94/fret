# IMUI Demo Metrics Debug Action Metadata v1 - Evidence and Gates

Status: Active
Last updated: 2026-05-30

## Repro

```bash
cargo run -p fretboard-dev -- list tool-apps --json
cargo run -p fretboard-dev -- list tool-apps
```

Expected evidence:

- The `demo-metrics-debug` route includes `action_metadata_doc`.
- `action_commands` entries include stable `id`, `category`, `primary`, and `requires_bundle`.
- DevTools and MCP first-open text include matching action metadata lines.

## Focused Gates

```bash
cargo nextest run -p fretboard-dev demos::tests::tool_apps_list_names_first_open_routes demos::tests::tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast
cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates --no-fail-fast
cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
```

## Verified 2026-05-30

```bash
cargo fmt -p fretboard-dev -p fret-devtools -p fret-devtools-mcp
cargo fmt -p fretboard-dev -p fret-devtools -p fret-devtools-mcp -- --check
cargo nextest run -p fretboard-dev demos::tests::tool_apps_list_names_first_open_routes demos::tests::tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast
cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates --no-fail-fast
cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Note: full-workspace `cargo fmt` hit Windows `os error 206` due command-line/path length, so the
verified formatting gate is package-scoped to the touched Rust packages.

## Verified 2026-05-30 - DMDA-020

```bash
cargo fmt -p fret-devtools
cargo fmt -p fret-devtools -- --check
cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes demo_metrics_debug_action_bundle_prioritizes_workbench_and_shared_gates demo_metrics_debug_lines_mark_bundle_actions_runnable_with_selected_bundle --no-fail-fast
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/gate_imui_workstream_source.py
git diff --check
```

## Evidence Anchors

- `apps/fretboard/src/demos.rs`
- `apps/fret-devtools/src/native.rs`
- `apps/fret-devtools-mcp/src/native.rs`
- `docs/diagnostics-first-open.md`
- `tools/diag_gate_imui_product_chain.py`
- `tools/diag_gate_imui_p2_devtools_first_open.py`
- `tools/gate_imui_workstream_source.py`
