# Fret Examples Build Latency v1 - Evidence And Gates

Status: active

## Smallest Repro

```text
python tools/gate_imui_shadcn_adapter_sortable_table_source.py
```

## Gate Set

```text
python tools/gate_imui_shadcn_adapter_sortable_table_source.py
python tools/gate_imui_shadcn_adapter_control_discoverability_source.py
python tools/gate_imui_facade_teaching_source.py
python tools/gate_table_source_policy.py
python tools/gate_examples_source_tree_policy.py
python tools/gate_fret_examples_imui_split_source.py
cargo check -p fret-examples-imui --bins --jobs 1
cargo check -p fret-examples-imui --bins --profile dev-fast --jobs 1
cargo check -p fret-demo --bin imui_hello_demo --bin imui_floating_windows_demo --bin imui_response_signals_demo --bin imui_interaction_showcase_demo --bin imui_shadcn_adapter_demo --jobs 1
cargo nextest run -p fret-examples-imui --no-fail-fast
cargo check -p fret-examples --lib --jobs 1
cargo check -p fret-examples --lib --profile dev-fast --jobs 1
python tools/check_workstream_catalog.py
git diff --check
```

## Baseline Evidence

- Prior cold `cargo nextest run -p fret-examples imui_shadcn_adapter_demo_keeps_sortable_table_diag_gate --no-fail-fast` took about 5m42s locally because it compiled the monolithic examples crate for a source-marker check.
- Prior cold `cargo build -p fret-demo --bin imui_shadcn_adapter_demo --jobs 1` took about 13m20s locally because the demo bin links through the full examples library.

## Current Evidence

- Passed: `python tools/gate_imui_shadcn_adapter_sortable_table_source.py` (now covers the adapter
  facade/entrypoint markers previously guarded by
  `imui_shadcn_adapter_demo_prefers_root_fret_imui_facade_lane`).
- Passed: `python tools/gate_imui_shadcn_adapter_control_discoverability_source.py`.
- Passed: `python tools/gate_imui_facade_teaching_source.py`.
- Passed: `python tools/gate_table_source_policy.py`.
- Passed: `python tools/gate_examples_source_tree_policy.py`.
- Passed: `python tools/gate_fret_examples_imui_split_source.py`.
- Passed: `python -m py_compile tools/gate_imui_shadcn_adapter_sortable_table_source.py`.
- Passed: `python -m py_compile tools/gate_imui_shadcn_adapter_control_discoverability_source.py`.
- Passed: `python -m py_compile tools/gate_imui_facade_teaching_source.py`.
- Passed: `python -m py_compile tools/gate_table_source_policy.py`.
- Passed: `python -m py_compile tools/gate_examples_source_tree_policy.py`.
- Passed: `python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py tools/examples_source_tree_policy/app_facing.py`.
- Passed: `python -m py_compile tools/gate_fret_examples_imui_split_source.py`.
- Passed: `cargo check -p fret-examples-imui --bins --jobs 1`.
- Passed: `cargo check -p fret-examples-imui --bins --profile dev-fast --jobs 1`.
- Passed: `cargo check -p fret-demo --bin imui_hello_demo --bin imui_floating_windows_demo --bin imui_response_signals_demo --bin imui_interaction_showcase_demo --bin imui_shadcn_adapter_demo --jobs 1`.
- Passed: `cargo nextest run -p fret-examples-imui --no-fail-fast` (2 tests).
- Checked: `cargo tree -p fret-examples-imui -e normal` has no `fret-examples v...` dependency
  entry; the direct IMUI proof crate does not depend on the monolithic examples crate.
- Passed: `cargo check -p fret-examples --lib --jobs 1`.
- Passed: `cargo check -p fret-examples --lib --profile dev-fast --jobs 1`.
- Passed: `python tools/check_workstream_catalog.py`.
- Passed: `git diff --check`.
- Recorded: `docs/workstreams/fret-examples-build-latency-v1/M3_PROFILE_POLICY_DECISION_2026-04-29.md`
  with the default `dev` stability policy and explicit `dev-fast` local speed override.
- Recorded: `docs/workstreams/fret-examples-build-latency-v1/M4_IMUI_EDITOR_THEME_SOURCE_GATE_2026-04-29.md`
  after moving two IMUI editor proof theme/preset source markers into
  `tools/gate_imui_facade_teaching_source.py`.
- Recorded: `docs/workstreams/fret-examples-build-latency-v1/M5_IMUI_STATE_SOURCE_GATE_2026-04-29.md`
  after moving IMUI local-state and workspace-shell entry source markers into
  `tools/gate_imui_facade_teaching_source.py`.
- Recorded: `docs/workstreams/fret-examples-build-latency-v1/M6_WORKSPACE_SHELL_SOURCE_GATE_2026-04-30.md`
  after moving workspace shell capability-helper source markers into
  `tools/gate_examples_source_tree_policy.py`.
- Recorded: `docs/workstreams/fret-examples-build-latency-v1/M7_VIEW_ENTRY_SOURCE_GATE_2026-04-30.md`
  after moving broad view-runtime AppUi alias and builder/run entry source markers into
  `tools/gate_examples_source_tree_policy.py`.
- Recorded: `docs/workstreams/fret-examples-build-latency-v1/M8_AUTHORING_IMPORT_SOURCE_GATE_2026-04-30.md`
  after moving grouped data, query facade, advanced entry alias, and docking import owner markers
  into `tools/gate_examples_source_tree_policy.py`.
- Recorded: `docs/workstreams/fret-examples-build-latency-v1/M9_THEME_SOURCE_GATE_2026-04-30.md`
  after moving default app, advanced runtime, element-context, and renderer bridge theme-read
  markers into `tools/gate_examples_source_tree_policy.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M10_LOCAL_STATE_BRIDGE_SOURCE_GATE_2026-04-30.md`
  after moving default app local-state-first, init-time `LocalState::new_in`, AppUi render-root
  bridge, and local-state component bridge markers into `tools/gate_examples_source_tree_policy.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M11_MODEL_READ_ASSET_SOURCE_GATE_2026-04-30.md`
  after moving grouped selector-model layout, driver-owned raw model-store read, GenUI state helper,
  UI asset helper, and embedded viewport driver extension markers into
  `tools/gate_examples_source_tree_policy.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M12_ADVANCED_ROSTER_SOURCE_GATE_2026-04-30.md`
  after moving advanced/reference surface selection, classification, and examples docs roster
  markers into `tools/gate_examples_source_tree_policy.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M13_DEFAULT_APP_SURFACE_SOURCE_GATE_2026-04-30.md`
  after moving simple-todo, query/query-async, and hello-counter default app surface markers into
  `tools/gate_examples_source_tree_policy.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M14_SOURCE_GATE_STRUCTURE_SPLIT_2026-04-30.md`
  after keeping `tools/gate_examples_source_tree_policy.py` as the stable command wrapper and moving
  the implementation to `tools/examples_source_tree_policy/gate.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M15_QUERY_MARKDOWN_EDITOR_NOTES_SOURCE_GATE_2026-04-30.md`
  after moving query/query-async capability-first landing, markdown layout-query/capability-first
  landing, and editor notes reusable-panel/root markers into
  `tools/examples_source_tree_policy/gate.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M16_TODO_ASYNC_PLAYGROUND_SOURCE_GATE_2026-04-30.md`
  after moving todo default-app/root builder and async playground `AppRenderContext`/root landing
  markers into `tools/examples_source_tree_policy/gate.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M17_API_WORKBENCH_SOURCE_GATE_2026-04-30.md`
  after moving API workbench lite default-app, AppRenderContext, capability-first, and SQLite
  query/mutation ownership markers into `tools/examples_source_tree_policy/gate.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M18_APP_FACING_SOURCE_GATE_MODULE_SPLIT_2026-04-30.md`
  after moving app-facing demo source-policy matrices from
  `tools/examples_source_tree_policy/gate.py` into `tools/examples_source_tree_policy/app_facing.py`.
- Recorded: `docs/workstreams/fret-examples-build-latency-v1/M1_SOURCE_POLICY_AUDIT_2026-04-29.md`
  with the remaining source-policy test count and migration candidates.
- Current count after the API workbench source migration: 281 `include_str!` occurrences and 70
  Rust `#[test]` functions remain in `apps/fret-examples/src/lib.rs`.
- Noted: `python tools/check_workstream_state.py` is not usable as a lane-local gate yet because
  existing historical workstream state files fail the global strict validator before this lane is
  evaluated.

## Evidence Anchors

- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples-imui/src/imui_hello_demo.rs`
- `apps/fret-examples-imui/src/imui_floating_windows_demo.rs`
- `apps/fret-examples-imui/src/imui_response_signals_demo.rs`
- `apps/fret-examples-imui/src/imui_interaction_showcase_demo.rs`
- `apps/fret-examples-imui/src/bin/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples-imui/src/bin/imui_hello_demo.rs`
- `apps/fret-examples-imui/src/bin/imui_floating_windows_demo.rs`
- `apps/fret-examples-imui/src/bin/imui_response_signals_demo.rs`
- `apps/fret-examples-imui/src/bin/imui_interaction_showcase_demo.rs`
- `tools/gate_examples_source_tree_policy.py`
- `tools/examples_source_tree_policy/__init__.py`
- `tools/examples_source_tree_policy/app_facing.py`
- `tools/examples_source_tree_policy/gate.py`
- `tools/gate_fret_examples_imui_split_source.py`
- `apps/fret-demo/Cargo.toml`
- `apps/fret-demo/src/bin/imui_shadcn_adapter_demo.rs`
- `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-control-discoverability.json`
- `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json`
- `docs/workstreams/fret-examples-build-latency-v1/M1_SOURCE_POLICY_AUDIT_2026-04-29.md`
- `docs/workstreams/fret-examples-build-latency-v1/M2_DEMO_BUILD_SPLIT_DECISION_2026-04-29.md`
- `docs/workstreams/fret-examples-build-latency-v1/M3_PROFILE_POLICY_DECISION_2026-04-29.md`
- `docs/workstreams/fret-examples-build-latency-v1/M4_IMUI_EDITOR_THEME_SOURCE_GATE_2026-04-29.md`
- `docs/workstreams/fret-examples-build-latency-v1/M5_IMUI_STATE_SOURCE_GATE_2026-04-29.md`
- `docs/workstreams/fret-examples-build-latency-v1/M6_WORKSPACE_SHELL_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M7_VIEW_ENTRY_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M8_AUTHORING_IMPORT_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M9_THEME_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M10_LOCAL_STATE_BRIDGE_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M11_MODEL_READ_ASSET_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M12_ADVANCED_ROSTER_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M13_DEFAULT_APP_SURFACE_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M14_SOURCE_GATE_STRUCTURE_SPLIT_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M15_QUERY_MARKDOWN_EDITOR_NOTES_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M16_TODO_ASYNC_PLAYGROUND_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M17_API_WORKBENCH_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M18_APP_FACING_SOURCE_GATE_MODULE_SPLIT_2026-04-30.md`
- `Cargo.toml`
