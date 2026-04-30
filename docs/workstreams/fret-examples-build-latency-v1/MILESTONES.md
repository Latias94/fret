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
