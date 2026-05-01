# Fret Examples Build Latency v1 - Milestones

Status: active

## M0 - Baseline And First Source Gate

Exit criteria:

- The lane records current assumptions and gates.
- One representative pure source-marker check runs without compiling `fret-examples`.
- The deleted Rust unit test has equivalent source coverage elsewhere.

## M1 - Source-Policy Test Migration Plan

Status: complete

Exit criteria:

- Remaining source-marker tests in `apps/fret-examples/src/lib.rs` are grouped by owner surface.
- Tests that only need text scanning have a Python gate migration plan.
- Tests that need Rust type checking remain in `fret-examples` with an explicit reason.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M1_SOURCE_POLICY_AUDIT_2026-04-29.md`
- `tools/gate_imui_facade_teaching_source.py`
- `tools/gate_table_source_policy.py`
- `tools/gate_examples_source_tree_policy.py`

## M2 - Demo Build Split Decision

Status: complete

Exit criteria:

- Single-demo build coupling is measured on at least one representative IMUI demo.
- The lane chooses between feature-family split, separate examples crates, or direct demo-local bins.
- The chosen split has a small compatibility gate before broad migration.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M2_DEMO_BUILD_SPLIT_DECISION_2026-04-29.md`
- `apps/fret-examples-imui/Cargo.toml`
- `tools/gate_fret_examples_imui_split_source.py`

## M3 - Profile Policy Decision

Status: complete

Exit criteria:

- The macOS incremental-link workaround is either kept global with evidence or narrowed through a
  documented developer profile path.
- Windows iteration guidance is updated if `dev-fast` becomes the recommended local path.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M3_PROFILE_POLICY_DECISION_2026-04-29.md`
- `Cargo.toml`

## M4 - IMUI Editor Theme Source Gate

Status: complete

Exit criteria:

- Source-only IMUI editor theme/preset markers no longer compile the monolithic `fret-examples`
  unit-test module.
- The existing IMUI facade/teaching source gate owns the markers.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M4_IMUI_EDITOR_THEME_SOURCE_GATE_2026-04-29.md`
- `tools/gate_imui_facade_teaching_source.py`

## M5 - IMUI State Source Gate

Status: complete

Exit criteria:

- IMUI local-state read markers and workspace-shell IMUI entry markers no longer compile the
  monolithic `fret-examples` unit-test module.
- The existing IMUI facade/teaching source gate owns the markers.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M5_IMUI_STATE_SOURCE_GATE_2026-04-29.md`
- `tools/gate_imui_facade_teaching_source.py`

## M6 - Workspace Shell Source Gate

Status: complete

Exit criteria:

- Workspace shell capability-helper markers no longer compile the monolithic `fret-examples`
  unit-test module.
- The examples source-tree policy gate owns the markers.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M6_WORKSPACE_SHELL_SOURCE_GATE_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`

## M7 - View Entry Source Gate

Status: complete

Exit criteria:

- Broad view-runtime AppUi alias and builder/run entry markers no longer compile the monolithic
  `fret-examples` unit-test module.
- The examples source-tree policy gate owns the markers.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M7_VIEW_ENTRY_SOURCE_GATE_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`

## M8 - Authoring Import Source Gate

Status: complete

Exit criteria:

- Grouped data, query facade, advanced entry alias, and docking import owner markers no longer
  compile the monolithic `fret-examples` unit-test module.
- The examples source-tree policy gate owns the markers.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M8_AUTHORING_IMPORT_SOURCE_GATE_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`

## M9 - Theme Source Gate

Status: complete

Exit criteria:

- Theme snapshot/read source markers no longer compile the monolithic `fret-examples` unit-test
  module.
- The examples source-tree policy gate owns the default app, advanced runtime, element-context, and
  renderer bridge theme-read markers.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M9_THEME_SOURCE_GATE_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`

## M10 - Local-State Bridge Source Gate

Status: complete

Exit criteria:

- Local-state-first default app markers, init-time `LocalState::new_in` markers, AppUi render-root
  bridge markers, and local-state component bridge markers no longer compile the monolithic
  `fret-examples` unit-test module.
- The examples source-tree policy gate owns those source-only checks.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M10_LOCAL_STATE_BRIDGE_SOURCE_GATE_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`

## M11 - Model-Read And Asset Helper Source Gate

Status: complete

Exit criteria:

- Grouped selector-model layout markers, driver-owned raw model-store read markers, state-owned
  GenUI helper markers, UI asset helper entrypoint markers, and embedded viewport driver extension
  markers no longer compile the monolithic `fret-examples` unit-test module.
- The examples source-tree policy gate owns those source-only checks.
- Parser/function behavior tests remain in Rust.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M11_MODEL_READ_ASSET_SOURCE_GATE_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`

## M12 - Advanced Reference Roster Source Gate

Status: complete

Exit criteria:

- Advanced/reference surface selection, advanced/reference classification comments, and
  `docs/examples/README.md` roster markers no longer compile the monolithic `fret-examples`
  unit-test module.
- The examples source-tree policy gate owns those source-only checks.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M12_ADVANCED_ROSTER_SOURCE_GATE_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`

## M13 - Default App Surface Source Gate

Status: complete

Exit criteria:

- Simple todo, query/query-async, and hello-counter default app surface markers no longer compile
  the monolithic `fret-examples` unit-test module.
- The examples source-tree policy gate owns those source-only checks.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M13_DEFAULT_APP_SURFACE_SOURCE_GATE_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`

## M14 - Source Gate Structure Split

Status: complete

Exit criteria:

- `python tools/gate_examples_source_tree_policy.py` remains the stable command entrypoint.
- The large implementation lives under `tools/examples_source_tree_policy/` so future source-policy
  slices can split by owner without changing callers.
- No source-policy behavior changes.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M14_SOURCE_GATE_STRUCTURE_SPLIT_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`
- `tools/examples_source_tree_policy/gate.py`

## M15 - Query Markdown Editor Notes Source Gate

Status: complete

Exit criteria:

- Query/query-async capability-first landing markers no longer compile the monolithic
  `fret-examples` unit-test module.
- Markdown layout-query and capability-first landing markers no longer compile the monolithic
  `fret-examples` unit-test module.
- Editor notes reusable-panel and workspace-shell root markers no longer compile the monolithic
  `fret-examples` unit-test module.
- The examples source-tree policy gate owns those source-only checks.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M15_QUERY_MARKDOWN_EDITOR_NOTES_SOURCE_GATE_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`
- `tools/examples_source_tree_policy/gate.py`

## M16 - Todo Async Playground Source Gate

Status: complete

Exit criteria:

- Todo default-app and capability-first root builder markers no longer compile the monolithic
  `fret-examples` unit-test module.
- Async playground `AppRenderContext` helper and root capability-first landing markers no longer
  compile the monolithic `fret-examples` unit-test module.
- The examples source-tree policy gate owns those source-only checks.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M16_TODO_ASYNC_PLAYGROUND_SOURCE_GATE_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`
- `tools/examples_source_tree_policy/gate.py`

## M17 - API Workbench Source Gate

Status: complete

Exit criteria:

- API workbench lite default-app, AppRenderContext helper, and capability-first landing markers no
  longer compile the monolithic `fret-examples` unit-test module.
- API workbench lite SQLite query/mutation ownership markers no longer compile the monolithic
  `fret-examples` unit-test module.
- The examples source-tree policy gate owns those source-only checks.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M17_API_WORKBENCH_SOURCE_GATE_2026-04-30.md`
- `tools/gate_examples_source_tree_policy.py`
- `tools/examples_source_tree_policy/gate.py`

## M18 - App Facing Source Gate Module Split

Status: complete

Exit criteria:

- `python tools/gate_examples_source_tree_policy.py` remains the stable command entrypoint.
- App-facing demo source-policy marker matrices live under
  `tools/examples_source_tree_policy/app_facing.py`.
- `tools/examples_source_tree_policy/gate.py` keeps orchestration and shared helpers without
  duplicating the app-facing marker matrices.
- No source-policy behavior changes.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M18_APP_FACING_SOURCE_GATE_MODULE_SPLIT_2026-04-30.md`
- `tools/examples_source_tree_policy/app_facing.py`
- `tools/examples_source_tree_policy/gate.py`

## M19 - Low Level Interop Source Gate

Status: complete

Exit criteria:

- Low-level interop direct-leaf root markers no longer compile the monolithic `fret-examples`
  unit-test module.
- The examples source-tree policy gate owns those source-only checks.
- Interop/source marker ownership lives under `tools/examples_source_tree_policy/interop.py`.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M19_LOW_LEVEL_INTEROP_SOURCE_GATE_2026-04-30.md`
- `tools/examples_source_tree_policy/interop.py`
- `tools/examples_source_tree_policy/gate.py`

## M20 - Manual UI Tree Source Gate

Status: complete

Exit criteria:

- Manual `UiTree<App>` root-wrapper markers no longer compile the monolithic `fret-examples`
  unit-test module.
- The examples source-tree policy gate owns those source-only checks.
- Manual root-wrapper source marker ownership lives under
  `tools/examples_source_tree_policy/manual.py`.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M20_MANUAL_UI_TREE_SOURCE_GATE_2026-04-30.md`
- `tools/examples_source_tree_policy/manual.py`
- `tools/examples_source_tree_policy/gate.py`

## M21 - Components Gallery Owner Split Source Gate

Status: complete

Exit criteria:

- Components gallery retained render/app-theme/driver-event owner split markers no longer compile
  the monolithic `fret-examples` unit-test module.
- The components gallery owner split audit markers no longer compile the monolithic
  `fret-examples` unit-test module.
- Owner-split source/document marker ownership lives under
  `tools/examples_source_tree_policy/owner_split.py`.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M21_COMPONENTS_GALLERY_OWNER_SPLIT_SOURCE_GATE_2026-04-30.md`
- `tools/examples_source_tree_policy/owner_split.py`
- `tools/examples_source_tree_policy/gate.py`

## M22 - Selected Raw Owner Source Gate

Status: complete

Exit criteria:

- Selected raw-owner escape-hatch markers no longer compile the monolithic `fret-examples`
  unit-test module.
- The examples source-tree policy gate owns those source-only checks.
- Raw-owner source policy entries spell their source root explicitly so split-crate IMUI examples
  are not resolved through `apps/fret-examples/src` by accident.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M22_SELECTED_RAW_OWNER_SOURCE_GATE_2026-04-30.md`
- `tools/examples_source_tree_policy/owner_split.py`
- `tools/examples_source_tree_policy/gate.py`

## M23 - IMUI Editor Proof Source Gate

Status: complete

Exit criteria:

- IMUI editor proof non-raw helper return markers no longer compile the monolithic
  `fret-examples` unit-test module.
- IMUI editor proof official adapter markers no longer compile the monolithic `fret-examples`
  unit-test module.
- IMUI editor proof app-owned sortable/dock helper markers and the matching app-owner audit markers
  are owned by the IMUI facade/teaching source gate.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M23_IMUI_EDITOR_PROOF_SOURCE_GATE_2026-04-30.md`
- `tools/gate_imui_facade_teaching_source.py`
- `apps/fret-examples/src/lib.rs`

## M24 - IMUI Interaction Showcase Source Gate

Status: complete

Exit criteria:

- IMUI interaction showcase compact rail layout markers no longer compile the monolithic
  `fret-examples` unit-test module.
- The interaction showcase grouped state/action markers no longer require
  `apps/fret-examples/src/lib.rs` to `include_str!` the split-crate IMUI showcase source.
- The IMUI facade/teaching source gate owns the showcase markers alongside the rest of the IMUI
  split-crate source policy.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M24_IMUI_INTERACTION_SHOWCASE_SOURCE_GATE_2026-04-30.md`
- `tools/gate_imui_facade_teaching_source.py`
- `apps/fret-examples/src/lib.rs`

## M25 - IMUI Response Signals Source Gate

Status: complete

Exit criteria:

- IMUI response signals menu/combo lifecycle markers no longer compile the monolithic
  `fret-examples` unit-test module.
- IMUI response signals canonical menu/tab trigger markers no longer compile the monolithic
  `fret-examples` unit-test module.
- The response signals grouped local-state markers no longer require
  `apps/fret-examples/src/lib.rs` to `include_str!` the split-crate IMUI response signals source.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M25_IMUI_RESPONSE_SIGNALS_SOURCE_GATE_2026-04-30.md`
- `tools/gate_imui_facade_teaching_source.py`
- `apps/fret-examples/src/lib.rs`

## M26 - IMUI P0 Workstream Source Gate

Status: complete

Exit criteria:

- IMUI response-status lifecycle follow-on document freeze markers no longer compile the monolithic
  `fret-examples` unit-test module.
- IMUI key-owner surface follow-on and M2 no-new-surface verdict markers no longer compile the
  monolithic `fret-examples` unit-test module.
- The closed IMUI workstreams point their source-policy gate at the Python gate instead of deleted
  Rust source-marker tests.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M26_IMUI_P0_WORKSTREAM_SOURCE_GATE_2026-04-30.md`
- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`

## M27 - IMUI Collection/Pane Workstream Source Gate

Status: complete

Exit criteria:

- IMUI collection/pane proof M1/M2/M3 workstream freeze markers no longer compile the monolithic
  `fret-examples` unit-test module.
- Collection-first asset-browser and pane-first workspace-shell proof-surface markers are covered
  by `tools/gate_imui_workstream_source.py`.
- The closed collection/pane proof workstream points its current source-policy gates at Python
  gates instead of deleted Rust source-marker tests.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M27_IMUI_COLLECTION_PANE_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`

## M28 - IMUI Facade Modularization Workstream Source Gate

Status: complete

Exit criteria:

- IMUI facade internal modularization workstream freeze markers no longer compile the monolithic
  `fret-examples` unit-test module.
- Roadmap/workstream-index/todo-tracker markers for the closed modularization lane are covered by
  `tools/gate_imui_workstream_source.py`.
- The closed facade modularization workstream points its current source-policy gate at the Python
  gate instead of a deleted Rust source-marker test.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M28_IMUI_FACADE_MODULARIZATION_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`

## M29 - IMUI Collection Box Select Workstream Source Gate

Status: complete

Exit criteria:

- IMUI collection box-select workstream freeze markers no longer compile the monolithic
  `fret-examples` unit-test module.
- The closed collection box-select workstream points its source-policy gate at the Python gate
  instead of a deleted Rust source-marker test.
- The real `proof_collection_*` unit tests remain in Rust and stay named as the behavior floor.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M29_IMUI_COLLECTION_BOX_SELECT_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`

## M30 - IMUI Collection Keyboard Owner Workstream Source Gate

Status: complete

Exit criteria:

- IMUI collection keyboard-owner workstream freeze markers no longer compile the monolithic
  `fret-examples` unit-test module.
- The closed collection keyboard-owner workstream points its source-policy gate at the Python gate
  instead of a deleted Rust source-marker test.
- The real `proof_collection_keyboard_*` unit tests remain in Rust and stay named as the behavior
  floor.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M30_IMUI_COLLECTION_KEYBOARD_OWNER_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`

## M31 - IMUI Collection Delete Action Workstream Source Gate

Status: complete

Exit criteria:

- IMUI collection delete-action workstream freeze markers no longer compile the monolithic
  `fret-examples` unit-test module.
- The closed collection delete-action workstream points its source-policy gate at the Python gate
  instead of a deleted Rust source-marker test.
- The real `proof_collection_delete_*` unit tests remain in Rust and stay named as the behavior
  floor.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M31_IMUI_COLLECTION_DELETE_ACTION_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`

## M32 - IMUI Collection Context Menu Workstream Source Gate

Status: complete

Exit criteria:

- IMUI collection context-menu workstream freeze markers no longer compile the monolithic
  `fret-examples` unit-test module.
- The closed collection context-menu workstream points its source-policy gate at the Python gate
  instead of a deleted Rust source-marker test.
- The real `proof_collection_context_menu_*` unit tests remain in Rust and stay named as the
  behavior floor.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M32_IMUI_COLLECTION_CONTEXT_MENU_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`

## M33 - IMUI Collection Zoom Workstream Source Gate

Status: complete

Exit criteria:

- IMUI collection zoom workstream freeze markers no longer compile the monolithic `fret-examples`
  unit-test module.
- The closed collection zoom workstream points its source-policy gate at the Python gate instead of
  a deleted Rust source-marker test.
- The real `proof_collection_layout_metrics_*` and `proof_collection_zoom_request_*` unit tests
  remain in Rust and stay named as the behavior floor.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M33_IMUI_COLLECTION_ZOOM_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`

## M34 - IMUI Collection Select-All Workstream Source Gate

Status: complete

Exit criteria:

- IMUI collection select-all workstream freeze markers no longer compile the monolithic
  `fret-examples` unit-test module.
- The closed collection select-all workstream points its source-policy gate at the Python gate
  instead of a deleted Rust source-marker test.
- The real `proof_collection_select_all_*` unit tests remain in Rust and stay named as the behavior
  floor.

Current evidence:

- `docs/workstreams/fret-examples-build-latency-v1/M34_IMUI_COLLECTION_SELECT_ALL_WORKSTREAM_SOURCE_GATE_2026-05-01.md`
- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
