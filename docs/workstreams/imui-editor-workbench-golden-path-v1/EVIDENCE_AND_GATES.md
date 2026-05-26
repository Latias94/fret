# IMUI Editor Workbench Golden Path v1 - Evidence & Gates

Status: Closed
Last updated: 2026-05-25

## Evidence Anchors

- Workstream:
  - `docs/workstreams/imui-editor-workbench-golden-path-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-editor-workbench-golden-path-v1/DESIGN.md`
  - `docs/workstreams/imui-editor-workbench-golden-path-v1/TODO.md`
  - `docs/workstreams/imui-editor-workbench-golden-path-v1/MILESTONES.md`
  - `docs/workstreams/imui-editor-workbench-golden-path-v1/EVIDENCE_AND_GATES.md`
- Canonical route:
  - `apps/fret-examples/src/imui_editor_workbench_demo.rs`
  - `apps/fret-examples/src/lib.rs`
  - `apps/fret-demo/src/bin/imui_editor_workbench_demo.rs`
  - `apps/fret-examples/tests/imui_editor_workbench_golden_path_surface.rs`
- Supporting current proof surfaces:
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
  - `apps/fret-examples/src/docking_arbitration_demo.rs`
- Current gap and product-closure references:
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_EXECUTION_PRIORITY_REVIEW_2026-05-06.md`
  - `docs/workstreams/imui-editor-grade-product-closure-v1/GOAL_COMPLETION_AUDIT_2026-05-15.md`
- Proof-led child lanes:
  - `docs/workstreams/imui-list-box-container-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-plot-adapter-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-style-theme-editor-proof-v1/WORKSTREAM.json`

## Repro

```powershell
cargo run -p fret-demo --bin imui_editor_workbench_demo
```

## Focused Gates

```powershell
cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast
cargo check -p fret-demo --bin imui_editor_workbench_demo
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

## 2026-05-25 Slice Results

- `cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast` passed and proved the canonical route surface test is wired to `imui_editor_workbench_demo`.
- `cargo check -p fret-demo --bin imui_editor_workbench_demo` passed and proved the new direct demo binary compiles.
- `python tools/gate_imui_workstream_source.py` passed after updating the UI Kit List torture doc marker to the current expected contract string.
- `python tools/check_workstream_catalog.py` passed after adding this dedicated workstream directory to `docs/workstreams/README.md` and bumping the dedicated-directory count.
- `git diff --check` passed.
- `cargo fmt --check -p fret-examples -p fret-demo` passed after targeted formatting.
- `cargo fmt --check -p fret-examples -p fret-demo -p fret-ui-gallery` reported a pre-existing formatting delta in `apps/fret-ui-gallery/tests/menubar_docs_surface.rs`; that file was not part of this slice.
- `rustfmt --check --edition 2024 apps/fret-ui-gallery/src/ui/previews/pages/harness/ui_kit_list_torture.rs` passed after the gallery marker update.
## 2026-05-25 EWG-020 Product Route Promotion

- `apps/fret-cookbook/README.md`, `apps/fret-cookbook/EXAMPLES.md`, and `docs/examples/README.md` now promote `cargo run -p fret-demo --bin imui_editor_workbench_demo` as the editor-grade IMUI first-open workbench route.
- `docs/diagnostics-first-open.md` now describes `first_open_routes` as exposing the canonical editor workbench plus supporting proof demos.
- `apps/fretboard/src/demos.rs` now exposes `demo editor workbench` in `list tool-apps` human and JSON output, while keeping `demo editor proof supporting` discoverable.
- `apps/fret-devtools/src/native.rs` and `apps/fret-devtools-mcp/src/native.rs` now surface the same Demo/Metrics/Debug route split.
- `tools/diag_gate_imui_product_chain.py`, `tools/diag_gate_imui_p2_devtools_first_open.py`, and `tools/gate_imui_workstream_source.py` now enforce the canonical/supporting route distinction.

Fresh gates:

- `cargo fmt --check -p fret-examples -p fret-demo -p fretboard -p fret-devtools -p fret-devtools-mcp` passed.
- `cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast` passed with 2 tests.
- `cargo check -p fret-demo --bin imui_editor_workbench_demo` passed.
- `python tools/diag_gate_imui_product_chain.py --only discovery` passed.
- `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fretboard-dev demos::tests::tool_apps_list_names_first_open_routes demos::tests::tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast` passed with 2 tests.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes --no-fail-fast` passed with 1 test.
- `cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast` passed with 1 test.
- `python tools/check_workstream_catalog.py` passed.
- `git diff --check` passed.
- `cargo run -p fretboard-dev -- list native-demos --all` listed `imui_editor_workbench_demo`, `imui_editor_proof_demo`, `workspace_shell_demo`, and `docking_arbitration_demo`.

Notes:

- `cargo nextest run -p fretboard tool_apps --no-fail-fast` and `cargo test -p fretboard tool_apps -- --nocapture` ran against the `crates/fretboard` library package and matched 0 tests; the actual CLI tests live in package `fretboard-dev` and passed above.
- Commands emitted existing warnings from `crates/fret-ui` (`unexpected cfg: unstable-retained-bridge`, plus dead-code warnings in some builds). They are outside this slice.

## 2026-05-25 EWG-030 Workbench Composition Convergence

- `apps/fret-examples/src/imui_editor_workbench_demo.rs` now owns the app shell for the canonical workbench route instead of forwarding directly to `workspace_shell_demo`.
- `apps/fret-examples/src/editor_notes_demo.rs` now exposes `EditorNotesDemoView` as `pub(crate)` so the canonical route can mount the reusable editor-notes workflow directly.
- `apps/fret-examples/tests/imui_editor_workbench_golden_path_surface.rs` now verifies that the canonical route mounts `crate::editor_notes_demo::EditorNotesDemoView`, installs the editor-notes theme, and no longer calls `workspace_shell_demo::run()`.

Fresh gates:

- `cargo fmt --check -p fret-examples -p fret-demo` passed.
- `cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast` passed with 2 tests after one retry from a transient cargo build lock and timeout on the first attempt.
- `cargo check -p fret-demo --bin imui_editor_workbench_demo` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

## 2026-05-25 EWG-040 Demo/Metrics/Debug Productization Handoff

- Created `docs/workstreams/imui-demo-metrics-debug-devtools-v1/` as the dedicated DevTools and
  diagnostics owner lane for the Fret equivalent of Dear ImGui `ShowDemoWindow` / Metrics / Debug.
- `apps/fretboard/src/demos.rs`, `apps/fret-devtools/src/native.rs`, and
  `apps/fret-devtools-mcp/src/native.rs` now expose the owner lane for `demo-metrics-debug`.
- The discovery route keeps `cargo run -p fret-demo --bin imui_editor_workbench_demo` as
  `demo editor workbench` and keeps `imui_editor_proof_demo` as a supporting proof route.
- `docs/diagnostics-first-open.md` now documents the owner route and keeps diagnostics
  productization in DevTools/diagnostics surfaces rather than `fret-imui`.

Fresh gates:

- `cargo fmt --check -p fretboard-dev -p fret-devtools -p fret-devtools-mcp` passed.
- `cargo nextest run -p fretboard-dev demos::tests::tool_apps_list_names_first_open_routes demos::tests::tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast` passed with 2 tests.
- `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes --no-fail-fast` passed with 1 test.
- `cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast` passed with 1 test.
- `python tools/diag_gate_imui_product_chain.py --only discovery` passed.
- `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python tools/check_workstream_catalog.py` passed.
- `git diff --check` passed with a Git CRLF-to-LF working-copy warning for
  `apps/fret-examples/src/lib.rs`.

Notes:

- Discovery builds emitted existing `crates/fret-ui` warnings for `unstable-retained-bridge` and
  `current_effective_opacity`; they are outside this slice.

## 2026-05-25 EWG-050 Docking / Runner Handoff

- `demo-metrics-debug` now points to the docking hand-feel owner lane instead of reopening docking
  work inside the canonical workbench lane.
- CLI human output, CLI JSON, DevTools GUI first-open lines, and MCP first-open resource text now
  expose the docking owner doc:
  `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`.
- The route also exposes the bounded `imui-p3-multiwindow-parity` campaign validate command and the
  real-host Wayland acceptance runbook:
  `docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`.
- `docs/diagnostics-first-open.md` now records that local Wayland policy-skip evidence is not
  compositor acceptance.

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

## 2026-05-25 EWG-060 Proof-Led Helper/API Candidates

Candidate verdicts:

- ListBox container: accepted as a narrow `BeginListBox`-style container lane in
  `docs/workstreams/imui-list-box-container-proof-v1/`. It stamps ListBox semantics and hosts rows
  without owning selection, filtering, command packages, or collection policy.
- Collection-helper widening: deferred. The collection-helper readiness closeout already found no
  need for a reusable helper across the two proof surfaces; keep collection behavior app-owned until
  product routes show repeated friction.
- Plot adapter: accepted as a narrow owner lane because `fret-plot` already has mature declarative
  panel proof surfaces and the gap catalog named an IMUI adapter as a candidate. The owner lane is
  `docs/workstreams/imui-plot-adapter-proof-v1/`.
- Style/theme editor: accepted as a narrow editor-owned preset picker, not a broad Dear ImGui
  mutable style-stack clone. The owner lane is
  `docs/workstreams/imui-style-theme-editor-proof-v1/`, and the canonical editor-notes inspector now
  exposes `editor-notes-demo.inspector.theme-preset`.
- Porting sugar: deferred. The gap review keeps it candidate-only until two product proof surfaces
  need the same shorthand.

Evidence:

- `docs/workstreams/imui-collection-helper-readiness-v1/CLOSEOUT_AUDIT_2026-04-24.md`
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_EXECUTION_PRIORITY_REVIEW_2026-05-06.md`
- `docs/workstreams/imui-list-box-container-proof-v1/WORKSTREAM.json`
- `docs/workstreams/imui-plot-adapter-proof-v1/WORKSTREAM.json`
- `docs/workstreams/imui-style-theme-editor-proof-v1/WORKSTREAM.json`
- `apps/fret-examples/src/editor_notes_demo.rs`
- `apps/fret-examples/src/editor_notes_device_shell_demo.rs`
- `apps/fret-examples/tests/imui_editor_workbench_golden_path_surface.rs`

Fresh gates:

- Passed in `docs/workstreams/imui-list-box-container-proof-v1/EVIDENCE_AND_GATES.md`:
  - `cargo fmt --check -p fret-ui-kit -p fret-imui`
  - `cargo check -p fret-ui-kit --features imui`
  - `cargo nextest run -p fret-imui list_box_container_stamps_semantics_scroll_and_hosts_selectables --no-fail-fast`
  - `python tools/gate_imui_workstream_source.py`
  - `python tools/check_workstream_catalog.py`
  - `python -m json.tool docs/workstreams/imui-list-box-container-proof-v1/WORKSTREAM.json`
  - `git diff --check`
- Passed in `docs/workstreams/imui-plot-adapter-proof-v1/EVIDENCE_AND_GATES.md`:
  - `cargo fmt --check -p fret-plot`
  - `cargo check -p fret-plot`
  - `cargo check -p fret-plot --features imui`
  - `cargo nextest run -p fret-plot imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast`
  - `python tools/gate_imui_workstream_source.py`
  - `python tools/check_workstream_catalog.py`
  - `python -m json.tool docs/workstreams/imui-plot-adapter-proof-v1/WORKSTREAM.json`
  - `git diff --check`
- Passed in `docs/workstreams/imui-style-theme-editor-proof-v1/EVIDENCE_AND_GATES.md`:
  - `cargo fmt --check -p fret-ui-editor`
  - `cargo check -p fret-ui-editor --features imui`
  - `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast`
  - `cargo fmt --check -p fret-examples -p fret-demo`
  - `cargo check -p fret-demo --bin imui_editor_workbench_demo`
  - `cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast`
  - `cargo nextest run -p fret-examples --test editor_notes_device_shell_surface --no-fail-fast`
  - `cargo nextest run -p fret-examples parse_editor_theme_preset_key --no-fail-fast`
  - `python tools/gate_imui_workstream_source.py`
  - `python -m py_compile tools/gate_imui_workstream_source.py`
  - `python tools/check_workstream_catalog.py`
  - `python -m json.tool docs/workstreams/imui-style-theme-editor-proof-v1/WORKSTREAM.json`
  - `python -m json.tool docs/workstreams/imui-editor-workbench-golden-path-v1/WORKSTREAM.json`
  - `git diff --check`

Notes:

- The first parallel attempt to run the two `fret-examples` nextest commands timed out while
  waiting on Cargo locks; both commands passed when rerun serially.
- `git diff --check` has no whitespace errors. Git still reports existing line-ending warnings for
  `Cargo.lock` and `apps/fret-examples/src/lib.rs`.
- Cargo commands emit existing warnings from `crates/fret-ui` (`unstable-retained-bridge`) plus
  unrelated dead-code warnings in `fret-chart` / `fret-plot`.

## 2026-05-25 Closeout Verification

- Added `CLOSEOUT_AUDIT_2026-05-25.md` and marked the following completed owner lanes `closed` with
  `start_follow_on` continue policy:
  - `imui-editor-workbench-golden-path-v1`
  - `imui-demo-metrics-debug-devtools-v1`
  - `imui-list-box-container-proof-v1`
  - `imui-plot-adapter-proof-v1`
  - `imui-style-theme-editor-proof-v1`
  - `imui-table-header-owner-split-v1`
  - `imui-table-body-owner-split-v1`
- The closeout does not close
  `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`; real Wayland compositor
  acceptance remains open there as `DW-P1-linux-003`.

Fresh gates:

- PASS: `python -m json.tool` for all seven closed `WORKSTREAM.json` files.
- PASS: `python tools/check_workstream_catalog.py`
  - Validated 443 dedicated directories and 47 standalone markdown files.
- PASS: `python tools/gate_imui_workstream_source.py`
- PASS: status scan found no `Status: Active`, `"status": "active"`,
  `"default_action": "continue"`, or unchecked `- [ ]` entries in the seven closed lanes.
- PASS: `cargo fmt --check -p fret-examples -p fret-demo -p fret-ui-editor -p fret-ui-kit -p fret-imui -p fret-plot`
- PASS: `cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast`
  - 2 passed.
- PASS: `cargo check -p fret-demo --bin imui_editor_workbench_demo`
- PASS_WITH_WARNINGS: `git diff --check`
  - No whitespace errors.
  - Existing line-ending warnings remain for `Cargo.lock` and `apps/fret-examples/src/lib.rs`.

## 2026-05-25 EWG-070 Kit Owner Split Follow-Ons

Owner split chosen:

- `ecosystem/fret-ui-kit/src/imui/table_controls.rs` was the next high-payoff private split because
  editor-grade tables already carry closed proof lanes for stable column identity, sortable header
  responses, context-menu requests, and resize handles.
- The split keeps public `ImUiTable`, `ImUiTableRow`, `TableColumn`, `TableResponse`, and
  `TableHeaderResponse` names stable.
- `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs` now owns visible-label parsing,
  sortable/plain header cells, sort indicator visuals, active trigger response assembly, and resize
  handle behavior.
- `table_controls.rs` remains the table authoring/body assembly owner and delegates header behavior
  to the private owner module.

Evidence:

- `docs/workstreams/imui-table-header-owner-split-v1/WORKSTREAM.json`
- `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
- `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs`
- `tools/gate_imui_workstream_source.py`

Fresh gates:

- Passed in `docs/workstreams/imui-table-header-owner-split-v1/EVIDENCE_AND_GATES.md`:
  - `cargo fmt --check -p fret-ui-kit`
  - `cargo check -p fret-ui-kit --features imui`
  - `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_sortable_header_api_compiles table_resizable_column_api_compiles --no-fail-fast`
  - `cargo nextest run -p fret-imui table_sortable_header_reports_app_owned_trigger_without_sorting_rows table_resizable_header_reports_drag_response table_plain_header_left_click_does_not_activate_or_click --no-fail-fast`
  - `python tools/gate_imui_workstream_source.py`
  - `python tools/check_workstream_catalog.py`
  - `python -m json.tool docs/workstreams/imui-table-header-owner-split-v1/WORKSTREAM.json`
  - `git diff --check`
