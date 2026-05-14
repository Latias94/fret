# ImUi Dear ImGui Gap Closure v1 - Evidence & Gates

Status: Active
Last updated: 2026-05-14

## Evidence Anchors

- Current lane:
  - `docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-imgui-gap-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P0_CURRENT_SOURCE_AUDIT_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P1_CLEANUP_AUDIT_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P1_CLOSEOUT_AUDIT_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P2_GOLDEN_PATH_PROMOTION_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_PUBLIC_SURFACE_CATALOG_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_DESIGN_SURFACE_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_PORTING_SUGAR_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_CHILD_REGION_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_COLLECTION_HELPER_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_EXECUTION_PRIORITY_REVIEW_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/TODO.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/MILESTONES.md`
- Current Fret IMUI source:
  - `ecosystem/fret-imui/src/lib.rs`
  - `ecosystem/fret-imui/src/frontend.rs`
  - `ecosystem/fret-imui/src/tests/mod.rs`
  - `ecosystem/fret-imui/src/tests/harness/mod.rs`
  - `ecosystem/fret-imui/src/tests/harness/events.rs`
  - `ecosystem/fret-imui/src/tests/harness/floating_scenes.rs`
  - `ecosystem/fret-imui/src/tests/harness/frames.rs`
  - `ecosystem/fret-imui/src/tests/harness/host.rs`
  - `ecosystem/fret-imui/src/tests/harness/hover_scenes.rs`
  - `ecosystem/fret-imui/src/tests/harness/lookup.rs`
  - `ecosystem/fret-imui/src/tests/harness/services.rs`
  - `ecosystem/fret-imui/src/tests/composition/mod.rs`
  - `ecosystem/fret-imui/src/tests/floating/mod.rs`
  - `ecosystem/fret-imui/src/tests/interaction_drag/mod.rs`
  - `ecosystem/fret-imui/src/tests/interaction_menu_tabs/mod.rs`
  - `ecosystem/fret-imui/src/tests/interaction_press/mod.rs`
  - `ecosystem/fret-imui/src/tests/interaction_shortcuts/mod.rs`
  - `ecosystem/fret-imui/src/tests/label_identity/mod.rs`
  - `ecosystem/fret-imui/src/tests/models_combo/mod.rs`
  - `ecosystem/fret-imui/src/tests/models_controls/mod.rs`
  - `ecosystem/fret-imui/src/tests/models_text_area/mod.rs`
  - `ecosystem/fret-imui/src/tests/models_text_picker/mod.rs`
  - `ecosystem/fret-imui/src/tests/popup_hover/mod.rs`
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/disclosure.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/text_models.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/value_models.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs`
  - `ecosystem/fret-ui-editor/src/imui.rs`
  - `ecosystem/fret/src/lib.rs`
  - `apps/fret-cookbook/src/lib.rs`
- Current proof surfaces:
  - `apps/fret-cookbook/README.md`
  - `apps/fret-cookbook/EXAMPLES.md`
  - `docs/examples/README.md`
  - `apps/fret-cookbook/examples/imui_action_basics.rs`
  - `apps/fret-cookbook/examples/imui_debug_draw_basics.rs`
  - `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`
  - `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `tools/gate_imui_editor_collection_source.py`
  - `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
  - `ecosystem/fret-ui-kit/src/recipes/imui_sortable.rs`
  - `ecosystem/fret-ui-kit/src/recipes/imui_drag_preview.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
  - `apps/fret-examples/src/docking_arbitration_demo.rs`
  - `tools/gate_imui_workstream_source.py`
  - `tools/diag_gate_imui_product_chain.py`
  - `tools/diag_gate_imui_p2_devtools_first_open.py`
- Prior status:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Dear ImGui reference:
  - `repo-ref/imgui/imgui.h`
  - `repo-ref/imgui/imgui.cpp`
  - `repo-ref/imgui/imgui_draw.cpp`
  - `repo-ref/imgui/imgui_demo.cpp`
  - `repo-ref/imgui/docs/BACKENDS.md`

## P3 Public Surface Catalog Gates

Use these for the current public-surface catalog note:

```powershell
python tools/audit_crate.py --crate fret-imui
python tools/audit_crate.py --crate fret-ui-kit
python tools/audit_crate.py --crate fret-ui-editor
python tools/audit_crate.py --crate fret
rg -n "pub mod imui|pub use fret_imui|pub use fret_ui_kit::imui|pub mod kit|pub mod editor|pub mod docking|pub mod prelude" ecosystem/fret/src/lib.rs
cargo nextest run -p fret root_surface_exposes_explicit_imui_module readme_and_rustdoc_expose_imui_as_explicit_optional_surface --no-fail-fast
cargo check -p fret --no-default-features --features imui
```

## P3 Component Surface Catalog Gates

Use these for the current component-surface catalog note:

```powershell
rg --files ecosystem/fret-ui-kit/src/imui ecosystem/fret-ui-kit/tests
rg -n "pub use debug_draw_controls|pub use options|pub use response|pub use tab_family_controls::ImUiTabBar|pub use table_controls" ecosystem/fret-ui-kit/src/imui.rs
rg -n "fn (button|small_button|arrow_button|checkbox_model|radio|switch_model|slider_f32_model|combo|combo_model|selectable|multi_selectable|tree_node|collapsing_header|child_region|virtual_list|table|tab_bar|open_popup|begin_popup|tooltip|drag_source|drop_target|debug_draw)" ecosystem/fret-ui-kit/src/imui/facade_writer.rs ecosystem/fret-ui-kit/src/imui/facade_writer
rg -n "pub fn (text_field|checkbox|color_edit|drag_value|numeric_input|slider|enum_select|property_grid|gradient_editor|inspector_panel)" ecosystem/fret-ui-editor/src/imui.rs
rg -n "Widgets: Text|Widgets: Main|Widgets: Combo Box|Widgets: Trees|Widgets: Selectables|Widgets: List Boxes|Widgets: Data Plotting|Widgets: Menus|Tooltips|Popups, Modals|Tables|Tab Bars|Drag and Drop|Debug Utilities" repo-ref/imgui/imgui.h
cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_combo_smoke --test imui_table_smoke --test imui_disclosure_smoke --test imui_textarea_smoke --test imui_drag_drop_smoke --test imui_virtual_list_smoke --test imui_debug_draw_smoke --test imui_tooltip_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy --no-fail-fast
```

## P3 Design Surface Readiness Gates

Use these for the current design/theme readiness note:

```powershell
rg -n "EditorThemePresetV1|ImguiLikeDense|install_editor_theme_preset_v1|reapply_installed_editor_theme_preset_v1" ecosystem/fret-ui-editor/src/theme.rs apps/fret-cookbook/examples/imui_editor_controls_basics.rs apps/fret-examples/src/imui_editor_proof_demo.rs
rg -n "component\\.imui\\.disabled_alpha|imui_text_input_style_from_theme|input_text_model_uses_compact_imui_chrome_without_focus_ring|textarea_model_uses_compact_imui_chrome_without_focus_ring|hovered_like_imgui|ImUiHoveredFlags" ecosystem/fret-ui-kit/src/imui
rg -n "ShowStyleEditor|ImGuiStyle|PushStyleColor|PushStyleVar|StyleColorsDark|StyleColorsLight|StyleColorsClassic" repo-ref/imgui/imgui.h repo-ref/imgui/imgui_demo.cpp
cargo nextest run -p fret-ui-editor default_preset_keeps_existing_editor_patch_baseline imgui_like_dense_preset_overrides_density_and_field_chrome installed_preset_can_be_reapplied_after_base_theme_reset --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui input_text_model_uses_compact_imui_chrome_without_focus_ring textarea_model_uses_compact_imui_chrome_without_focus_ring --no-fail-fast
```

## P3 Porting Sugar Readiness Gates

Use these for the current porting-sugar readiness note:

```powershell
rg -n "SameLine|PushItemWidth|SetNextItemWidth|CalcItemWidth|PushID|##|###" repo-ref/imgui/imgui.h repo-ref/imgui/imgui.cpp repo-ref/imgui/imgui_demo.cpp
rg -n "row\\(|horizontal\\(|horizontal_with_options|row_with|id_source|test_id|push_id" ecosystem/fret-imui/src/frontend.rs ecosystem/fret-ui-kit/src/imui/facade_writer.rs ecosystem/fret-ui-kit/src/imui/facade_writer ecosystem/fret-ui-kit/src/imui/options/containers.rs apps/fret-cookbook/examples/imui_action_basics.rs apps/fret-cookbook/examples/imui_editor_controls_basics.rs apps/fret-examples/src/imui_editor_proof_demo.rs
rg -n "row_cx\\.row_options" apps/fret-examples/src/imui_editor_proof_demo.rs apps/fret-examples/src/editor_notes_demo.rs
cargo check -p fret-demo --bin imui_editor_proof_demo
```

Run evidence:

- 2026-05-14: removed proof-surface calls that manually passed
  `PropertyRow::new().options(row_cx.row_options.clone())` in
  `apps/fret-examples/src/imui_editor_proof_demo.rs` and `apps/fret-examples/src/editor_notes_demo.rs`.
  Default property-row policy now stays centralized in `PropertyGridRowCx::row_with(...)` instead
  of leaking into app/proof code.
- 2026-05-14: `tools/gate_imui_workstream_source.py` now rejects `row_cx.row_options` in both
  proof surfaces so future cleanup cannot regress into ad hoc row-policy wiring.
- 2026-05-14: `cargo check -p fret-demo --bin imui_editor_proof_demo`,
  `cargo check -p fret-demo --bin editor_notes_demo`,
  `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast`,
  `python tools/gate_imui_workstream_source.py`, `python tools/gate_imui_facade_teaching_source.py`,
  and `git diff --check` passed locally.

## P3 Child Region Readiness Gates

Use these for the current child-region readiness note:

```powershell
cargo nextest run -p fret-imui child_region --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast
cargo check -p fret-demo --bin workspace_shell_demo
```

## P3 Collection Helper Readiness Gates

Use these for the current collection-helper readiness note:

```powershell
python tools/gate_imui_editor_collection_source.py
cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke --test imui_sortable_recipe_smoke --test imui_drag_preview_smoke --no-fail-fast
```

## P3 Execution Priority Review Gates

Use these when changing the P3 execution-priority read:

```powershell
python tools/diag_gate_imui_product_chain.py
python tools/audit_crate.py --crate fret-imui
python tools/audit_crate.py --crate fret-ui-kit
python tools/audit_crate.py --crate fret-ui-editor
python tools/audit_crate.py --crate fret
python tools/gate_imui_workstream_source.py
python tools/gate_imui_facade_teaching_source.py
python tools/check_workstream_catalog.py
git diff --check
```

## P3 Product Chain Gate

Use this gate before treating a Dear ImGui-class gap as widget/API breadth. It validates the current
product chain's discovery path, promoted script/suite inputs, and source gates across the generic
IMUI cookbook path, editor-controls cookbook path, editor proof, and workspace shell:

```powershell
python tools/diag_gate_imui_product_chain.py
```

Run evidence:

- 2026-05-14: `python tools/diag_gate_imui_product_chain.py` passed locally.
- The run validated cookbook example discovery, native demo discovery, tool-app discovery,
  campaign doctor output, all promoted product-chain diagnostics scripts, and both IMUI source
  gates.
- This is not a replacement for launched GUI gates. It is the first-open maintenance gate that keeps
  the cross-app product path visible before opening a new helper or widget follow-on.

## Runtime Boundary Gate

Use this gate to protect the core architectural constraint for this lane: `fret-imui` stays a thin
immediate facade over `fret-authoring` + `fret-ui`; policy-heavy IMUI helpers stay in
`fret-ui-kit::imui`, editor controls stay in `fret-ui-editor`, and docking/workspace behavior stays
outside the runtime facade.

```powershell
python tools/gate_imui_workstream_source.py
python tools/audit_crate.py --crate fret-imui
python tools/check_layering.py
```

Run evidence:

- 2026-05-14: `python tools/audit_crate.py --crate fret-imui` passed and reported direct runtime
  dependencies only on `fret-authoring` and `fret-ui`; `fret-ui-kit` remains dev-only for focused
  behavior tests.
- 2026-05-14: `python tools/check_layering.py` passed.
- 2026-05-14: `python tools/gate_imui_workstream_source.py` now validates
  `ecosystem/fret-imui/Cargo.toml` `[dependencies]` and rejects policy/runtime drift such as
  `fret-ui-kit`, `fret-ui-editor`, `fret-docking`, `fret-workspace`, `fret-ui-shadcn`, `winit`, or
  `wgpu` in the runtime dependency section.

## IMUI Maintainer Test Ownership / Table Layout Gate

Use this gate when changing menu/tab interaction ownership, table cell semantics, or IMUI layout
mechanics that can accidentally make diagnostics bounds diverge from visual column layout.

```powershell
cargo nextest run -p fret-imui composition --no-fail-fast
cargo nextest run -p fret-imui floating --no-fail-fast
cargo nextest run -p fret-imui interaction_drag --no-fail-fast
cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast
cargo nextest run -p fret-imui interaction_press --no-fail-fast
cargo nextest run -p fret-imui interaction_shortcuts --no-fail-fast
cargo nextest run -p fret-imui label_identity --no-fail-fast
cargo nextest run -p fret-imui models_combo --no-fail-fast
cargo nextest run -p fret-imui models_controls --no-fail-fast
cargo nextest run -p fret-imui models_text_area --no-fail-fast
cargo nextest run -p fret-imui models_text_picker --no-fail-fast
cargo nextest run -p fret-imui popup_hover --no-fail-fast
cargo nextest run -p fret-imui table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells --no-fail-fast
cargo nextest run -p fret-imui --no-fail-fast
python tools/gate_imui_workstream_source.py
git diff --check
```

Run evidence:

- 2026-05-14: moved the shared IMUI test host, fake text/path/svg/material service, event
  dispatch helpers, geometry helpers, and floating overlay harness from
  `ecosystem/fret-imui/src/tests/mod.rs` into the shared IMUI test harness.
  `mod.rs` now only imports the harness and owns the test-module index.
- 2026-05-14: split the shared IMUI test harness into owner modules under
  `ecosystem/fret-imui/src/tests/harness/`: `services`, `host`, `frames`, `events`, `lookup`,
  `hover_scenes`, and `floating_scenes`.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/composition.rs` file into
  `mount_smoke`, `control_geometry`, and `layout_collections` test owners under
  `ecosystem/fret-imui/src/tests/composition/`.
- 2026-05-14: `cargo nextest run -p fret-imui composition --no-fail-fast` passed locally with 19
  tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/floating.rs` file into
  `movement_z_order`, `input_modes`, `layer_dismissal`, and `window_options` test owners under
  `ecosystem/fret-imui/src/tests/floating/`.
- 2026-05-14: `cargo nextest run -p fret-imui floating --no-fail-fast` passed locally with 25
  tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/interaction_drag.rs` file
  into `multi_select`, `collection_drag`, `drag_core`, `drag_preview`, and `sortable` test owners
  under `ecosystem/fret-imui/src/tests/interaction_drag/`.
- 2026-05-14: `cargo nextest run -p fret-imui interaction_drag --no-fail-fast` passed locally with
  8 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
  file into `menu_activation`, `submenu_hover`, `submenu_shortcuts`, and `tabs` test owners under
  `ecosystem/fret-imui/src/tests/interaction_menu_tabs/`.
- 2026-05-14: `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast` passed
  locally with 18 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/interaction_press.rs`
  file into `click_edges`, `lifecycle`, `context_menu`, and `press_hold` test owners under
  `ecosystem/fret-imui/src/tests/interaction_press/`.
- 2026-05-14: `cargo nextest run -p fret-imui interaction_press --no-fail-fast` passed
  locally with 9 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/interaction_shortcuts.rs`
  file into `command_metadata`, `button_shortcuts`, `selectable_shortcuts`, and `disclosure_tree`
  test owners under `ecosystem/fret-imui/src/tests/interaction_shortcuts/`.
- 2026-05-14: `cargo nextest run -p fret-imui interaction_shortcuts --no-fail-fast` passed
  locally with 10 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/label_identity.rs` file
  into `visible_suffixes`, `model_controls`, `explicit_ids`, and `table_headers` test owners
  under `ecosystem/fret-imui/src/tests/label_identity/`.
- 2026-05-14: `cargo nextest run -p fret-imui label_identity --no-fail-fast` passed locally
  with 7 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/models_combo.rs` file into
  `combo_model` and `combo_direct` test owners under
  `ecosystem/fret-imui/src/tests/models_combo/`.
- 2026-05-14: `cargo nextest run -p fret-imui models_combo --no-fail-fast` passed locally with 11
  tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/models_controls.rs` file
  into `checkbox`, `switch`, and `slider` test owners under
  `ecosystem/fret-imui/src/tests/models_controls/`.
- 2026-05-14: `cargo nextest run -p fret-imui models_controls --no-fail-fast` passed locally
  with 6 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/models_text_area.rs`
  file into `modes`, `commands`, `model_changed`, and `lifecycle` test owners under
  `ecosystem/fret-imui/src/tests/models_text_area/`.
- 2026-05-14: `cargo nextest run -p fret-imui models_text_area --no-fail-fast` passed locally
  with 8 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/models_text_picker.rs`
  file into `completion_popup`, `history_popup`, `completion_keyboard`, `history_keyboard`, and
  `empty_keyboard` test owners under `ecosystem/fret-imui/src/tests/models_text_picker/`.
- 2026-05-14: `cargo nextest run -p fret-imui models_text_picker --no-fail-fast` passed
  locally with 6 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/popup_hover.rs` file into
  `context_basics`, `hover_flags`, `item_keyboard`, `item_pointer`, and `lifecycle_modal` test
  owners under `ecosystem/fret-imui/src/tests/popup_hover/`.
- 2026-05-14: `cargo nextest run -p fret-imui popup_hover --no-fail-fast` passed locally with 21
  tests.
- 2026-05-14: `cargo nextest run -p fret-imui
  table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells --no-fail-fast` passed
  locally after moving table body-cell test semantics to layout-transparent
  `SemanticsDecoration`.
- 2026-05-14: `cargo nextest run -p fret-imui --no-fail-fast` passed locally with 163 tests.
- 2026-05-14: `python tools/gate_imui_workstream_source.py` and `git diff --check` passed locally.

## P3 Diagnostics / DevTools First-Open Gate

Use this gate for the current Dear ImGui-class diagnostics discoverability read. It verifies the
shared CLI-first path that DevTools GUI and MCP consume later: direct script run, named bundle
capture, latest bundle resolution through `script.result.json:last_bundle_dir`, bundle compare,
campaign execution, `diag summarize`, and `diag dashboard`.

```powershell
python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke
```

Run evidence:

- 2026-05-14: `python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke-2026-05-14-gap-refresh-rerun --timeout-ms 240000` passed locally.
- Direct script result:
  `target/imui-p2-devtools-first-open-smoke-2026-05-14-gap-refresh-rerun/1778714082990/direct/sessions/1778714086733-135148/script.result.json`
  reports `stage=passed`, `run_id=1778714090298`, and `last_bundle_dir=1778714090682-todo-after-remove`.
- Campaign summary:
  `target/imui-p2-devtools-first-open-smoke-2026-05-14-gap-refresh-rerun/1778714082990/campaign/campaigns/devtools-first-open-smoke/1778714091193/regression.summary.json`
  reports `items_total=1`, `passed=1`, and zero deterministic/flaky/tooling/timeout failures.
- Suite summary:
  `target/imui-p2-devtools-first-open-smoke-2026-05-14-gap-refresh-rerun/1778714082990/campaign/campaigns/devtools-first-open-smoke/1778714091193/script-results/01-tools-diag-scripts-tooling-todo-todo-baseline.json/suite.summary.json`
  reports `status=passed` and `stage_counts.passed=1`.

## P0 Gates

Run these after doc edits in the first slice:

```powershell
python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/gate_imui_facade_teaching_source.py
python tools/gate_imui_workstream_source.py
rustfmt --edition 2024 --check apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs
rustfmt --edition 2024 --check apps/fret-examples/src/workspace_shell_demo.rs
rustfmt --edition 2024 --check apps/fret-examples/src/imui_editor_proof_demo.rs apps/fret-examples/src/imui_editor_proof_demo/collection.rs
cargo check -p fret-examples-imui
cargo check -p fret-demo --bin workspace_shell_demo
cargo check -p fret-demo --bin imui_editor_proof_demo
git diff --check
```

## Focused Code Gates For First Implementation Slice

Use these once the lane moves from audit/docs into code cleanup:

```powershell
cargo nextest run -p fret-imui --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy --no-fail-fast
```

## User-Usable Golden Path Gates

Use these to validate the current editor-panel proof surface:

```powershell
python tools/gate_imui_editor_collection_source.py
cargo check -p fret-demo --bin imui_editor_proof_demo
rg -n "imui_editor_proof_demo|state, command actions|command/action dispatch" apps/fret-cookbook/README.md apps/fret-cookbook/EXAMPLES.md docs/examples/README.md
```

## Runnable Proof Surfaces

```powershell
cargo run -p fret-cookbook --features cookbook-imui --example imui_action_basics
cargo run -p fret-cookbook --features cookbook-imui --example imui_debug_draw_basics
cargo run -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics
cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fret-demo --bin workspace_shell_demo
cargo run -p fret-demo --bin docking_arbitration_demo
```

## Gate Interpretation

- Passing source gates proves the current teaching/doc surfaces remain within the intended owner
  split. It does not prove Dear ImGui parity.
- Passing focused crate tests proves current helper behavior did not regress. It does not justify
  widening public APIs.
- Public helper widening still needs a separate follow-on, two proof surfaces, and a focused gate.
