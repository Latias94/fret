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
python tools/gate_imui_workstream_source.py
python tools/gate_fret_launch_runner_scheduling_source.py
python tools/gate_table_source_policy.py
python tools/gate_examples_source_tree_policy.py
python tools/gate_fret_examples_imui_split_source.py
cargo check -p fret-examples-imui --bins --jobs 1
cargo check -p fret-examples-imui --bins --profile dev-fast --jobs 1
cargo check -p fret-demo --bin imui_hello_demo --bin imui_floating_windows_demo --bin imui_response_signals_demo --bin imui_interaction_showcase_demo --bin imui_shadcn_adapter_demo --jobs 1
cargo nextest run -p fret-examples-imui --no-fail-fast
cargo check -p fret-examples --lib --jobs 1
cargo check -p fret-examples --lib --profile dev-fast --jobs 1
cargo nextest run -p fret-examples --lib parse_editor_theme_preset_key --no-fail-fast
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
- Passed: `python tools/gate_imui_workstream_source.py`.
- Passed: `python tools/gate_fret_launch_runner_scheduling_source.py`.
- Passed: `python tools/gate_table_source_policy.py`.
- Passed: `python tools/gate_examples_source_tree_policy.py`.
- Passed: `python tools/gate_fret_examples_imui_split_source.py`.
- Passed: `python -m py_compile tools/gate_imui_shadcn_adapter_sortable_table_source.py`.
- Passed: `python -m py_compile tools/gate_imui_shadcn_adapter_control_discoverability_source.py`.
- Passed: `python -m py_compile tools/gate_imui_facade_teaching_source.py`.
- Passed: `python -m py_compile tools/gate_imui_workstream_source.py`.
- Passed: `python -m py_compile tools/gate_fret_launch_runner_scheduling_source.py`.
- Passed: `python -m py_compile tools/gate_table_source_policy.py`.
- Passed: `python -m py_compile tools/gate_examples_source_tree_policy.py`.
- Passed: `python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py tools/examples_source_tree_policy/app_facing.py`.
- Passed: `python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py tools/examples_source_tree_policy/app_facing.py tools/examples_source_tree_policy/interop.py`.
- Passed: `python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py tools/examples_source_tree_policy/app_facing.py tools/examples_source_tree_policy/interop.py tools/examples_source_tree_policy/manual.py`.
- Passed: `python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/__init__.py tools/examples_source_tree_policy/gate.py tools/examples_source_tree_policy/app_facing.py tools/examples_source_tree_policy/interop.py tools/examples_source_tree_policy/manual.py tools/examples_source_tree_policy/owner_split.py`.
- Passed: `python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/advanced_helpers.py tools/examples_source_tree_policy/gate.py tools/check_workstream_catalog.py`.
- Passed: `python -m py_compile tools/gate_examples_source_tree_policy.py tools/examples_source_tree_policy/grouped_state.py tools/examples_source_tree_policy/gate.py tools/check_workstream_catalog.py`.
- Passed: `python -m py_compile tools/gate_fret_examples_imui_split_source.py`.
- Passed: `cargo check -p fret-examples-imui --bins --jobs 1`.
- Passed: `cargo check -p fret-examples-imui --bins --profile dev-fast --jobs 1`.
- Passed: `cargo check -p fret-demo --bin imui_hello_demo --bin imui_floating_windows_demo --bin imui_response_signals_demo --bin imui_interaction_showcase_demo --bin imui_shadcn_adapter_demo --jobs 1`.
- Passed: `cargo nextest run -p fret-examples-imui --no-fail-fast` (2 tests).
- Checked: `cargo tree -p fret-examples-imui -e normal` has no `fret-examples v...` dependency
  entry; the direct IMUI proof crate does not depend on the monolithic examples crate.
- Passed: `cargo check -p fret-examples --lib --jobs 1`.
- Passed: `cargo check -p fret-examples --lib --profile dev-fast --jobs 1`.
- Passed: `cargo nextest run -p fret-examples --lib parse_editor_theme_preset_key --no-fail-fast`
  (2 parser behavior tests).
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
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M19_LOW_LEVEL_INTEROP_SOURCE_GATE_2026-04-30.md`
  after moving low-level interop direct-leaf root markers into
  `tools/examples_source_tree_policy/interop.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M20_MANUAL_UI_TREE_SOURCE_GATE_2026-04-30.md`
  after moving manual `UiTree<App>` root-wrapper markers into
  `tools/examples_source_tree_policy/manual.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M21_COMPONENTS_GALLERY_OWNER_SPLIT_SOURCE_GATE_2026-04-30.md`
  after moving components gallery owner-split source/document markers into
  `tools/examples_source_tree_policy/owner_split.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M22_SELECTED_RAW_OWNER_SOURCE_GATE_2026-04-30.md`
  after moving selected raw-owner escape-hatch source markers into
  `tools/examples_source_tree_policy/owner_split.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M23_IMUI_EDITOR_PROOF_SOURCE_GATE_2026-04-30.md`
  after moving IMUI editor proof non-raw helper, official adapter, and app-owner source markers into
  `tools/gate_imui_facade_teaching_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M24_IMUI_INTERACTION_SHOWCASE_SOURCE_GATE_2026-04-30.md`
  after moving IMUI interaction showcase layout and grouped state/action source markers into
  `tools/gate_imui_facade_teaching_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M25_IMUI_RESPONSE_SIGNALS_SOURCE_GATE_2026-04-30.md`
  after moving IMUI response signals lifecycle, canonical trigger, and grouped state/action source
  markers into `tools/gate_imui_facade_teaching_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M26_IMUI_P0_WORKSTREAM_SOURCE_GATE_2026-04-30.md`
  after moving IMUI response/key-owner workstream document freeze markers into
  `tools/gate_imui_workstream_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M27_IMUI_COLLECTION_PANE_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection/pane proof workstream document and proof-surface markers into
  `tools/gate_imui_workstream_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M28_IMUI_FACADE_MODULARIZATION_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI facade internal modularization workstream document/index markers into
  `tools/gate_imui_workstream_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M29_IMUI_COLLECTION_BOX_SELECT_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection box-select workstream document/index markers into
  `tools/gate_imui_workstream_source.py` while keeping the real `proof_collection_*` unit tests in
  Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M30_IMUI_COLLECTION_KEYBOARD_OWNER_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection keyboard-owner workstream document/index markers into
  `tools/gate_imui_workstream_source.py` while keeping the real `proof_collection_keyboard_*` unit
  tests in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M31_IMUI_COLLECTION_DELETE_ACTION_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection delete-action workstream document/index markers into
  `tools/gate_imui_workstream_source.py` while keeping the real `proof_collection_delete_*` unit
  tests in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M32_IMUI_COLLECTION_CONTEXT_MENU_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection context-menu workstream document/index markers into
  `tools/gate_imui_workstream_source.py` while keeping the real `proof_collection_context_menu_*`
  unit tests in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M33_IMUI_COLLECTION_ZOOM_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection zoom workstream document/index markers into
  `tools/gate_imui_workstream_source.py` while keeping the real `proof_collection_layout_metrics_*`
  and `proof_collection_zoom_request_*` unit tests in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M34_IMUI_COLLECTION_SELECT_ALL_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection select-all workstream document/index markers into
  `tools/gate_imui_workstream_source.py` while keeping the real `proof_collection_select_all_*`
  unit tests in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M35_IMUI_COLLECTION_RENAME_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection rename workstream document/index markers into
  `tools/gate_imui_workstream_source.py` while keeping the real `proof_collection_*rename*` unit
  tests in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M36_IMUI_COLLECTION_INLINE_RENAME_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection inline-rename workstream document/index markers into
  `tools/gate_imui_workstream_source.py` while keeping the real rename/inline-rename unit tests in
  Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M37_IMUI_COLLECTION_MODULARIZATION_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection modularization workstream document/source-boundary markers into
  `tools/gate_imui_workstream_source.py` while keeping real collection behavior unit tests in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M38_IMUI_COLLECTION_COMMAND_PACKAGE_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection command-package workstream document/index markers into
  `tools/gate_imui_workstream_source.py` while keeping real duplicate/rename behavior unit tests in
  Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M39_IMUI_COLLECTION_SECOND_PROOF_SURFACE_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection second proof-surface workstream document/source-shape markers into
  `tools/gate_imui_workstream_source.py` while keeping real shell-mounted surface tests in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M40_IMUI_COLLECTION_HELPER_READINESS_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI collection helper-readiness workstream document/no-helper-widening markers into
  `tools/gate_imui_workstream_source.py` while keeping real proof-surface tests in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M41_IMUI_EDITOR_NOTES_INSPECTOR_COMMAND_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI editor-notes inspector command workstream document/source-shape markers into
  `tools/gate_imui_workstream_source.py` while keeping the real editor rail surface test in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M42_IMUI_EDITOR_NOTES_DIRTY_STATUS_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI editor-notes dirty status workstream document/source-shape markers into
  `tools/gate_imui_workstream_source.py` while keeping the real editor rail/device shell surface
  tests in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M43_IMUI_NEXT_GAP_AUDIT_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI next-gap audit decision markers into `tools/gate_imui_workstream_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M44_IMUI_EDITOR_NOTES_DRAFT_ACTIONS_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI editor-notes draft actions workstream document/source-shape markers into
  `tools/gate_imui_workstream_source.py` while keeping real editor rail/device shell surface tests
  in Rust.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M45_IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI TextField draft-buffer contract audit document/source-shape markers into
  `tools/gate_imui_workstream_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M46_IMUI_TEXTFIELD_DRAFT_CONTROLLER_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI TextField draft-controller API proof document/source-shape markers into
  `tools/gate_imui_workstream_source.py` while keeping real API smoke, editor surface, and launched
  diagnostics gates outside the source freeze.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M47_IMUI_CHILD_REGION_DEPTH_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI child-region depth workstream document/index markers into
  `tools/gate_imui_workstream_source.py` while keeping real `fret-ui-kit`, `fret-imui`, and
  pane-proof behavior gates outside the source freeze.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M48_IMUI_MENU_TAB_TRIGGER_RESPONSE_SURFACE_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI menu/tab trigger response-surface workstream document markers into
  `tools/gate_imui_workstream_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M49_IMUI_MENU_TAB_TRIGGER_RESPONSE_CANONICALIZATION_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI menu/tab trigger response canonicalization workstream document markers into
  `tools/gate_imui_workstream_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M50_IMUI_WORKBENCH_SHELL_CLOSURE_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI workbench shell closure source-policy markers into
  `tools/gate_imui_workstream_source.py` while keeping the real shell surface tests and launched
  diagnostics suite as behavior floors.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M51_IMUI_P2_DIAGNOSTICS_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
  after moving IMUI P2 diagnostics/tooling source-policy markers into
  `tools/gate_imui_workstream_source.py` while keeping real `fret-diag`, launched diagnostics,
  DevTools, and campaign doctor gates as behavior floors.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M52_IMUI_P3_MULTIWINDOW_PACKAGE_SOURCE_GATE_2026-05-01.md`
  after moving IMUI P3 runner-gap and bounded multi-window campaign package source-policy markers
  into `tools/gate_imui_workstream_source.py` while keeping real campaign validate/run gates as
  behavior floors.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M53_DOCKING_P3_SOURCE_POLICY_GATE_2026-05-01.md`
  after moving the docking parity source-policy subset into `tools/gate_imui_workstream_source.py`
  while keeping owner-crate, campaign, and host-admitted gates as behavior floors.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M54_DOCKING_MIXED_DPI_SUPPORT_SOURCE_GATE_2026-05-01.md`
  after moving the docking mixed-DPI support note source-policy checks into
  `tools/gate_imui_workstream_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M55_DIAGNOSTICS_ENVIRONMENT_SOURCE_GATE_2026-05-01.md`
  after moving diagnostics environment source-policy checks into
  `tools/gate_imui_workstream_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M56_EMBEDDED_VIEWPORT_SOURCE_GATE_2026-05-01.md`
  after moving embedded viewport source-policy checks into
  `tools/examples_source_tree_policy/app_facing.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M57_HELLO_WORLD_COMPARE_SOURCE_GATE_2026-05-01.md`
  after moving the hello-world compare app-facing helper source-policy check into
  `tools/examples_source_tree_policy/app_facing.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M58_APP_UI_RENDER_ACCESSOR_SOURCE_GATE_2026-05-01.md`
  after moving app-facing render-root accessor source-policy checks into
  `tools/examples_source_tree_policy/app_facing.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M59_VIEW_RUNTIME_GROUPED_STATE_SOURCE_GATE_2026-05-01.md`
  after moving view-runtime grouped state/action source-policy checks into
  `tools/examples_source_tree_policy/app_facing.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M60_FIRST_FRAME_BOOTSTRAP_SOURCE_GATE_2026-05-01.md`
  after moving first-frame bootstrap runner scheduling source-policy checks into
  `tools/gate_fret_launch_runner_scheduling_source.py`.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M61_ADVANCED_HELPER_CONTEXT_SOURCE_GATE_2026-05-01.md`
  after moving advanced helper/context source-policy checks into
  `tools/examples_source_tree_policy/advanced_helpers.py` and deleting orphaned `include_str!`
  constants.
- Recorded:
  `docs/workstreams/fret-examples-build-latency-v1/M62_GROUPED_STATE_SOURCE_GATE_CLOSURE_2026-05-01.md`
  after moving the remaining grouped state/model-read source-policy checks into
  `tools/examples_source_tree_policy/grouped_state.py`.
- Recorded: `docs/workstreams/fret-examples-build-latency-v1/M1_SOURCE_POLICY_AUDIT_2026-04-29.md`
  with the remaining source-policy test count and migration candidates.
- Current count after the grouped state source migration: 0 `include_str!` occurrences and 2 Rust
  `#[test]` functions remain in `apps/fret-examples/src/lib.rs`; both remaining tests are parser
  behavior tests.
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
- `tools/examples_source_tree_policy/advanced_helpers.py`
- `tools/examples_source_tree_policy/app_facing.py`
- `tools/examples_source_tree_policy/gate.py`
- `tools/examples_source_tree_policy/grouped_state.py`
- `tools/examples_source_tree_policy/interop.py`
- `tools/examples_source_tree_policy/manual.py`
- `tools/examples_source_tree_policy/owner_split.py`
- `tools/gate_fret_launch_runner_scheduling_source.py`
- `tools/gate_fret_examples_imui_split_source.py`
- `tools/gate_imui_workstream_source.py`
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
- `docs/workstreams/fret-examples-build-latency-v1/M19_LOW_LEVEL_INTEROP_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M20_MANUAL_UI_TREE_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M21_COMPONENTS_GALLERY_OWNER_SPLIT_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M22_SELECTED_RAW_OWNER_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M23_IMUI_EDITOR_PROOF_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M24_IMUI_INTERACTION_SHOWCASE_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M25_IMUI_RESPONSE_SIGNALS_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M26_IMUI_P0_WORKSTREAM_SOURCE_GATE_2026-04-30.md`
- `docs/workstreams/fret-examples-build-latency-v1/M27_IMUI_COLLECTION_PANE_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M28_IMUI_FACADE_MODULARIZATION_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M29_IMUI_COLLECTION_BOX_SELECT_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M30_IMUI_COLLECTION_KEYBOARD_OWNER_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M31_IMUI_COLLECTION_DELETE_ACTION_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M32_IMUI_COLLECTION_CONTEXT_MENU_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M33_IMUI_COLLECTION_ZOOM_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M34_IMUI_COLLECTION_SELECT_ALL_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M35_IMUI_COLLECTION_RENAME_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M36_IMUI_COLLECTION_INLINE_RENAME_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M37_IMUI_COLLECTION_MODULARIZATION_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M38_IMUI_COLLECTION_COMMAND_PACKAGE_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M39_IMUI_COLLECTION_SECOND_PROOF_SURFACE_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M40_IMUI_COLLECTION_HELPER_READINESS_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M41_IMUI_EDITOR_NOTES_INSPECTOR_COMMAND_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M42_IMUI_EDITOR_NOTES_DIRTY_STATUS_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M43_IMUI_NEXT_GAP_AUDIT_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M44_IMUI_EDITOR_NOTES_DRAFT_ACTIONS_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M45_IMUI_TEXTFIELD_DRAFT_BUFFER_CONTRACT_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M46_IMUI_TEXTFIELD_DRAFT_CONTROLLER_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M47_IMUI_CHILD_REGION_DEPTH_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M48_IMUI_MENU_TAB_TRIGGER_RESPONSE_SURFACE_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M49_IMUI_MENU_TAB_TRIGGER_RESPONSE_CANONICALIZATION_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M50_IMUI_WORKBENCH_SHELL_CLOSURE_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M51_IMUI_P2_DIAGNOSTICS_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M52_IMUI_P3_MULTIWINDOW_PACKAGE_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M53_DOCKING_P3_SOURCE_POLICY_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M54_DOCKING_MIXED_DPI_SUPPORT_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M55_DIAGNOSTICS_ENVIRONMENT_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M56_EMBEDDED_VIEWPORT_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M57_HELLO_WORLD_COMPARE_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M58_APP_UI_RENDER_ACCESSOR_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M59_VIEW_RUNTIME_GROUPED_STATE_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M60_FIRST_FRAME_BOOTSTRAP_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M61_ADVANCED_HELPER_CONTEXT_SOURCE_GATE_2026-05-01.md`
- `docs/workstreams/fret-examples-build-latency-v1/M62_GROUPED_STATE_SOURCE_GATE_CLOSURE_2026-05-01.md`
- `Cargo.toml`
