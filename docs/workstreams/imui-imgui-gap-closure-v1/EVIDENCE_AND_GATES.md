# ImUi Dear ImGui Gap Closure v1 - Evidence & Gates

Status: Active
Last updated: 2026-05-20

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
  - `docs/workstreams/imui-imgui-gap-closure-v1/P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md`
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
  - `ecosystem/fret-ui-kit/src/declarative/text.rs`
  - `ecosystem/fret-ui-kit/src/declarative/file_tree.rs`
  - `ecosystem/fret-ui-kit/src/declarative/table.rs`
  - `ecosystem/fret-ui-kit/src/imui/control_chrome.rs`
  - `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/separator_text_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/disclosure.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/text_models.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/value_models.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs`
  - `ecosystem/fret-ui-kit/src/imui/image_item_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/response/widgets.rs`
  - `ecosystem/fret-ui-kit/tests/imui_image_item_smoke.rs`
  - `ecosystem/fret-ui-editor/src/imui.rs`
  - `ecosystem/fret-ui-editor/src/primitives/drag_value_core.rs`
  - `ecosystem/fret-ui-editor/src/primitives/input_group.rs`
  - `ecosystem/fret-ui-editor/src/primitives/popup_list.rs`
  - `ecosystem/fret-ui-editor/src/primitives/readout.rs`
  - `ecosystem/fret-ui-editor/src/composites/property_group.rs`
  - `ecosystem/fret-ui-editor/src/composites/property_row.rs`
  - `ecosystem/fret-ui-editor/src/controls/field_status.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit/popup/copy.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit/popup/numeric.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit/popup/options.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit/popup/preview.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit/popup/tooltip.rs`
  - `ecosystem/fret-ui-editor/src/controls/drag_value.rs`
  - `ecosystem/fret-ui-editor/src/controls/axis_drag_value.rs`
  - `ecosystem/fret-ui-editor/src/controls/enum_select.rs`
  - `ecosystem/fret-ui-editor/src/controls/text_assist_field.rs`
  - `ecosystem/fret/src/lib.rs`
  - `apps/fret-cookbook/src/lib.rs`
- Closed image item and child resize follow-ons:
  - `docs/workstreams/imui-image-item-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-image-item-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-image-item-proof-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-child-region-resize-y-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-child-region-resize-y-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-child-region-resize-x-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-child-region-resize-x-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-selectable-highlight-policy-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-selectable-highlight-policy-v1/EVIDENCE_AND_GATES.md`
- Current proof surfaces:
  - `apps/fret-cookbook/README.md`
  - `apps/fret-cookbook/EXAMPLES.md`
  - `docs/examples/README.md`
  - `apps/fret-cookbook/examples/imui_action_basics.rs`
  - `apps/fret-cookbook/examples/imui_debug_draw_basics.rs`
  - `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`
  - `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`
  - `apps/fret-examples-imui/src/imui_interaction_showcase_demo.rs`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `tools/gate_imui_editor_collection_source.py`
  - `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
  - `ecosystem/fret-ui-kit/src/recipes/imui_sortable.rs`
  - `ecosystem/fret-ui-kit/src/recipes/imui_drag_preview.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
  - `apps/fret-examples/src/docking_demo.rs`
  - `apps/fret-examples/src/docking_arbitration_demo.rs`
  - `apps/fret-examples/src/container_queries_docking_demo.rs`
  - `apps/fret-examples/src/form_demo.rs`
  - `apps/fret-examples/src/sonner_demo.rs`
  - `apps/fret-examples/src/echarts_demo.rs`
  - `apps/fret-examples/tests/docking_demo_surface.rs`
  - `apps/fret-examples/tests/docking_arbitration_surface.rs`
  - `apps/fret-examples/tests/container_queries_docking_surface.rs`
  - `apps/fret-examples/tests/form_demo_surface.rs`
  - `apps/fret-examples/tests/sonner_demo_surface.rs`
  - `apps/fret-examples/tests/echarts_demo_surface.rs`
  - `apps/fret-ui-gallery/src/driver/toaster.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/sidebar/app_sidebar.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/command/behavior_demos.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/command/composable_shell.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/basic.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/borders.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/card.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/demo.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/disabled.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/focusable_disabled.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/multiple.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/showcase.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/children.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/flex_1_items.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/full_width_items.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/label.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/outline.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/spacing.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/children.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/demo.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/disabled.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/label.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/outline.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/size.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/with_text.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/button/children.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/tabs/icons.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/tabs/parts.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/tabs/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/basic.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/controlled_state.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/demo.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/file_tree.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/alert_dialog/rich_content.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/alert_dialog/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/alert_dialog/small_with_media.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/basic.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/children.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/demo.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/positioning.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/sides.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/trigger_delays.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/popover/align.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/tooltip/keyboard_shortcut.rs`
  - `apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/mvp/gates.rs`
  - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
  - `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs`
  - `ecosystem/fret-ui-ai/src/elements/mod.rs`
  - `ecosystem/fret-ui-ai/src/surface_policy_tests.rs`
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
rg -n "fn (text|text_wrapped|button|small_button|arrow_button|checkbox_model|radio|switch_model|slider_f32_model|combo|combo_model|selectable|multi_selectable|tree_node|collapsing_header|child_region|virtual_list|table|tab_bar|open_popup|begin_popup|tooltip|drag_source|drop_target|debug_draw)" ecosystem/fret-ui-kit/src/imui/facade_writer.rs ecosystem/fret-ui-kit/src/imui/facade_writer
rg -n "pub fn (text_field|checkbox|color_edit|drag_value|numeric_input|slider|enum_select|property_grid|gradient_editor|inspector_panel)" ecosystem/fret-ui-editor/src/imui.rs
rg -n "Widgets: Text|Widgets: Main|Widgets: Combo Box|Widgets: Trees|Widgets: Selectables|Widgets: List Boxes|Widgets: Data Plotting|Widgets: Menus|Tooltips|Popups, Modals|Tables|Tab Bars|Drag and Drop|Debug Utilities" repo-ref/imgui/imgui.h
cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_combo_smoke --test imui_table_smoke --test imui_disclosure_smoke --test imui_textarea_smoke --test imui_drag_drop_smoke --test imui_virtual_list_smoke --test imui_debug_draw_smoke --test imui_tooltip_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy --no-fail-fast
```

Run evidence:

- 2026-05-14: made `FloatingAreaContext` externally opaque in
  `ecosystem/fret-ui-kit/src/imui/floating_options.rs`. The facade still hands callers a context
  with `id()`, `position()`, and `drag_kind()` accessors, but external code can no longer construct
  or mutate area identity / drag-kind fields. `tools/gate_imui_workstream_source.py` rejects public
  fields from returning.
- 2026-05-14: made `FloatingAreaResponse` / `FloatingWindowResponse` accessor-first too. Floating
  response identity, geometry, drag, resize, and collapse state now stay behind methods instead of
  public fields, and the floating tests use `resp.id()` instead of `resp.area.id`.
- 2026-05-16: refreshed the public `window(...)` posture in
  `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`. The source docs now describe the current
  in-window floating surface instead of deferring z-order/focus arbitration to a future work item.
  Evidence anchors: `ecosystem/fret-imui/src/tests/floating/movement_z_order.rs` covers
  bring-to-front hit-test order, `input_modes.rs` covers focus-on-click vs activation plus
  no-inputs / pointer-pass-through behavior, and `window_options.rs` covers close, resize, and
  collapse policy.
- 2026-05-16: narrowed the table advanced-gap wording in
  `docs/workstreams/imui-imgui-gap-closure-v1/P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`.
  `TableOptions::striped` remains the existing alternating row-background policy, with proof in
  `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`,
  `ecosystem/fret-ui-kit/tests/imui_table_smoke.rs`, and
  `ecosystem/fret-imui/src/tests/composition/layout_collections.rs`.
- 2026-05-16: landed the explicit table row/cell background override slice. Evidence anchors:
  `TableRowOptions::background`, `TableCellOptions::background`,
  `ImUiTableRow::cell_with_options(...)`, `ImUiTableRow::cell_text_with_options(...)`,
  `ecosystem/fret-ui-kit/tests/imui_table_smoke.rs`, and
  `ecosystem/fret-imui/src/tests/composition/layout_collections.rs`.
  `table_helper_applies_explicit_row_and_cell_background_overrides` proves that explicit cell
  backgrounds paint after explicit row backgrounds.
- 2026-05-16: introduced `text_table_cell(...)` as the first shared table-cell text role and wired
  `ImUiTableRow::cell_text(...)` through it instead of bare paragraph text. Gate:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  table_cell_text_uses_compact_single_line_truncation --no-fail-fast`.
- 2026-05-16: routed sortable and plain IMUI table header labels through
  `text_table_cell(...)` too. Header labels now share compact single-line ellipsis semantics with
  body cells instead of default word wrapping. Gate: `cargo nextest run -p fret-ui-kit --features
  imui --lib table_header_label_uses_shared_table_cell_text_role --no-fail-fast`.
- 2026-05-17: routed sortable IMUI table header sort indicators through `text_chrome_glyph(...)`
  instead of bare `cx.text(...)`. Header labels remain table-cell text, while sort arrows are now
  fixed chrome glyphs with single-line clip semantics. Gate: `cargo nextest run -p fret-ui-kit
  --features imui --lib table_sort_indicator_uses_shared_chrome_glyph_text_role
  --no-fail-fast`.
- 2026-05-16: added static table column visibility through `TableColumn::hidden()` and
  `TableColumn::with_visible(bool)`. Hidden columns still consume author-submitted row cells in
  declared column order, but they do not render header/body cells and do not emit header responses.
  Gates: `cargo nextest run -p fret-ui-kit --features imui --lib
  hidden_table_columns_do_not_render_header_body_or_response --no-fail-fast`, `cargo nextest run -p
  fret-ui-kit --features imui --test imui_table_smoke table_column_visibility_helpers_compile
  --no-fail-fast`, and `cargo nextest run -p fret-imui
  table_helper_skips_hidden_columns_in_header_and_body --no-fail-fast`.
- 2026-05-17: added runtime table-column visibility state through
  `ImUiTableColumnVisibilityState`. The helper stays in `fret-ui-kit::imui`, keeps storage opaque,
  applies stable-id overrides to `TableColumn` lists before render, and intentionally leaves
  persistence, freeze panes, and durable column storage outside the model helper. Gates: `cargo
  nextest run -p fret-ui-kit --features imui --lib
  visibility_state_applies_runtime_overrides_by_stable_column_id
  visibility_state_leaves_unlisted_and_unidentified_columns_at_declared_visibility
  visibility_state_toggle_uses_current_override_or_default_visibility --no-fail-fast`, `cargo
  nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  table_column_visibility_state_applies_runtime_visibility_by_column_id --no-fail-fast`, `cargo
  nextest run -p fret-imui table_helper_applies_runtime_column_visibility_state
  table_helper_skips_hidden_columns_in_header_and_body --no-fail-fast`, and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-17: added a table column visibility menu-item bridge through
  `table_column_visibility_menu_item(...)`. The helper stays in `fret-ui-kit::imui`, reuses the
  existing checkbox menu-item behavior, updates `ImUiTableColumnVisibilityState`, and remains
  usable for custom popup/menu surfaces even after the default header context-menu helper below.
  Gates: `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`,
  `cargo nextest run -p fret-imui table_column_visibility_menu_item_updates_visibility_state
  --no-fail-fast`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: added a table column visibility menu-items group helper through
  `table_column_visibility_menu_items(...)`. The helper filters to stable-id, human-labeled
  columns, clones caller-owned item options, returns opaque/accessor-first per-column responses,
  and feeds both custom popup surfaces and the default header context-menu helper below. Gates:
  `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`,
  `cargo nextest run -p fret-imui
  table_column_visibility_menu_items_update_shared_visibility_state_and_filter_columns
  --no-fail-fast`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: added table header context-menu request reporting for sortable and plain headers
  through a shared private header trigger surface in `fret-ui-kit::imui`. Sortable headers still
  own button-like primary activation/click lifecycle, while plain headers only expose context-menu
  request signals. `TableHeaderResponse::response()` now reports right-click context-menu requests
  with a pointer anchor, plus keyboard requests from the ContextMenu key and Shift+F10; the helper
  below consumes this response signal for the default visibility menu surface. Gates: `cargo
  nextest run -p fret-imui table_plain_header_left_click_does_not_activate_or_click
  table_plain_header_reports_context_menu_request_from_keyboard_without_clicking
  table_column_visibility_header_context_menu_opens_from_plain_header
  table_sortable_header_reports_context_menu_request --no-fail-fast`, `cargo nextest run -p
  fret-imui interaction_press interaction_menu_tabs --no-fail-fast`, and `cargo nextest run -p
  fret-ui-kit --features imui --test imui_response_contract_smoke --test imui_table_smoke
  --no-fail-fast`.
- 2026-05-17: added automatic table header visibility-menu wiring through
  `table_column_visibility_header_context_menu(...)`. The helper stays in `fret-ui-kit::imui`,
  scans `TableResponse` header responses from both sortable and plain headers for context-menu
  requests, opens a popup menu with the existing popup policy, renders
  `table_column_visibility_menu_items(...)`, and returns an opaque/accessor-first response.
  `TableColumnVisibilityHeaderContextMenuOptions` exposes popup and menu-item policy instead of
  hard-coding placement/sizing. Callers still own when to apply `ImUiTableColumnVisibilityState` to
  their columns; persistence, freeze panes, and old columns API shape were still separate
  follow-ons at that historical point.
  Gates: `cargo nextest run -p fret-imui
  table_column_visibility_header_context_menu_opens_from_plain_header
  table_column_visibility_header_context_menu_opens_and_updates_state
  table_column_visibility_menu_items_update_shared_visibility_state_and_filter_columns
  table_sortable_header_reports_context_menu_request --no-fail-fast`, `cargo nextest run -p
  fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`, and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-17: added the narrow persistence seam for runtime table-column visibility through
  `TableColumnVisibilitySnapshot` and `TableColumnVisibilityEntry`. The public data shape
  serializes stable column ids as `id` and visible flags as `visible`, while
  `ImUiTableColumnVisibilityState::snapshot()`, `from_snapshot(...)`, and
  `replace_from_snapshot(...)` keep runtime storage opaque and caller-owned. Restore ignores empty
  ids and duplicate entries use last-entry-wins. This does not add file storage, schema registry,
  freeze panes, or a mutable table runtime to `fret-imui`. Gates: `cargo nextest run -p
  fret-ui-kit --features imui --lib visibility_state_snapshot_roundtrips_stable_column_ids
  visibility_state_snapshot_restore_ignores_empty_ids_and_last_entry_wins --no-fail-fast`,
  `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  table_column_visibility_snapshot_api_compiles_and_roundtrips
  table_column_visibility_snapshot_entries_are_public_data_shape --no-fail-fast`, and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-17: added IMUI table column pinning as the first narrow freeze-pane seam. `TableColumn`
  now exposes `TableColumn::pinned_left()`, `TableColumn::pinned_right()`, and `with_pin(...)`;
  `TableOptions` accepts an optional `horizontal_scroll` handle. The helper render path partitions visible header/body cells
  into left/center/right groups and keeps frozen left/right cells outside the shared center X-scroll
  region. This uses `fret-ui` scroll mechanics and does not add table-state storage to `fret-imui`.
  Gates: `cargo nextest run -p fret-ui-kit --features imui --lib
  horizontal_scroll_option_wraps_unpinned_header_and_body_center_groups --no-fail-fast`,
  `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  table_column_pinning_helpers_compile --no-fail-fast`, `cargo nextest run -p fret-imui
  table_helper_pins_left_and_right_columns_while_center_columns_scroll --no-fail-fast`, and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: added the first accessor-first cleanup for `TableColumn`. New read accessors cover
  `header()`, `id()`, `width()`, `visible()`, `is_sortable()`, `sort_direction()`,
  `resize_options()`, and `pin()`. `ImUiTableColumnVisibilityState::apply_to_columns(...)` now uses
  a crate-local visibility mutator, while table rendering, visibility menu policy, `fret-imui`
  composition tests, and public smoke tests use read accessors instead of direct field reads. This
  prepared the private-field hardening follow-up below. Gates: `cargo nextest run -p fret-ui-kit --features imui --test
  imui_table_smoke table_column_helpers_compile table_column_visibility_helpers_compile
  table_resizable_column_api_compiles table_sortable_header_api_compiles --no-fail-fast`, `cargo
  nextest run -p fret-ui-kit --features imui --lib
  visibility_state_applies_runtime_overrides_by_stable_column_id
  visibility_state_leaves_unlisted_and_unidentified_columns_at_declared_visibility
  horizontal_scroll_option_wraps_unpinned_header_and_body_center_groups --no-fail-fast`, `cargo
  nextest run -p fret-imui table_helper_pins_left_and_right_columns_while_center_columns_scroll
  table_column_visibility_menu_item_updates_visibility_state --no-fail-fast`, and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: completed the `TableColumn` private-field hardening follow-up. The struct is now
  builder/accessor-first instead of a public option-data bag; fields are private, render and
  visibility-policy internals keep crate-local `header_arc(...)`, `id_arc(...)`, and
  `set_visible_for_policy(...)` seams, and `tools/gate_imui_workstream_source.py` rejects the old
  public field shape from returning. Gates: `cargo nextest run -p fret-ui-kit --features imui
  --test imui_table_smoke table_column_helpers_compile table_column_visibility_helpers_compile
  table_resizable_column_api_compiles table_sortable_header_api_compiles --no-fail-fast`, `cargo
  nextest run -p fret-ui-kit --features imui --lib
  visibility_state_applies_runtime_overrides_by_stable_column_id
  visibility_state_leaves_unlisted_and_unidentified_columns_at_declared_visibility
  horizontal_scroll_option_wraps_unpinned_header_and_body_center_groups --no-fail-fast`, `cargo
  nextest run -p fret-imui table_helper_pins_left_and_right_columns_while_center_columns_scroll
  table_column_visibility_menu_item_updates_visibility_state --no-fail-fast`, and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: refreshed the current table-gap wording after the visibility snapshot,
  freeze-pane pinning, and `TableColumn` private-field slices landed. These are no longer current table gaps in this workstream.
  Future table work should only open app/editor storage/schema policy or concrete Dear ImGui
  table-runtime parity lanes with a new proof and gate. Gates:
  `python tools/gate_imui_workstream_source.py`, `python -m py_compile
  tools/gate_imui_workstream_source.py`, and `git diff --check`.
- 2026-05-16: introduced `text_control_readout(...)` as the shared compact control-readout text
  role. The UI Gallery code-editor toolbar keeps its doc-layout helper, but that helper now
  delegates to `fret-ui-kit::declarative::text::text_control_readout(...)`, so dense status/readout
  text no longer carries app-local wrap/overflow policy. Gate: `cargo nextest run -p fret-ui-kit
  --features imui --lib control_readout_text_uses_muted_compact_single_line_truncation
  --no-fail-fast`.
- 2026-05-16: introduced `text_button_label(...)` as the shared compact button-label text role and
  routed IMUI `control_text(...)` through it. IMUI buttons and pill-style control labels now keep
  single-line truncation instead of inheriting word-wrap text semantics. Gate: `cargo nextest run -p
  fret-ui-kit --features imui --lib button_label_text_uses_medium_single_line_truncation
  imui::control_chrome::tests::imui_control_text_uses_shared_button_label_role --no-fail-fast`.
- 2026-05-16: introduced `text_code_block(...)` as the shared code-block text role beside the
  existing inline/wrapping `text_code_wrap(...)`. The UI Gallery docs scaffold now uses the shared
  role for scrollable code blocks instead of constructing monospace `TextProps` locally. Gate:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  prose_variants_and_code_wrap_install_semantic_inherited_overrides --no-fail-fast`.
- 2026-05-16: introduced `text_paragraph(...)` and `text_paragraph_break_words(...)` as the stable
  semantic paragraph role names over the existing `text_prose(...)` helpers. This closes the first
  text-role vocabulary pass without breaking shadcn/Tailwind-oriented naming. Gate: `cargo nextest
  run -p fret-ui-kit --features imui --lib
  prose_variants_and_code_wrap_install_semantic_inherited_overrides --no-fail-fast`.
- 2026-05-17: added a shared text-role layout gate in
  `fret-ui-kit::declarative::text`. The gate uses a wrapping fake text service plus
  `UiTree::layout_all(...)` to prove the base single-line roles (`text_control_readout(...)`,
  `text_button_label(...)`, `text_table_cell(...)`, and `text_code_block(...)`) stay one measured
  line under narrow resize, while `text_paragraph(...)` measures as multiple lines. Gate:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  base_single_line_text_roles_stay_single_line_under_narrow_layout
  paragraph_text_role_measures_multiple_lines_under_narrow_layout --no-fail-fast`.
- 2026-05-17: introduced `text_compact_paragraph(...)` as the shared dense wrapping paragraph role
  for editor/IMUI body copy. IMUI `bullet_text(...)` labels and
  `UiWriterImUiFacadeExt::text_wrapped(...)` now route through it, preserving explicit wrapping
  while moving fill-width/min-width-zero layout policy out of local `TextProps`. Gate: `cargo
  nextest run -p fret-ui-kit --features imui --lib
  compact_paragraph_text_uses_wrapping_fill_width_layout
  bullet_text_uses_shared_compact_paragraph_role imui_text_wrapped_is_explicit_wrapping_text
  --no-fail-fast`.
- 2026-05-18: routed IMUI `tooltip_text(...)` / `tooltip_text_with_options(...)` body copy through
  a private `tooltip_body_text(...)` helper backed by `text_compact_paragraph(...)`. This keeps the
  convenience tooltip path on dense wrapping body/help text, while rich-content `tooltip(...)`
  closures remain caller-owned. Gate: `cargo nextest run -p fret-ui-kit --features imui --lib
  tooltip_body_text_uses_compact_paragraph_role --no-fail-fast`.
- 2026-05-16: routed IMUI tab triggers and menubar triggers through the shared
  `text_button_label(...)` role. This keeps button-like trigger labels single-line and truncating
  while leaving menu item/selectable row labels out of the button-label role. Gate: `cargo nextest
  run -p fret-ui-kit --features imui --lib
  imui::tab_family_controls::tests::tab_trigger_visual_uses_button_label_text_role
  imui::menu_family_controls::tests::menu_trigger_visual_uses_button_label_text_role
  --no-fail-fast`.
- 2026-05-16: introduced `text_list_row_label(...)` as the shared dense list/command-row label
  role and routed IMUI menu items, selectables, and disclosure/tree rows through it. The role uses
  regular `text-sm` styling with fill-width, `min-width: 0`, single-line ellipsis semantics so row
  labels truncate instead of wrapping or increasing row height under resize. Gate: `cargo nextest
  run -p fret-ui-kit --features imui --lib
  list_row_label_text_uses_fill_width_single_line_truncation
  menu_item_label_text_uses_shared_list_row_text_role
  selectable_row_label_uses_shared_list_row_text_role
  tree_row_label_uses_shared_list_row_text_role --no-fail-fast`.
- 2026-05-16: routed IMUI menu shortcut labels through the existing `text_control_readout(...)`
  role as muted compact auxiliary readouts. This keeps shortcut text single-line, shrinkable, and
  ellipsis-truncated without adding a menu-specific shortcut role. Gate: `cargo nextest run -p
  fret-ui-kit --features imui --lib menu_item_shortcut_text_uses_shared_control_readout_role
  menu_item_label_text_uses_shared_list_row_text_role --no-fail-fast`.
- 2026-05-17: routed IMUI menu checkbox/radio indicators and submenu chevrons through the shared
  `text_chrome_glyph(...)` role instead of bare `cx.text(...)`. Menu glyph chrome now follows the
  same single-line clip contract as disclosure indicators, and the source gate rejects the old
  indicator text paths. Gate: `cargo nextest run -p fret-ui-kit --features imui --lib
  menu_item_indicator_text_uses_shared_chrome_glyph_role --no-fail-fast`.
- 2026-05-17: introduced `text_section_chrome_label(...)` as the shared compact section/chrome
  label role and routed IMUI `separator_text` labels through it. Separator labels no longer carry
  local `TextProps` policy or default word wrapping; they stay single-line, shrinkable, and
  ellipsis-truncated under resize. Gate: `cargo nextest run -p fret-ui-kit --features imui --lib
  section_chrome_label_text_uses_single_line_truncation --no-fail-fast`.
- 2026-05-17: introduced `text_chrome_title(...)` as the shared medium, fill-width chrome title
  role and routed floating window title-bar text through shared chrome text helpers. Resizable
  floating titles keep fill, grow, shrink, `min-width: 0`, and ellipsis behavior; non-resizable
  titles reuse `text_section_chrome_label(...)` instead of local `TextProps`. Gate: `cargo nextest
  run -p fret-ui-kit --features imui --lib
  chrome_title_text_uses_medium_fill_width_single_line_truncation
  section_chrome_label_text_uses_single_line_truncation --no-fail-fast`.
- 2026-05-17: introduced `text_chrome_glyph(...)` as the shared compact fixed-slot chrome glyph
  role and routed disclosure/tree indicators through it. Indicator glyphs now stay single-line and
  clipped inside fixed chrome slots without owning local `TextProps` policy in disclosure controls.
  Gate: `cargo nextest run -p fret-ui-kit --features imui --lib
  chrome_glyph_text_uses_fixed_slot_single_line_clip
  disclosure_indicator_uses_shared_chrome_glyph_text_role --no-fail-fast`.
- 2026-05-17: routed `list_from_strings(...)` row text through the shared text-role vocabulary.
  Leading list glyphs now use `text_chrome_glyph(...)`, row labels use
  `text_list_row_label(...)`, and trailing shortcut/readout text uses
  `text_control_readout(...)`. This closes a generic fixed-row compatibility-helper path without
  adding an IMUI ListBox API or moving policy into `fret-imui`. Gates: `cargo nextest run -p
  fret-ui-kit --features imui --lib list_from_strings_uses_shared_single_line_text_roles
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: routed the default retained tree row renderer through the shared text-role
  vocabulary too. Tree labels now use `text_list_row_label(...)` and toggle glyphs use
  `text_chrome_glyph(...)` instead of the local `crate::ui::text(...).truncate()` / bare
  `cx.text(...)` path. Gates: `cargo nextest run -p fret-ui-kit --features imui --lib
  default_tree_row_label_uses_shared_list_row_text_role tree_toggle_glyph_uses_shared_chrome_glyph_text_role
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: routed floating-window close button glyphs through the same
  `text_chrome_glyph(...)` role via `floating_window_close_glyph_text(...)`. This keeps fixed
  title-bar action chrome on the shared single-line clip contract instead of bare `cx.text(...)`
  default wrapping semantics. Gate: `cargo nextest run -p fret-ui-kit --features imui --lib
  floating_window_close_glyph_uses_shared_chrome_glyph_text_role --no-fail-fast`.
- 2026-05-17: introduced `text_control_label(...)` as the shared compact control-label text role
  and routed `control_chrome::fill_text(...)` through it. Checkbox/radio/switch labels plus
  combo/slider captions keep their fill, grow, shrink, `min-width: 0`, and ellipsis behavior
  without owning local `TextProps` policy inside IMUI chrome. Gate: `cargo nextest run -p
  fret-ui-kit --features imui --lib control_label_text_uses_fill_width_single_line_truncation
  imui_fill_text_is_single_line_and_shrinkable imui_control_text_uses_shared_button_label_role
  --no-fail-fast`.
- 2026-05-18: hardened `tools/gate_imui_workstream_source.py` so direct `TextProps`
  construction under `fret-ui-kit::imui` fails the source gate unless it is moved into shared text
  roles with explicit evidence. The check now covers both `TextProps::new(...)` and
  `TextProps { ... }` struct literals, matching the editor allowlist scanner and closing the
  remaining source-gate bypass for local IMUI text policy.
- 2026-05-17: removed the last IMUI direct text constructor exception by routing
  `UiWriterImUiFacadeExt::text(...)` through the shared `text_section_chrome_label(...)` role.
  Gates: `cargo nextest run -p fret-ui-kit --features imui --lib
  imui_text_item_is_single_line_and_shrinkable imui_text_wrapped_is_explicit_wrapping_text
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: introduced `editor_input_value_text(...)` in `fret-ui-editor` input-group primitives
  and routed drag-value plus axis-drag-value scrub readouts through it. The helper keeps editor
  numeric value text fill-width, `min-width: 0`, shrinkable, single-line, and ellipsis-truncated
  while preserving editor-specific density and chrome policy outside `fret-imui`. Gates: `cargo
  nextest run -p fret-ui-editor editor_input_value_text_is_single_line_and_shrinkable --no-fail-fast`
  and `cargo nextest run -p fret-ui-editor drag_value axis_drag_value --no-fail-fast`.
- 2026-05-19: moved the remaining input-group text props constructors out of
  `primitives/input_group.rs` and into `primitives/readout.rs` as
  `editor_input_segment_text_props(...)`, `editor_input_value_text_props(...)`, and
  `editor_axis_marker_text_props(...)`. `input_group` now consumes editor text roles instead of
  owning local `TextProps` policy, and the direct editor `TextProps` allowlist no longer includes
  `input_group.rs`. Gates: `cargo nextest run -p fret-ui-editor
  editor_input_segment_text_keeps_fixed_segment_line_box
  editor_input_value_text_props_are_single_line_and_shrinkable
  editor_axis_marker_text_keeps_fixed_centered_line_box
  editor_input_value_text_is_single_line_and_shrinkable --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: moved `FieldStatusBadge` label text policy into
  `editor_status_badge_text_props(...)` in the editor readout primitive layer. The control keeps its
  compact centered badge label, single-line ellipsis, and palette behavior without owning local
  `TextProps` policy. Gate: `cargo nextest run -p fret-ui-editor
  editor_status_badge_text_uses_compact_single_line_readout_role
  error_badge_palette_keeps_short_visible_label loading_badge_palette_uses_short_label
  loading_badge_palette_stays_darker_than_editor_foreground --no-fail-fast`.
- 2026-05-17: introduced `editor_inline_error_text_props(...)` for compact single-line editor error
  readouts and routed both `ColorEdit` root errors and popup numeric errors through it. The role is
  destructive-color aware through caller-supplied color, fill-width, `min-width: 0`, and ellipsis
  truncated; wrapping validation prose remains a separate explicit control policy. Gate: `cargo
  nextest run -p fret-ui-editor editor_inline_error_text_is_single_line_and_shrinkable
  editor_preview_caption_text_is_single_line_and_shrinkable
  editor_tooltip_readout_text_is_single_line_and_shrinkable
  numeric_readout_formats_rgb_hsv_and_optional_alpha
  color_tooltip_lines_match_imgui_hex_rgb_hsv_preview_text --no-fail-fast`.
- 2026-05-17: introduced `editor_validation_message_text_props(...)` for editor validation prose
  that is allowed to wrap and grow height. `NumericInput` inline validation messages now use that
  explicit role instead of local `TextProps`, while `tools/gate_imui_workstream_source.py` freezes
  direct `TextProps` construction under `fret-ui-editor/src` to the editor primitive owners
  (`input_group`, `popup_list`, and `readout`). Gates: `cargo nextest run -p fret-ui-editor
  editor_validation_message_text_wraps_and_shrinks --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: introduced `editor_preview_caption_text_props(...)` and
  `editor_tooltip_readout_text_props(...)` for color side-preview captions and color tooltip
  numeric lines. These stay in the editor readout primitive layer instead of being folded into
  popup-list rows, preserving the semantic distinction while removing local `TextProps` policy from
  `color_edit/popup/preview.rs` and `color_edit/popup/tooltip.rs`.
- 2026-05-17: introduced shared transform-label text helpers in
  `ecosystem/fret-ui-editor/src/primitives/readout.rs` and routed `TransformEdit` section badges,
  section headings, and inline link/uniform checkbox labels through them. The control no longer
  owns local `TextProps` literals for compact labels, and those labels now consistently stay
  single-line, `min-width: 0` where they need to shrink, and ellipsis/clip constrained under
  resize. Gate: `cargo nextest run -p fret-ui-editor
  editor_section_badge_text_is_single_line_centered_badge_label
  editor_section_heading_text_is_single_line_and_shrinkable
  editor_inline_control_label_text_is_single_line_and_shrinkable
  transform_edit_axis_outcome_exposes_read_only_signals --no-fail-fast`.
- 2026-05-17: introduced `editor_popup_list_row_text_props(...)` and
  `editor_popup_empty_text_props(...)` in `ecosystem/fret-ui-editor/src/primitives/popup_list.rs`.
  `EnumSelect` trigger/row/empty text and `TextAssistField` row/empty text now use shared editor
  helpers instead of local `TextProps` or `TextProps::new(...)`, closing another default
  word-wrap path under resize. Color-edit copy menu rows and popup option captions now reuse the
  same popup-list family through start-aligned, centered-row, and fixed-caption variants while
  leaving preview labels and tooltip lines as separate semantics. Gate: `cargo nextest run -p
  fret-ui-editor
  popup_list_row_text_is_single_line_and_shrinkable
  popup_empty_text_is_single_line_and_shrinkable
  popup_list_centered_row_text_keeps_row_fill_and_center_alignment
  popup_list_option_caption_text_keeps_fixed_caption_line_box
  enum_select_item_test_id_segment_is_stable_ascii empty_label_is_inline_only
  color_copy_entries_match_imgui_copy_as_payloads
  popup_options_default_to_imgui_like_hue_bar_surface --no-fail-fast`.
- 2026-05-19: centralized the popup-list text props family into
  `ecosystem/fret-ui-editor/src/primitives/readout.rs`. `popup_list.rs` now owns only popup-list
  geometry, row state, and palette helpers; popup row, empty-state, centered-row, and fixed-caption
  text roles are part of the shared editor text role owner. The source gate now removes
  `popup_list.rs` from the direct editor `TextProps` allowlist and forbids `TextProps`,
  `TextWrap`, `TextOverflow`, and typography policy from returning there. Gate:
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: introduced `editor_property_group_header_text_props(...)`,
  introduced `editor_property_row_reset_glyph_text_props(...)`. This slice also
  introduced `editor_inspector_panel_title_text_props(...)` in
  `ecosystem/fret-ui-editor/src/primitives/readout.rs`, then routed `PropertyGroup` header labels,
  `PropertyRow` reset glyphs, and `InspectorPanel` titles through those shared roles. This removes
  local inspector chrome/default text policy from the composites while keeping fixed-row text
  single-line, shrinkable where needed, and line-height constrained under resize. The
  `InspectorPanel` layout gate renders a narrow header with toolbar siblings and proves the title
  remains one measured line. Gate:
  `cargo nextest run -p fret-ui-editor
  editor_property_group_header_text_is_single_line_and_shrinkable
  editor_property_row_reset_glyph_text_keeps_fixed_button_line_box
  editor_inspector_panel_title_text_is_single_line_and_shrinkable
  inspector_panel_title_stays_single_line_when_header_is_narrow --no-fail-fast`.
- 2026-05-17: introduced `editor_property_row_label_text_props(...)` and the property-grid row
  context `label_text(...)` convenience path for fixed inspector label chrome. `PropertyRow` label
  slots now clamp their own line box to the editor row height, so accidental bare/default label
  text cannot wrap and grow a fixed-height row under resize. `GradientEditor`, eager
  `PropertyGrid`, and virtualized grid smoke usage now route first-party labels through the helper,
  while arbitrary custom label elements remain possible for fully custom rows. Gate:
  `cargo nextest run -p fret-ui-editor
  editor_property_row_label_text_is_single_line_and_shrinkable
  row_label_slot_keeps_fixed_line_box_when_label_text_wraps_under_narrow_layout --no-fail-fast`.
- 2026-05-17: migrated the `imui_editor_proof_demo` property-grid labels to
  `row_cx.label_text(...)` so the selected editor-grade proof teaches the fixed-label role instead
  of relying on `PropertyRow`'s container fallback for bare `cx.text(...)`. The source gate now
  requires representative proof labels (`Name`, `Typed numeric`, `Blend slider`, `Transform`) to
  use `label_text(...)` and forbids those labels from returning to `|cx| cx.text(...)` in property
  label slots. Gates: `python tools/gate_imui_workstream_source.py` and
  `cargo check -p fret-demo --bin imui_editor_proof_demo`.
- 2026-05-19: extended the `imui_editor_proof_demo` text-role proof from property rows/readouts to
  the main IMUI proof chrome. The demo now uses `proof_imui_section_text(...)`,
  `proof_imui_readout_text(...)`, and `proof_imui_compact_paragraph_text(...)` backed by shared
  `text_section_chrome_label(...)`, `text_control_readout(...)`, and
  `text_compact_paragraph(...)` instead of `fret_ui_kit::ui::text(...).text_xs()` /
  `.font_semibold()` local styling. The focused source test and workstream source gate reject the
  old headline, hint, parity intro/state hint, and editor label paths. Gates:
  `cargo nextest run -p fret-examples --test imui_editor_proof_text_roles_surface
  imui_editor_proof_main_fixed_text_uses_shared_roles --no-fail-fast`,
  `cargo check -p fret-demo --bin imui_editor_proof_demo`, and
  `python tools/gate_imui_workstream_source.py`.
  Verification note: the first focused nextest attempt timed out while it contended with a parallel
  `cargo check` for Cargo package-cache locks; after the background Cargo/Rustc processes exited,
  the same focused nextest command passed. `cargo fmt --check -p fret-examples` and
  `python -m py_compile tools\gate_imui_workstream_source.py` also passed before the final gate
  run. `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast`,
  `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`, and
  `git diff --check` passed.
- 2026-05-17: migrated the `workspace_shell_demo` editor rail to teach the shared text roles too.
  Rail command buttons now use `text_button_label(...)`, property labels use `row_cx.label_text(...)`,
  and compact property values route through `text_control_readout(...)` via a local proof helper.
  Gates: `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface
  --no-fail-fast`, `cargo check -p fret-demo --bin workspace_shell_demo`, and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: migrated the remaining `workspace_shell_demo` editor-rail header copy to a local
  `workspace_shell_paragraph_text(...)` helper backed by shared `text_paragraph(...)` instead of
  app-local `fret_ui_kit::ui::text(...).text_sm().text_color(...)` styling. This keeps the shell
  proof's explanatory text in the shared paragraph role while leaving compact rail values on
  `workspace_shell_readout_text(...)`. Gates: `cargo nextest run -p fret-examples --test
  workspace_shell_editor_rail_surface
  workspace_shell_demo_composes_editor_rail_through_workspace_frame_slots --no-fail-fast`,
  `cargo check -p fret-demo --bin workspace_shell_demo`, `cargo nextest run -p fret-examples
  --test text_role_residual_surface remaining_bare_text_in_fret_examples_is_explicit_capability_surface
  --no-fail-fast`, and `python tools/gate_imui_workstream_source.py`.
  Verification note: the first focused nextest attempt timed out during background compile; after
  Cargo/Rustc exited, the same focused nextest command passed. `cargo fmt --check -p
  fret-examples` also passed.
- 2026-05-17: migrated the `editor_notes_demo` inspector metadata surface to the same resize-safe
  text roles. Property-grid labels now use `row_cx.label_text(...)`, inspector subtitle and compact
  committed/outcome/draft/summary status values route through a local
  `editor_notes_readout_text(...)` helper backed by `text_control_readout(...)`, and the
  `editor_notes_editor_rail_surface` test plus the IMUI workstream source gate reject those fixed
  property-row labels/readouts drifting back to bare `cx.text(...)`. Gates:
  `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --no-fail-fast`,
  `cargo check -p fret-demo --bin editor_notes_demo`, and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: extended the `editor_notes_demo` proof text-role migration to the center preview and
  collection summary surfaces. Collection summary/status values now use
  `editor_notes_readout_text(...)`, center section labels use `editor_notes_section_text(...)`, and
  app-owned explanatory/preview text uses `editor_notes_paragraph_text(...)` instead of local
  `ui::text(...).wrap(...)` styling. The inspector `PropertyGrid` and `TextField` behavior remain
  unchanged. Gates: `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface
  editor_notes_demo_composes_shell_mounted_rails_through_workspace_frame_slots --no-fail-fast`,
  `cargo check -p fret-demo --bin editor_notes_demo`, and `python
  tools/gate_imui_workstream_source.py`.
  Verification note: the first focused nextest attempt timed out while a background Cargo/Rustc
  compile continued; after Cargo/Rustc exited, the same focused nextest command passed. `cargo
  check -p fret-demo --bin editor_notes_demo`, `python tools/gate_imui_workstream_source.py`,
  `cargo fmt --check -p fret-examples`, `python -m json.tool
  docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json > $null`, and `git diff --check`
  passed.
- 2026-05-19: migrated the `editor_notes_device_shell_demo` compact mobile header to the same
  shared text-role vocabulary. The mobile title now uses `device_shell_section_text(...)` backed by
  `text_section_chrome_label(...)`, and the explanatory header copy uses
  `device_shell_paragraph_text(...)` backed by `text_paragraph(...)` instead of local
  `ui::text(...).text_base().font_semibold()` / `ui::text(...).text_sm().text_color(...).wrap(...)`
  styling. Gates: `cargo nextest run -p fret-examples --test editor_notes_device_shell_surface
  editor_notes_device_shell_demo_keeps_shell_switch_explicit_and_reuses_inner_editor_content
  --no-fail-fast`, `cargo check -p fret-demo --bin editor_notes_device_shell_demo`, `cargo nextest
  run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast`, and
  `python tools/gate_imui_workstream_source.py`.
  Verification note: the first focused nextest attempt timed out during background compile; after
  Cargo/Rustc exited, the same focused nextest command passed. `cargo fmt --check -p
  fret-examples` also passed.
- 2026-05-17: migrated the `workspace_shell_demo` dirty-close prompt title/details to the same
  role vocabulary. The title now uses `workspace_shell_section_chrome_label(...)` backed by
  `text_section_chrome_label(...)`, while reason/dirty-detail lines use
  `workspace_shell_readout_text(...)` backed by `text_control_readout(...)`. Gates:
  `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --no-fail-fast`,
  `cargo check -p fret-demo --bin workspace_shell_demo`, and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: migrated selected `imui_editor_proof_demo` proof text to shared roles without
  widening framework API. Material/advanced/global `No matches` labels now use
  `proof_empty_state_text(...)`, authoring shared-state lines use `proof_compact_readout_element(...)`
  backed by `text_control_readout(...)`, and the declarative gradient section label uses
  `proof_section_chrome_label(...)` backed by `text_section_chrome_label(...)`. Gates:
  `python tools/gate_imui_workstream_source.py` and
  `cargo check -p fret-demo --bin imui_editor_proof_demo`.
- 2026-05-18: migrated the `imui_editor_proof_demo` collection proof fixed text to the same shared
  role vocabulary without adding shared collection policy. Collection title text uses a
  proof-local section-chrome helper, compact state/status/readout lines use a proof-local helper
  backed by `text_control_readout(...)`, and inline rename explanatory copy opts into
  `text_wrapped(...)`. Focused source tests and the IMUI source gate reject those fixed collection
  paths drifting back to bare `ui.text(...)`.
- 2026-05-18: moved the UI Gallery app-facing DataTable snippets onto the shared table-cell text
  role. `apps/fret-ui-gallery/src/ui/snippets/data_table/mod.rs` now owns a narrow
  `table_cell_text(...)` helper over `text_table_cell(...)`, and the default/basic/guide/RTL/reusable
  snippets use it for fixed body cells plus fallback cells instead of bare `cx.text(...)`. Amount
  columns remain on the existing tabular numeric formatting path pending a separate numeric-cell
  role. Gates: `cargo nextest run -p fret-ui-gallery --test data_table_docs_surface
  data_table_snippets_keep_fixed_cell_text_on_table_role --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: removed empty text nodes from copyable AI AudioPlayer state markers. Local and remote
  snippets now use a `state_marker(...)` helper that mounts a zero-size `SpacerProps` child under
  generic semantics with the same diagnostics `test_id`s. This keeps non-visible test anchors out
  of text layout semantics. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_audio_player_text_role_surface audio_player_state_markers_use_non_text_spacers
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: migrated visible fixed text in copyable AI Message and Terminal snippets onto the
  shared text-role vocabulary. Message uses `text_control_readout(...)` for the compact action
  status, `text_section_chrome_label(...)` for the fixed demo title, and `text_paragraph(...)` for
  user-message prose. Terminal uses section-chrome title text, paragraph explanatory copy, and the
  non-text spacer-marker pattern for its empty-output diagnostics marker. Gates: `cargo nextest run
  -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: moved the actual `fret-ui-ai` `TerminalTitle` default label from local
  `ui::raw_text(...).wrap(None).overflow(Clip)` policy to the shared `text_chrome_title(...)`
  role. The component now keeps terminal chrome titles fill-width, `min-width: 0`, grow/shrink
  enabled, single-line, and ellipsized under narrow resize. Gate: `cargo nextest run -p
  fret-ui-ai terminal_title_label_uses_chrome_title_text_role --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: tightened `text_chrome_title(...)` to own the medium-weight chrome-title contract as
  well as fill/grow/shrink/min-width-zero/ellipsis layout, then routed the real `fret-ui-ai`
  `EnvironmentVariablesTitle` default/text paths through that shared role instead of local
  `ui::raw_text(...).wrap(None).overflow(Clip)` typography policy. Custom title children still use
  the component-owned inherited title refinement because the upstream surface is children-first.
  Gates: `cargo nextest run -p fret-ui-kit --lib
  chrome_title_text_uses_medium_fill_width_single_line_truncation
  section_chrome_label_text_uses_single_line_truncation --no-fail-fast`, `cargo nextest run -p
  fret-ui-ai environment_variables_title_text_uses_chrome_title_text_role
  environment_variables_title_children_scope_inherited_typography
  terminal_title_label_uses_chrome_title_text_role --no-fail-fast`, and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: routed `fret-ui-ai` `EnvironmentVariableName` and non-selectable
  `EnvironmentVariableValue` text through `text_code_label(...)` with component-owned foreground
  inheritance instead of local monospace `TextProps`. Revealed values intentionally stay on
  `SelectableTextProps` because that path owns text selection, not ordinary fixed chrome text. The
  environment-variable empty/custom-child marker paths now use the crate-local non-text spacer
  placeholder instead of empty `Text` nodes. Gates: `cargo nextest run -p fret-ui-ai
  environment_variable_name_and_masked_value_use_code_label_text_role
  environment_variable_value_is_selectable_only_when_shown
  environment_variable_copy_button_supports_custom_children --no-fail-fast`, `cargo check -p
  fret-ui-ai`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: added the shared `text_code_label_emphasis(...)` and
  `text_compact_paragraph_inherited(...)` text-role derivatives, then routed real `fret-ui-ai`
  `PackageInfo` defaults through them. Package names and target versions now use emphasized
  code-label text, current versions and dependency rows use regular code-label text, the
  Dependencies heading uses section-chrome text, and `PackageInfoDescription` keeps the shared
  wrapping/fill-width paragraph layout without overriding the component-owned description
  typography scope. Gates: `cargo nextest run -p fret-ui-kit --features imui --lib
  prose_variants_and_code_wrap_install_semantic_inherited_overrides
  compact_paragraph_text_uses_wrapping_fill_width_layout
  inherited_compact_paragraph_keeps_wrapping_layout_without_leaf_refinement --no-fail-fast`,
  `cargo nextest run -p fret-ui-ai package_info_default_identifier_text_uses_code_label_role
  package_info_description_scopes_inherited_description_typography
  package_info_root_children_support_docs_shaped_compound_parts --no-fail-fast`,
  `cargo check -p fret-ui-ai`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the same visible text-role migration to AI Artifact, CodeBlock, and Sandbox
  snippets. Fixed demo titles and panel labels now use section-chrome text, explanatory copy uses
  paragraph text, Artifact's closed state uses a compact control readout, and CodeBlock's
  active-language diagnostics marker now uses a zero-size generic spacer marker rather than an
  invisible empty `Text`. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to the Queue copyable snippet. The fixed
  demo title now uses `text_section_chrome_label(...)`, explanatory copy uses
  `text_paragraph(...)`, and the action-revision diagnostics anchor uses a generic zero-size
  spacer marker instead of an empty/invisible `Text` node. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to the Checkpoint copyable snippet.
  Conversation and explanatory copy now use `text_paragraph(...)`, restore status uses
  `text_control_readout(...)`, the checkpoint trigger uses `text_button_label(...)`, and custom
  checkpoint icon symbols use `text_chrome_glyph(...)`. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to simple chrome/prose snippets:
  Agent, CodeBlock usage, Environment Variables, and OpenIn. Each now routes fixed demo titles
  through `text_section_chrome_label(...)` and explanatory body copy through `text_paragraph(...)`
  instead of default bare `cx.text(...)`. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to selector/branch state markers.
  MessageBranch, MicSelector, and ModelSelector now use generic zero-size `SpacerProps` markers for
  diagnostics-only state anchors instead of empty `Text`, and their fixed demo title/body copy uses
  shared section-chrome/paragraph roles. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to CommitLarge, Plan,
  PromptInputActionMenu, and PromptInputTooltip. CommitLarge now uses a generic zero-size
  `SpacerProps` marker for its opened-file diagnostics anchor instead of an empty `Text`, and the
  four snippets route their outer fixed demo title/body copy through shared
  section-chrome/paragraph roles. Inner Plan prose/Button composition stays out of this slice until
  a separate semantics pass. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: extended the AI visible text-role migration to PlanContent internals. Plan section
  headings now use `text_section_chrome_label(...)`, the overview body uses `text_paragraph(...)`,
  bullet rows use `text_list_row_label(...)`, and the custom Build button child uses
  `text_button_label(...)` instead of local `ui::text(...)` styling. Plan open/streaming behavior
  and `fret-ui-ai` Plan component ownership were intentionally left unchanged. Gates: `cargo
  nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_prompt_and_plan_snippets_use_shared_outer_text_roles --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to large/status snippets:
  StackTraceLarge, TestResultsLarge, Tool, and Suggestions. StackTraceLarge/TestResultsLarge
  opened/activated markers and Tool/Suggestions test markers now use generic zero-size
  `SpacerProps` anchors instead of empty `Text`; fixed outer title/body copy routes through shared
  section-chrome/paragraph roles, and Tool's fixed state-section labels use section-chrome text.
  Suggestions custom-children content stays app-owned for a later custom-content semantics pass.
  Gates: `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to QueuePromptInput and Transcription.
  QueuePromptInput now uses a generic zero-size `SpacerProps` anchor for the sent-count diagnostics
  marker, routes the custom Search button child through `text_button_label(...)`, and keeps fixed
  outer title/body copy on shared section-chrome/paragraph roles. Transcription now uses generic
  zero-size spacer markers for time/active diagnostics anchors while routing fixed title/body copy
  through shared roles. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to WebPreview. State diagnostics markers
  now use generic zero-size `SpacerProps` anchors instead of empty `Text`, navigation arrow/reload
  glyphs use `text_chrome_glyph(...)`, and composable child fixed body/footer copy routes through
  shared section-chrome/paragraph roles. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Chat's outer surface. The
  prompt-nonempty diagnostics marker now uses a generic zero-size spacer, empty marker fallbacks use
  spacers instead of empty `Text`, fixed header instructions route through paragraph roles, and the
  exported markdown length readout uses `text_control_readout(...)`. Chat message body rendering
  remains app/content-owned for a separate semantics pass. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to PromptInputProvider and PromptInput
  docs. Provider sent-count diagnostics now use a generic zero-size `SpacerProps` anchor, the
  custom external-add label uses `text_button_label(...)`, and fixed outer title/body copy uses
  shared roles. PromptInput docs now routes the custom Search label through button-label text and
  fixed outer title/body copy through section-chrome/paragraph roles. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-19: extended the AI visible text-role migration to Cursor-style PromptInput custom text.
  Command item titles now use `text_list_row_label(...)`, filenames/paths use
  `text_code_label(...)`, rules/tabs hover-card readouts use `text_control_readout(...)`, and
  custom trigger counts use `text_button_label(...)` instead of local `ui::text(...)` styling.
  PromptInput placeholders, command headings, and component-owned labels were intentionally left to
  `fret-ui-ai` component policy. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface ai_prompt_input_cursor_custom_text_uses_shared_roles
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: extended the AI visible text-role migration to Shimmer demo chrome. Typography,
  duration, and elements demo labels now use `text_control_readout(...)`; the inline non-shimmer
  prefix/suffix in the elements demo uses section-chrome/control-readout roles instead of bare
  `ui::text(...)`. `Shimmer::new(...)` calls intentionally remain because Shimmer itself is the
  explicit animated text capability surface. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface ai_shimmer_demo_chrome_text_uses_shared_roles --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Reasoning, StackTrace, and
  VoiceSelector fixed chrome/readouts. Fixed outer title/body copy uses shared
  section-chrome/paragraph roles; StackTrace status and VoiceSelector selected/open diagnostics use
  `text_control_readout(...)` instead of default wrapping text. Content renderers remain owned by
  their respective AI element surfaces. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Confirmation content snippets.
  Confirmation request prose now uses `text_paragraph(...)`, inline/code payloads use
  `text_code_wrap(...)`, approval/rejection result text uses `text_control_readout(...)`, and the
  demo's fixed outer title/body copy uses shared section-chrome/paragraph roles. Confirmation state
  and button policy stay in the AI element/recipe layer. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Task content. Task item labels now
  use `text_list_row_label(...)`, attached file names use `text_code_wrap(...)`, and the demo's
  fixed outer title/body copy uses shared section-chrome/paragraph roles. Gates: `cargo nextest run
  -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Conversation instrumentation and
  custom scroll-button text. Export/message-count diagnostics now use generic semantics with
  numeric values instead of text semantics, while the custom `Latest` scroll-bottom label uses
  `text_button_label(...)`. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to fixed usage snippet chrome/prose.
  Attachments usage explanatory copy now uses `text_paragraph(...)`, and StackTrace usage fixed
  title/body copy uses `text_section_chrome_label(...)` / `text_paragraph(...)`. Gates: `cargo
  nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Message usage. User message text now
  uses `text_paragraph(...)`, the last-action marker uses `text_control_readout(...)`, and fixed
  outer title/body copy uses `text_section_chrome_label(...)` / `text_paragraph(...)`; assistant
  markdown response rendering remains owned by `MessageResponse`. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to the Canvas world spike. Visible
  chrome, node copy, and debug/status readouts now use `text_section_chrome_label(...)`,
  `text_paragraph(...)`, and `text_control_readout(...)` instead of bare `cx.text(...)`. Canvas
  pan/zoom, bounds, drag/drop, and connection behavior were intentionally left unchanged. Gates:
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to the Image demo. The fixed explanatory
  copy now uses `text_paragraph(...)`, and the image-ready/loading readouts use
  `text_control_readout(...)` instead of bare `cx.text(...)`. Image asset lookup and presentation
  behavior were intentionally left unchanged. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to PromptInput referenced sources. The
  fixed title/body copy now uses `text_section_chrome_label(...)` and `text_paragraph(...)` instead
  of bare `cx.text(...)`; source chip state and PromptInput model behavior were intentionally left
  unchanged. Gates: `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: extended the AI visible text-role migration to Attachments inline hover-card details.
  App-owned attachment labels now use `text_list_row_label(...)`, and media-type values use
  `text_control_readout(...)` instead of default `ui::text(...)` builders; attachment chip,
  remove affordance, and hover-card behavior were intentionally left unchanged. Gates: `cargo
  nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_attachments_inline_hover_card_uses_shared_text_roles --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: migrated the AI Artifact docs code-display status marker from an invisible
  `cx.text(...)` node to a generic zero-size semantics marker with a diagnostic label. This keeps
  the existing `ui-gallery-ai-artifact-docs-run-action` `label_contains` script contract while
  removing the hidden text from text layout. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast`, `python tools/gate_imui_workstream_source.py`, and
  `python -m json.tool tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-artifact-docs-run-action.json`.
- 2026-05-18: `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  --no-fail-fast` passed after the large/status snippet slice landed. `python
  tools/gate_imui_workstream_source.py`, `python -m py_compile
  tools/gate_imui_workstream_source.py`, `python -m json.tool
  docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json`, and `git diff --check` also passed.
- 2026-05-18: `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  --no-fail-fast` passed after the Artifact/CodeBlock/Sandbox slice landed. `python
  tools/gate_imui_workstream_source.py`, `python -m json.tool
  docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json`, and `git diff --check` also passed.
- 2026-05-17: migrated `imui_editor_proof_demo` drag-preview cards away from a newline-joined bare
  text blob. Preview titles now use `text_section_chrome_label(...)`, optional subtitles use
  `text_control_readout(...)`, and `proof_drag_preview_card_uses_single_line_text_roles` locks the
  two-line preview as two single-line role elements. Gates: `cargo nextest run -p fret-examples
  --lib proof_drag_preview_card_uses_single_line_text_roles --no-fail-fast`, `cargo check -p
  fret-demo --bin imui_editor_proof_demo`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: introduced `editor_empty_state_text_props(...)` for compact editor empty-state
  labels and routed `GradientEditor`'s `No stops` label through
  `gradient_editor_empty_state_text(...)`. This removes another real component bare-text path while
  keeping the empty-state role in `fret-ui-editor`, not `fret-imui`. Gates: `cargo nextest run -p
  fret-ui-editor editor_empty_state_text_is_single_line_and_shrinkable
  gradient_editor_empty_state_text_is_single_line_and_shrinkable --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: removed clipping from `PropertyRow` value slots while keeping fixed label/reset/action
  chrome clipped. This is the layout-container side of the text-role contract: wrapping validation
  prose such as `NumericInput` inline errors may grow to multiple lines, so the parent value slot
  must not clip the measured line box under resize. Gates:
  `cargo nextest run -p fret-ui-editor row_value_slot_keeps_overflow_visible_for_wrapping_value_children row_value_slot_grows_to_wrapping_value_text_under_narrow_layout --no-fail-fast`
  and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: added a layout-level `PropertyRow` resize regression for wrapping validation text.
  The test renders a narrow row with `editor_validation_message_text_props(...)`, runs the real
  `UiTree::layout_all(...)` path through public element-bounds queries, and asserts that the
  measured multi-line text bottom stays inside the value slot and row bounds. This closes the
  evidence gap between the structural overflow contract and the visual resize bug report.
- 2026-05-17: added a composition-level `PropertyGrid` resize regression for the same wrapping
  validation role. The test renders mixed single-line and wrapping rows in a narrow grid, reuses the
  same measured wrapping text service, and asserts that the wrapping row grows, pushes the
  following row down, and remains contained by the grid. Gate:
  `cargo nextest run -p fret-ui-editor property_grid_keeps_rows_separated_when_value_text_wraps_under_narrow_layout --no-fail-fast`.
- 2026-05-17: added `P3_TEXT_ROLE_MATRIX_2026-05-17.md` as the resize triage contract for the
  current text-role work. It names the stable base roles (`text_control_readout(...)`,
  `text_button_label(...)`, `text_paragraph(...)` / `text_paragraph_break_words(...)`,
  `text_code_block(...)` / `text_code_wrap(...)`, and `text_table_cell(...)`), maps current derived
  roles, requires wrapping paragraph/validation copy to have parent layout that accounts for
  multi-line height, and keeps `fret-imui` policy-light by rejecting a public `TextRole` enum until
  two consumers need a data-driven role value. Gate: `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: UI Gallery's disabled toaster driver path now returns a spacer placeholder instead
  of an empty text node. This keeps app-shell placeholder plumbing out of the text layout/measure
  contract and prevents future resize fixes from treating invisible placeholders as text content.
  Gate: `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  gallery_driver_disabled_toaster_does_not_emit_empty_text --no-fail-fast`.
- 2026-05-17: the UI Gallery app-sidebar snippet collapsed-projects path now returns a spacer
  placeholder instead of an empty text node. This keeps the copyable sidebar recipe from teaching
  empty text as layout placeholder plumbing. Gate: `cargo nextest run -p fret-ui-gallery --test
  ui_authoring_surface_default_app sidebar_app_collapsed_projects_do_not_emit_empty_text
  --no-fail-fast`.
- 2026-05-17: `fret-ui-ai` now owns a crate-local `empty_placeholder(...)` helper for hidden or
  missing-content AI element paths. The affected AI surfaces return spacer placeholders instead of
  empty text nodes, keeping optional AI chrome out of text layout semantics without widening
  `fret-imui`. Gate: `cargo nextest run -p fret-ui-ai
  hidden_ai_element_paths_use_non_text_placeholder --no-fail-fast`.
- 2026-05-17: UI Gallery's retained-table torture page now routes fixed row/cell text through a
  local helper backed by `fret-ui-kit::declarative::text::text_table_cell(...)`, and table state
  readouts through `doc_layout::control_readout_text(...)`. The page keeps explicit prose/diagnostic
  description copy separate, while the fixed 28px table rows no longer use bare/default text. Gate:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_table_retained_torture_uses_structured_table_debug_ids --no-fail-fast`.
- 2026-05-17: UI Gallery's DataTable torture page now follows the same role split in both retained
  and non-retained render paths. Fixed cells route through a helper backed by
  `text_table_cell(...)`, while sorting/filter/pinning status lines use
  `doc_layout::control_readout_text(...)`. Gate: `cargo nextest run -p fret-ui-gallery --test
  ui_authoring_surface_internal_previews gallery_data_table_torture_exposes_header_row_anchor
  --no-fail-fast`.
- 2026-05-17: UI Gallery's DataGrid preview now routes virtualized grid cell text through a helper
  backed by `text_table_cell(...)`, and the selected-row status line through
  `doc_layout::control_readout_text(...)`. Gate: `cargo nextest run -p fret-ui-gallery --test
  ui_authoring_surface_internal_previews gallery_data_grid_uses_table_cell_text_roles
  --no-fail-fast`.
- 2026-05-17: UI Gallery's DataGrid/DataTable/Tree Torture explanatory header copy now routes
  through `doc_layout::paragraph_text(...)`, backed by shared `text_paragraph(...)`, instead of
  default `cx.text(...)`. Gates: `cargo nextest run -p fret-ui-gallery --test
  ui_authoring_surface_internal_previews gallery_data_grid_uses_table_cell_text_roles
  gallery_data_table_torture_exposes_header_row_anchor
  gallery_tree_torture_uses_control_readout_for_status_text --no-fail-fast`.
- 2026-05-17: UI Gallery's Inspector Torture preview now routes fixed virtual-row property labels
  through a helper backed by `text_list_row_label(...)`, and fixed row values through
  `doc_layout::control_readout_text(...)`. Gate: `cargo nextest run -p fret-ui-gallery --test
  ui_authoring_surface_internal_previews gallery_inspector_torture_uses_fixed_row_text_roles
  --no-fail-fast`.
- 2026-05-17: UI Gallery's virtual-list torture harness now routes fixed custom row labels through
  helpers backed by `text_list_row_label(...)`, and row detail/editing readouts through
  `doc_layout::control_readout_text(...)`. The UI Kit list torture custom row renderer also routes
  item labels through the shared list-row label role. Gate: `cargo nextest run -p fret-ui-gallery
  --test ui_authoring_surface_internal_previews
  harness_virtual_list_torture_uses_fixed_row_text_roles
  harness_ui_kit_list_torture_uses_fixed_row_text_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's retained-table, hit-test, UI Kit list, virtual-list, and view-cache
  harness headers now route explanatory copy through `doc_layout::paragraph_text(...)` and
  mode/status lines through `doc_layout::control_readout_text(...)` instead of bare `cx.text(...)`.
  Gates: `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_table_retained_torture_uses_structured_table_debug_ids
  harness_hit_test_torture_uses_header_text_roles
  harness_virtual_list_torture_uses_fixed_row_text_roles
  harness_ui_kit_list_torture_uses_fixed_row_text_roles
  harness_view_cache_uses_fixed_row_text_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's View Cache torture page now routes cached inner virtual-list row labels
  through a helper backed by `text_list_row_label(...)`. Gate: `cargo nextest run -p
  fret-ui-gallery --test ui_authoring_surface_internal_previews
  harness_view_cache_uses_fixed_row_text_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's View Cache torture page now routes fixed switch labels through
  `doc_layout::control_label_text(...)` instead of bare `cx.text(...)`. Gate: `cargo nextest run
  -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  harness_view_cache_uses_fixed_row_text_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's View Cache cached Popover body copy now routes through
  `doc_layout::paragraph_text(...)` instead of bare `cx.text(...)`. Gate: `cargo nextest run -p
  fret-ui-gallery --test ui_authoring_surface_internal_previews
  harness_view_cache_uses_fixed_row_text_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's Tree Torture dynamic target status now routes through
  `doc_layout::control_readout_text(...)` instead of local muted/text-sm styling. Gate: `cargo
  nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_tree_torture_uses_control_readout_for_status_text --no-fail-fast`.
- 2026-05-17: UI Gallery's overlay and menu last-action/status flags now route through
  `doc_layout::control_readout_text(...)` instead of bare `cx.text(...)`. Gate: `cargo nextest run
  -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_overlay_status_text_uses_control_readout_roles
  gallery_menus_last_action_uses_control_readout_role --no-fail-fast`.
- 2026-05-17: UI Gallery's overlay dialog/sheet/portal scroll filler rows now route through a
  helper backed by `text_list_row_label(...)` instead of bare `cx.text(...)`. Gate: `cargo nextest
  run -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_overlay_scroll_rows_use_list_row_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's HoverCard and Popover body copy now route through
  `doc_layout::paragraph_text(...)` instead of bare `cx.text(...)`. Gate: `cargo nextest run -p
  fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_overlay_body_copy_uses_paragraph_roles --no-fail-fast`.
- 2026-05-17: UI Gallery nav title and settings-sheet switch captions now route through the
  driver text-role owner. The nav title uses section-chrome text, and switch captions use
  control-label text instead of local `TextProps` policy. Gate: `cargo nextest run -p
  fret-ui-gallery --test code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-17: UI Gallery's chrome torture text-input/textarea labels now route through
  `doc_layout::control_label_text(...)`, backed by shared
  `fret-ui-kit::declarative::text::text_control_label(...)`, instead of bare `cx.text(...)`. Gate:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  page_chrome_torture_uses_control_label_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's status bar now routes metric, inspector-state, and last-action text
  through `driver::text_roles::chrome_readout_text(...)`, backed by
  `fret-ui-kit::declarative::text::text_control_readout(...)`. This keeps fixed status chrome on
  the shared single-line/shrinkable readout role instead of bare/default text under resize. Gate:
  `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-17: UI Gallery driver chrome now owns `driver::text_roles` as the local adapter over
  shared kit text roles. Disabled tabs/sidebar/content placeholders route through
  `chrome_readout_text(...)`, and settings-sheet section labels route through
  `chrome_section_label(...)`, keeping fixed app-shell chrome on single-line role helpers instead
  of bare/default text. Gate: `cargo nextest run -p fret-ui-gallery --test
  code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-17: UI Gallery's `BISECT_MINIMAL_ROOT` diagnostic root now routes its placeholder through
  `driver::text_roles::chrome_readout_text(...)` instead of bare `cx.text(...)`. This keeps the
  smallest resize/debug root on the same single-line readout role as the surrounding driver chrome.
  Gate: `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-17: UI Gallery's fixed-size debug HUD now renders each metric line through
  `driver::text_roles::chrome_readout_text(...)` instead of local word-wrapping `TextProps`. Long
  debug/readout lines now stay single-line and truncate inside the HUD chrome rather than growing
  line boxes under resize. Gate: `cargo nextest run -p fret-ui-gallery --test
  code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-17: UI Gallery shell content/nav text now routes page titles through
  `text_chrome_title(...)`, page origins through `text_control_readout(...)`, and sidebar group
  headings through `text_section_chrome_label(...)`. This removes local `TextProps` policy from
  fixed app-shell chrome without widening `fret-imui`. Gate: `cargo nextest run -p
  fret-ui-gallery --test code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-16: tightened `UiWriterImUiFacadeExt::text(...)` to match Dear ImGui's default
  `Text()` posture: single-line, shrinkable, `min-width: 0`, and ellipsis-truncated under resize.
  Added `UiWriterImUiFacadeExt::text_wrapped(...)` as the explicit wrapping path for explanatory
  prose, and routed first-party editor/workspace proof prose through it. Gates: `cargo nextest run
  -p fret-ui-kit --features imui --lib imui_text_item_is_single_line_and_shrinkable
  imui_text_wrapped_is_explicit_wrapping_text --no-fail-fast` and `cargo check -p fret-examples`.
- 2026-05-17: `UiWriterImUiFacadeExt::text(...)` now delegates to
  `text_section_chrome_label(...)`, removing the former local `TextProps` construction while
  keeping the same single-line resize contract. Gate: `cargo nextest run -p fret-ui-kit
  --features imui --lib imui_text_item_is_single_line_and_shrinkable
  imui_text_wrapped_is_explicit_wrapping_text --no-fail-fast`.
- 2026-05-16: tightened `control_chrome::fill_text(...)`, the shared path for boolean labels,
  combo preview/captions, and slider captions, to fill, shrink, `min-width: 0`, and truncate instead
  of word-wrapping inside compact control chrome. Gates: `cargo nextest run -p fret-ui-kit
  --features imui --lib imui_fill_text_is_single_line_and_shrinkable
  imui_control_text_uses_shared_button_label_role --no-fail-fast`, `cargo nextest run -p
  fret-ui-kit --features imui --test imui_button_smoke --test imui_combo_smoke --no-fail-fast`, and
  `cargo nextest run -p fret-ui-kit --features imui --lib
  input_text_model_uses_compact_imui_chrome_without_focus_ring
  textarea_model_uses_compact_imui_chrome_without_focus_ring --no-fail-fast`.
- 2026-05-17: `control_chrome::fill_text(...)` now delegates to the shared
  `text_control_label(...)` role instead of keeping that layout policy local to IMUI chrome.
- 2026-05-14: made `DisclosureResponse` / `ComboResponse` accessor-first for trigger/open/toggle
  state too. Public callers now read trigger details through `response()` and semantic helpers, the
  response types no longer expose external `Default` construction, and
  `tools/gate_imui_workstream_source.py` rejects public fields from returning.
- 2026-05-14: made `InputTextPickerResponse` accessor-first for input/open/pick state. Text picker
  tests now use `picked()` / `picked_index()` and the source gate rejects public picker response
  fields and public default construction from returning.
- 2026-05-14: made `TabBarResponse` / `TabTriggerResponse` accessor-first for selection and trigger
  state. Existing tab tests already use `selected_id()`, `selected_changed()`, `trigger(...)`, and
  trigger edge helpers; the source gate now rejects public tab response fields and public
  `TabBarResponse` default construction from returning.
- 2026-05-14: made `VirtualListResponse` accessor-first too. Public callers keep using `handle()`
  and `rendered_range()`, while the response's scroll handle and rendered-range storage are
  crate-local and guarded by the IMUI source gate.
- 2026-05-14: made `TableResponse` / `TableHeaderResponse` /
  `TableColumnResizeResponse` accessor-first for header metadata, sort state, resize bounds, and
  drag state. Public table tests now read `column_index()`, `sortable()`, `sort_direction()`,
  `resize()`, `min_width()`, and `max_width()` instead of response fields; the source gate rejects
  public table response fields and default construction from returning.
- 2026-05-14: made `DragResponse` accessor-first for started/dragging/stopped/delta/total state.
  Internal response assemblers can still populate drag state, while external callers stay on
  `started()`, `dragging()`, `stopped()`, `delta()`, and `total()` or the higher-level
  `ResponseExt` drag helpers.
- 2026-05-14: made `DragSourceResponse` / `DropTargetResponse` accessor-first too. Active source
  construction and empty target construction are crate-local, smoke tests now validate accessor
  usage through helper-returned responses, and the source gate rejects public drag/drop response
  fields or default construction from returning.
- 2026-05-14: made `ResponseExt` aggregate drag state accessor-first too. Public callers now read
  the aggregate response through `drag()`, `drag_started()`, `dragging()`, `drag_stopped()`,
  `drag_delta()`, and `drag_total()`, while `populate_pressable_drag_response(...)` and disabled
  sanitization use crate-local mutators. `tools/gate_imui_workstream_source.py` rejects a public
  `ResponseExt.drag` field or direct `ResponseExt` drag field reads from returning.
- 2026-05-14: made `ResponseExt` press/context-menu derived signal storage private too. Public
  callers keep using `secondary_clicked()`, `double_clicked()`, `long_pressed()`,
  `press_holding()`, `context_menu_requested()`, `context_menu_anchor()`, `pointer_clicked()`, and
  `pointer_click_modifiers()`, while `item_behavior`, disclosure headers, and disabled sanitization
  write through crate-local setters/clear helpers. `tools/gate_imui_workstream_source.py` rejects
  public fields or direct runtime writes from returning.
- 2026-05-14: made `ResponseExt` lifecycle edge storage private too. Public callers keep using
  `activated()`, `deactivated()`, `edited()`, and `deactivated_after_edit()`, while lifecycle
  runtime assembly plus combo/text-picker edit merging use crate-local set/merge helpers.
  `tools/gate_imui_workstream_source.py` rejects public lifecycle fields or direct runtime writes
  from returning.
- 2026-05-14: made `ResponseExt` raw hover, hover-delay, active-item block, and nav-highlight
  storage private too. Public callers and tests use `pointer_hovered_raw()`,
  `pointer_hovered_raw_below_barrier()`, the hover-delay accessors, `hover_blocked_by_active_item()`,
  and `nav_highlighted()`, while pressable/disclosure response assembly uses crate-local setters.
  Disabled sanitization still clears only nav highlight so `ALLOW_WHEN_DISABLED` raw-hover queries
  keep working.
- 2026-05-14: made `ResponseExt.enabled` storage private too. Public/demo/test callers use
  `enabled()`, while disabled sanitization and text controls use crate-local `set_enabled(...)`.
- 2026-05-14: made `ResponseExt.id` storage private too. Public/demo/test callers use `id()`, while
  pressable, disclosure, and text-control response assemblers use crate-local `set_id(...)`.
  Routing consumers in combo, menu/popup, tooltip, drag/drop, tab focus, text-picker, facade
  focusable recording, and the editor proof now consume identity through accessors. `ResponseExt.core`
  intentionally remains public in this slice because it is the broader shared
  `fret_authoring::Response` compatibility surface and needs a separate contract audit before
  accessor-only migration. The same compile sweep also migrated stale `apps/fret-examples-imui`
  demo reads for previously sealed floating/disclosure/combo/table response fields back onto their
  accessor APIs.
- 2026-05-14: made `ResponseExt.core` storage private too after the separate contract audit. Public
  callers keep the shared authoring-response bridge through `core()` and `from_core(...)`, plus
  focused signal accessors such as `rect()`, `focused()`, `hovered()`, and `pressed()`. Runtime
  pressable/disclosure/text/disabled/combo/multi-select/text-picker assembly writes through
  crate-local core setters and merge helpers, and `tools/gate_imui_workstream_source.py` rejects
  public `core` field access or direct `.core` reads/writes from returning in the covered IMUI
  surfaces.
- 2026-05-14: made emitted adapter seam records read-only as well. `AdapterSignalRecord` and
  `AdapterSignalMetadata` now expose constructor/accessor APIs while `report_adapter_signal(...)`
  remains the canonical emission path. `AdapterSeamOptions` stays as a public-field input options
  bag because callers need to provide a reporter and focus-restore target ergonomically.
- 2026-05-14: made `DragValueCoreResponse` accessor-first in `fret-ui-editor`. Scrub response
  storage for `dragging`, `hovered`, `pressed`, and `focused` is now private, `DragValueCore`
  constructs the response internally, and `DragValue` / `AxisDragValue` consume those signals
  through read-only accessors. The response no longer exposes external default construction. The
  source gate rejects public fields, stale direct field reads, and public default construction.
  Focused gates passed locally: `cargo check -p fret-ui-editor --tests`,
  `cargo nextest run -p fret-ui-editor drag_value --no-fail-fast`,
  `cargo check -p fret-demo --bin imui_editor_proof_demo`, `python tools/gate_imui_workstream_source.py`,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.
- 2026-05-14: made `DebugDrawResponse` accessor-first in `fret-ui-kit`. Response and summary
  storage are private, external default construction is gone, and public callers use
  `response()`, `list_summary()`, and `command_summaries()`. Cookbook and smoke tests now read the
  response through accessors, and the source gate rejects the old public-field shape.
- 2026-05-14: made `DebugDrawCommandSummary` and `DebugDrawListSummary` accessor-first as well.
  Command kind/channel/clip metadata and aggregate list counters remain readable through explicit
  accessors, but storage, default construction, and final-clip-depth mutation stay internal to
  `debug_draw_controls`. Cookbook, smoke tests, and debug-draw owner tests now use accessor reads,
  and the source gate rejects public summary fields or external default construction from returning.
- 2026-05-14: hardened `tools/gate_imui_workstream_source.py` with an opaque-output-struct check
  for the sealed IMUI response/context/summary records. The gate now parses each listed public
  output struct body and fails on any externally public field, so new response-surface cleanup does
  not depend only on per-field string markers.
- 2026-05-14: made `VecEditAxisOutcome` and `TransformEditAxisOutcome` accessor-first in
  `fret-ui-editor`. Axis edit events still carry the same section/axis/session-close values, but
  storage and construction are internal to the editor controls. The editor proof reads them through
  `section()`, `axis()`, and `outcome()`, and the source gate covers the records with the reusable
  opaque-output-struct check.
- 2026-05-14: hardened the opaque-output gate again so it scans the `fret-imui`,
  `fret-ui-editor`, and `fret-ui-kit::imui` source roots for public output-style structs by suffix.
  New public `*Response`/`*Outcome`/`*Summary`/`*Signal`/`*Record`/`*Context` records must now be
  explicitly registered in `tools/gate_imui_workstream_source.py` before the field-opacity check
  can pass.
- 2026-05-14: made editor `ColorEdit` event/request/payload records accessor-first too.
  `ColorEditPaletteSlotDrop`, `ColorEditEyedropperRequest`, and `ColorEditDragDropPayload` now keep
  storage private while exposing callback reads through explicit accessors. The opaque-output gate
  suffix scan now includes `*Request`, `*Payload`, and `*Drop` records in the IMUI/editor source
  roots. Focused gates passed locally: `cargo nextest run -p fret-ui-editor color_edit
  --no-fail-fast`, `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke
  --test imui_surface_policy --no-fail-fast`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-14: extended the opaque public-struct gate from output records to shared public state
  helpers by scanning `*State` names and registering `ImUiMultiSelectState`. This keeps future
  IMUI/editor state helpers accessor-first by default while leaving explicit options/input bags
  outside the catalog.

## P3 Design Surface Readiness Gates

Use these for the current design/theme readiness note:

```powershell
rg -n "EditorThemePresetV1|ImguiLikeDense|install_editor_theme_preset_v1|reapply_installed_editor_theme_preset_v1" ecosystem/fret-ui-editor/src/theme.rs apps/fret-cookbook/examples/imui_editor_controls_basics.rs apps/fret-examples/src/imui_editor_proof_demo.rs
rg -n "component\\.imui\\.disabled_alpha|imui_text_input_style_from_theme|input_text_model_uses_compact_imui_chrome_without_focus_ring|textarea_model_uses_compact_imui_chrome_without_focus_ring|hovered_like_imgui|ImUiHoveredFlags" ecosystem/fret-ui-kit/src/imui
rg -n "ShowStyleEditor|ImGuiStyle|PushStyleColor|PushStyleVar|StyleColorsDark|StyleColorsLight|StyleColorsClassic" repo-ref/imgui/imgui.h repo-ref/imgui/imgui_demo.cpp
cargo nextest run -p fret-ui-editor default_preset_keeps_existing_editor_patch_baseline imgui_like_dense_preset_overrides_density_and_field_chrome installed_preset_can_be_reapplied_after_base_theme_reset --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui input_text_model_uses_compact_imui_chrome_without_focus_ring textarea_model_uses_compact_imui_chrome_without_focus_ring --no-fail-fast
```

Run evidence:

- 2026-05-14: deleted the unused `apply_editor_theme_patch_v1` compatibility wrapper from
  `ecosystem/fret-ui-editor/src/theme.rs`. In-tree callers already use
  `apply_editor_theme_preset_v1(...)` or `install_editor_theme_preset_v1(...)`, and
  `tools/gate_imui_workstream_source.py` rejects the old wrapper from returning.

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
- 2026-05-14: `PropertyGridVirtualizedRowCx` now mirrors `PropertyGridRowCx` with `row(...)` and
  `row_with(...)`, so virtualized property-grid callers can keep row policy centralized instead of
  copying `row_cx.row_options` into each row. The adapter smoke now uses the helper, and
  `tools/gate_imui_workstream_source.py` rejects that manual copy from returning there.
- 2026-05-14: deleted the unused public `PropertyGridRow` wrapper from
  `ecosystem/fret-ui-editor/src/composites/property_grid.rs` and its composite re-export. The
  editor row authoring surface now has one canonical grid policy path:
  `PropertyGridRowCx::row(...)` / `row_with(...)`, with custom raw rows still using
  `PropertyRow` directly. `tools/gate_imui_workstream_source.py` rejects the redundant wrapper and
  re-export from returning.
- 2026-05-14: made `PropertyGridRowCx` and `PropertyGridVirtualizedRowCx` opaque row contexts
  instead of exposing `row_options` as a public field. External authoring now keeps the row policy
  path on `row(...)` / `row_with(...)`; editor-internal composites that need row-local policy
  patches use crate-local `row_options()`. The source gate rejects public `row_options` /
  `density` fields from returning.
- 2026-05-14: made `InspectorPanelCx` opaque as well: callers get `density()`, `query()`,
  `is_query_empty()`, and `matches(...)` instead of public `query_lower` implementation state. The
  source gate rejects the old public context fields from returning.

## P3 Child Region Readiness Gates

Use these for the current child-region readiness note:

```powershell
cargo nextest run -p fret-imui child_region --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui child_region_resize --no-fail-fast
cargo check -p fret-demo --bin workspace_shell_demo
```

## P3 Collection Helper Readiness Gates

Use these for the current collection-helper readiness note:

```powershell
python tools/gate_imui_editor_collection_source.py
cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke --test imui_sortable_recipe_smoke --test imui_drag_preview_smoke --no-fail-fast
```

- 2026-05-14: `ImUiMultiSelectState` now exposes explicit storage operations instead of public
  `selected`/`anchor` fields. The proof collection uses `selected()`, `anchor()`,
  `first_selected()`, `selected_count()`, `is_empty()`, `clear()`, `new(...)`, and `single(...)`;
  `tools/gate_imui_workstream_source.py` rejects the public-field shape and direct proof-side field
  mutation from returning.
- 2026-05-14: moved visible-order selection repair into
  `ImUiMultiSelectState::from_ordered_selection(...)`. This keeps the storage helper aligned with
  Dear ImGui's `ImGuiSelectionBasicStorage` direction without copying `BeginMultiSelect` /
  `EndMultiSelect` runtime ownership into `fret-imui`; the proof collection no longer carries a
  local `proof_collection_normalize_selection(...)`.
- 2026-05-14: refreshed `P3_COLLECTION_HELPER_READINESS_2026-05-06.md` to keep multi-select
  request/IO vocabulary candidate-only. `tools/gate_imui_workstream_source.py` now rejects
  `BeginMultiSelect`/`EndMultiSelect`-style runtime names from the current `fret-ui-kit::imui`
  storage helper.
- 2026-05-14: the reusable opaque public-struct gate now includes `*State` and covers
  `ImUiMultiSelectState`, so the collection helper storage contract cannot regress to public
  `selected` / `anchor` fields by slipping outside the previous output-record catalog. Focused
  gates passed locally: `python tools/gate_imui_workstream_source.py`,
  `python tools/check_workstream_catalog.py`, `cargo nextest run -p fret-imui interaction_drag
  --no-fail-fast`, `cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke
  --test imui_sortable_recipe_smoke --test imui_drag_preview_smoke --no-fail-fast`, and
  `git diff --check`.

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

## P4 Performance Alignment Review Gates

Use these when changing the Dear ImGui / Zed / egui performance interpretation:

```powershell
rg -n "FrameArenaScratch|bounds-tree|diag perf|diag stats|renderer churn|Zed|egui" docs/workstreams/imui-imgui-gap-closure-v1/P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Run evidence:

- 2026-05-15: registered `P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md` in
  `WORKSTREAM.json`, `TODO.md`, `MILESTONES.md`, and this evidence file so performance discipline
  remains part of the active Dear ImGui gap read.
- The review keeps perf work in `diag-perf-attribution-v1`, `ui-perf-zed-smoothness-v1`, and the
  product-chain docking perf gate. It explicitly rejects turning egui's full-layout-every-frame
  model or Dear ImGui's widget breadth into a `fret-imui` runtime/API widening target.

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
cargo nextest run -p fret-imui table_helper_applies_explicit_row_and_cell_background_overrides --no-fail-fast
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
- 2026-05-16: `cargo nextest run -p fret-imui
  table_helper_applies_explicit_row_and_cell_background_overrides
  table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells --no-fail-fast` passed
  locally with 2 tests, proving explicit row/cell background paint order and preserving the
  existing table layout semantics gate.
- 2026-05-14: `cargo nextest run -p fret-imui --no-fail-fast` passed locally with 163 tests.
- 2026-05-14: `python tools/gate_imui_workstream_source.py` and `git diff --check` passed locally.

## P3 Diagnostics / DevTools First-Open Gate

Use these gates for the current Dear ImGui-class diagnostics discoverability read. The discovery
gate verifies the first-open DevTools/tool-app discovery index and repo-owned campaign preflight
that a maintainer should find before opening the GUI/MCP branch. `--discovery-only` is the
cold-start entry and may still build `fretboard-dev`; use `--discovery-only --reuse-built` for the
fast drift check when the binary is already present and the goal is to validate the discovery path
without hiding it behind a large Rust build. The launched gate verifies the shared CLI-first path
that DevTools GUI and MCP consume later: direct script run, named bundle capture, latest bundle
resolution through `script.result.json:last_bundle_dir`, bundle compare, campaign execution,
`diag summarize`, and `diag dashboard`. The gate writes `gate.progress.jsonl` in launched mode so
an outer timeout still leaves the last reached stage.

```powershell
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke
python tools/diag_gate_imui_p2_devtools_first_open.py --reuse-built --out-dir target/imui-p2-devtools-first-open-smoke
```

Run evidence:

- 2026-05-14: `python tools/diag_gate_imui_p2_devtools_first_open.py --reuse-built --out-dir target/imui-p2-devtools-first-open-smoke-reuse-final --timeout-ms 240000 --poll-ms 50` passed
  locally. Run root:
  `target/imui-p2-devtools-first-open-smoke-reuse-final/1778726646728`.
- 2026-05-14: `target/imui-p2-devtools-first-open-smoke-reuse-final/1778726646728/gate.progress.jsonl`
  records the full first-open path: tool-app discovery, campaign doctor preflight, direct
  `todo-baseline` run, latest resolution, compare, campaign run, summarize, dashboard, and final
  `gate.pass`.
- 2026-05-14: direct script result
  `target/imui-p2-devtools-first-open-smoke-reuse-final/1778726646728/direct/sessions/1778726646873-112152/script.result.json`
  reports `stage=passed`, `run_id=1778726649588`, and `last_bundle_dir=1778726650016-todo-after-remove`.
- 2026-05-14: campaign summary
  `target/imui-p2-devtools-first-open-smoke-reuse-final/1778726646728/campaign/campaigns/devtools-first-open-smoke/1778726650543/regression.summary.json`
  reports `items_total=1`, `passed=1`, and zero deterministic/flaky/tooling/timeout failures.
- 2026-05-14: `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only` passed
  locally. The gate now validates `fretboard-dev list tool-apps` human and JSON output, the
  `docs/diagnostics-first-open.md` first-open anchor, the DevTools GUI and MCP launch/docs/gate
  entries, and `fretboard-dev diag doctor campaigns --json` with `ok=true`.
- 2026-05-16: `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`
  passed locally. This is the preferred quick drift check for the first-open discovery surface when
  `target/debug/fretboard-dev.exe` already exists; the non-`--reuse-built` discovery form remains
  the cold-start entry that also proves the maintainer build path.
- 2026-05-16: DevTools GUI first-open posture now stays summary-first: the header renders
  `First-open Next Actions`, `Evidence & Results` defaults to `Guide`, and the full first-open /
  dogfood / demo-metrics-debug / gate-command reference panels remain available from that guide
  surface. Focused gates passed locally:
  `cargo nextest run -p fret-devtools devtools_first_open_next_action_lines_prioritize_stateful_workflow devtools_first_open_lines_surface_canonical_paths devtools_dogfood_workflow_lines_surface_ui_gallery_loop devtools_demo_metrics_debug_lines_surface_canonical_routes devtools_gate_command_lines_surface_first_class_gates --no-fail-fast`
  and `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`.
- 2026-05-14: tightened the first-open discovery gate so `docs/diagnostics-first-open.md` must link
  maintainers from aggregate `skipped_policy` outcomes to the policy-skip / capability-provenance
  checklist, while preserving the distinction between `capability_source` and
  `capabilities_check_path`. Focused gates passed locally:
  `python -m py_compile tools/diag_gate_imui_p2_devtools_first_open.py` and
  `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`.
- 2026-05-14: a full launched rerun with
  `python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke-2026-05-14-discovery-gate --timeout-ms 240000`
  exceeded the local 10 minute command timeout before returning a result. A later short diagnostic
  rerun showed the last recorded step was `cargo build -p fret-demo --bin todo_demo`, so the gate
  now writes stage progress and supports `--reuse-built` to separate build cost from diagnostics
  path verification.
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

## Latest Focused Results

2026-05-16 teaching-comment cleanup:

- `python tools/gate_imui_facade_teaching_source.py` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-examples-imui` passed.
- `git diff --check` passed.

2026-05-17 property-row wrapping value layout gate:

- `cargo nextest run -p fret-ui-editor row_value_slot_grows_to_wrapping_value_text_under_narrow_layout --no-fail-fast` passed.

2026-05-17 property-grid wrapping value layout gate:

- `cargo nextest run -p fret-ui-editor property_grid_keeps_rows_separated_when_value_text_wraps_under_narrow_layout row_value_slot_grows_to_wrapping_value_text_under_narrow_layout row_value_slot_keeps_overflow_visible_for_wrapping_value_children --no-fail-fast` passed.

2026-05-17 editor-notes proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast` passed.
- `cargo check -p fret-demo --bin editor_notes_demo` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 shared text-role layout gate:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib base_single_line_text_roles_stay_single_line_under_narrow_layout paragraph_text_role_measures_multiple_lines_under_narrow_layout --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 generic list text-role slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib list_from_strings_uses_shared_single_line_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 generic tree text-role slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib default_tree_row_label_uses_shared_list_row_text_role tree_toggle_glyph_uses_shared_chrome_glyph_text_role --no-fail-fast` passed.

2026-05-17 file tree text-role slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib file_tree_row_icon_uses_shared_chrome_glyph_text_role file_tree_row_label_uses_shared_list_row_text_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 retained table text-role slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib retained_table_text_uses_shared_table_cell_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 examples table proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-demo --bin table_demo --bin table_stress_demo` passed.
- `cargo nextest run -p fret-examples --test table_demo_surface table_demo_keeps_fixed_table_text_on_roles --test table_stress_demo_surface table_stress_demo_keeps_fixed_table_text_on_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 datatable proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test datatable_demo_surface datatable_demo_keeps_fixed_table_text_on_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 virtual-list stress proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-demo --bin virtual_list_stress_demo` passed.
- `cargo nextest run -p fret-examples --test virtual_list_stress_demo_surface virtual_list_stress_demo_keeps_fixed_row_text_on_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 IMUI virtual-list fixed-row clip slice:

- `cargo fmt --package fret-imui --package fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib fixed_virtual_list_rows_clip_content_to_row_height known_virtual_list_rows_clip_content_to_known_row_height measured_virtual_list_rows_keep_content_overflow_visible_for_measurement --no-fail-fast` passed.
- `cargo nextest run -p fret-imui virtual_list_fixed_rows_clip_oversized_row_content --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 retained tree fixed-row clip slice:

- `cargo fmt --package fret-ui-kit` passed.
- `cargo fmt --check --package fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib retained_tree_fixed_rows_clip_to_row_height retained_tree_known_rows_clip_to_row_height retained_tree_measured_rows_keep_overflow_visible_for_measurement retained_tree_fixed_rows_mount_as_clip_boundaries retained_tree_measured_rows_do_not_force_row_clip --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\check_workstream_catalog.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 retained file-tree fixed-row clip slice:

- `cargo fmt --package fret-ui-kit` passed.
- `cargo fmt --check --package fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib file_tree_retained_row_layout_clips_to_row_height file_tree_retained_rows_mount_as_clip_boundaries --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\check_workstream_catalog.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 retained table fixed-row clip slice:

- `cargo fmt --package fret-ui-kit` passed.
- `cargo fmt --check --package fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_fixed_body_row_layout_clips_to_row_height table_measured_body_row_layout_keeps_overflow_visible_for_measurement table_virtualized_retained_fixed_rows_mount_as_clip_boundaries table_virtualized_retained_measured_rows_do_not_force_row_clip --no-fail-fast` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_virtualized_retained_nested_pressable_remains_hittable_when_pointer_row_selection_disabled table_virtualized_retained_pointer_row_selection_policy_list_like table_virtualized_retained_colpin_alignment_gate_measured_rows_do_not_shrink_width table_virtualized_retained_colpin_alignment_gate_across_pin_resize_and_overflow --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\check_workstream_catalog.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 fixed-row clip milestone traceability slice:

- `MILESTONES.md` now records the IMUI virtual-list, retained tree, retained file-tree, and
  retained/eager table fixed-row clip results.
- `tools/gate_imui_workstream_source.py` now checks those milestone anchors so the row-owner clip
  contract does not disappear from the lane summary.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\check_workstream_catalog.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 canvas datagrid stress proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-demo --bin canvas_datagrid_stress_demo` passed.
- `cargo nextest run -p fret-examples --test canvas_datagrid_stress_demo_surface canvas_datagrid_stress_demo_keeps_header_text_on_readout_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 date picker proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test date_picker_demo_surface date_picker_demo_keeps_fixed_chrome_text_on_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 components gallery table and overlay proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test components_gallery_surface components_gallery_table_torture_uses_text_roles components_gallery_chrome_and_controls_use_text_roles components_gallery_overlay_text_uses_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 markdown proof chrome text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test markdown_demo_surface markdown_demo_chrome_text_uses_shared_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-19 markdown image placeholder text-role slice:

- Source gap before fix: `markdown_demo` still kept image placeholder text in local direct
  `TextProps`, including the clickable placeholder path. These placeholders can contain long image
  URLs, so they should not use a fixed single-line role, but they also should not bypass the shared
  text-role contract.
- `markdown_demo_image_placeholder_text(...)` now routes placeholder copy through
  `decl_text::text_paragraph_break_words(...)` and preserves the demo-owned muted foreground via
  `inherit_foreground(...)`. The optional `Pressable` activation/semantics wrapper remains local.
- `apps/fret-examples/tests/markdown_demo_surface.rs`,
  `apps/fret-examples/tests/text_role_residual_surface.rs`, and
  `tools/gate_imui_workstream_source.py` guard the migration and remove `markdown_demo` from the
  residual direct text allowlist.
- First post-fix focused `cargo nextest` runs for `markdown_demo_surface` and
  `text_role_residual_surface` timed out while background Cargo/Rustc compilation continued. Retried
  after Cargo/Rustc exited.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test markdown_demo_surface
  markdown_demo_chrome_text_uses_shared_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `git diff --check` passed.

2026-05-17 fret-examples residual bare text capability slice:

- `cargo fmt -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 fret-examples residual direct TextProps capability slice:

- `assets_demo` image/SVG stats and `image_heavy_memory_demo` image-memory stats now route through
  `text_control_readout(...)` instead of local direct `TextProps` construction.
- `text_role_residual_surface` now counts direct `TextProps { ... }` struct literals as well as
  `cx.text(...)` and `TextProps::new(...)`, limiting remaining direct text construction to explicit
  text/IME/rendering capability proofs.
- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 form proof header text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test form_demo_surface form_demo_header_status_uses_control_readout_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 sonner proof header text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test sonner_demo_surface sonner_demo_header_text_uses_fixed_chrome_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 echarts proof title text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test echarts_demo_surface echarts_demo_chart_titles_use_section_chrome_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 virtual row fallback text removal slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib missing_tree_virtual_row_placeholder_is_not_text missing_file_tree_virtual_row_placeholder_is_not_text --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 gallery disabled toaster placeholder slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app gallery_driver_disabled_toaster_does_not_emit_empty_text --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery app-sidebar collapsed placeholder slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app sidebar_app_collapsed_projects_do_not_emit_empty_text --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 fret-ui-ai empty placeholder helper slice:

- `cargo fmt -p fret-ui-ai` passed.
- `cargo nextest run -p fret-ui-ai hidden_ai_element_paths_use_non_text_placeholder --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery status-bar readout role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery driver chrome text role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery driver chrome label slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery minimal-root text role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery debug-HUD text role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery shell content/nav text role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery editor preview text role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews code_editor_mvp_internal_helpers_prefer_ui_child_over_anyelement --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 code-view editor preview prose slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews editor_code_view_header_uses_paragraph_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 text editor/conformance header prose slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews editor_text_conformance_headers_use_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 code-editor IME gate button-label slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews code_editor_mvp_internal_helpers_prefer_ui_child_over_anyelement --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.

2026-05-17 docking arbitration body/readout text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test docking_arbitration_surface docking_arbitration_demo_keeps_body_and_state_text_on_roles --no-fail-fast` passed.
- `cargo check -p fret-demo --bin docking_arbitration_demo` passed.
- `python tools\gate_imui_workstream_source.py` passed.

2026-05-17 docking/container-query panel text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-demo --bin container_queries_docking_demo --bin docking_demo` passed.
- `cargo nextest run -p fret-examples --test container_queries_docking_surface container_queries_docking_demo_keeps_fixed_panel_text_on_roles --test docking_demo_surface docking_demo_keeps_panel_text_on_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.

2026-05-17 gallery retained-table torture text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_table_retained_torture_uses_structured_table_debug_ids --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery data-table torture text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_data_table_torture_exposes_header_row_anchor --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 data-table snippet table-cell text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test data_table_docs_surface data_table_snippets_keep_fixed_cell_text_on_table_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-19 table snippet table-cell text-role slice:

- Added `text_table_cell_emphasis(...)` in `fret-ui-kit::declarative::text` as a medium-weight
  derivative of the shared table-cell role. It keeps the same single-line, shrinkable,
  min-width-zero, ellipsis layout contract as `text_table_cell(...)`.
- UI Gallery ordinary Table snippets now share directory-local `table_cell_text(...)` and
  `table_cell_text_emphasis(...)` helpers. Demo/Usage/Footer/RTL/Actions and fixed body cells in
  Children no longer mount bare `ui::text(...)` inside fixed table cells. The rich
  `table_head_children(...)` / `table_caption_children(...)` sample text remains intentionally
  scoped to the children-API follow-up.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_cell_emphasis_text_keeps_single_line_truncation_and_medium_weight --no-fail-fast` passed.
- `cargo nextest run -p fret-ui-gallery --test table_docs_surface table_snippets_keep_fixed_cell_text_on_table_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-19 checkbox table-cell text-role slice:

- Red repro: `cargo nextest run -p fret-ui-gallery --test checkbox_table_action_first_surface
  checkbox_table_snippet_keeps_fixed_cell_text_on_table_role --no-fail-fast` failed before the fix
  because `checkbox/table.rs` was missing the local table-cell text helper.
- `apps/fret-ui-gallery/src/ui/snippets/checkbox/table.rs` keeps the existing
  `cx.actions().models::<act::ToggleAllRows>` / `.action(act::ToggleAllRows)` select-all flow, and
  now routes member/role fixed table cells through a local helper backed by
  `text_table_cell(...)` instead of `ui::text(id)` / `ui::text(role)`.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test checkbox_table_action_first_surface --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-19 typography table-cell text-role slice:

- Red repro: `cargo nextest run -p fret-ui-gallery --test typography_docs_surface
  typography_table_snippets_keep_fixed_cell_text_on_table_role --no-fail-fast` failed before the
  fix because `typography/mod.rs` was missing the shared table-cell text helper.
- `apps/fret-ui-gallery/src/ui/snippets/typography/table.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/typography/demo.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/typography/rtl.rs` now route fixed table body cells through
  a typography-local helper backed by `text_table_cell(...)`. Typography prose, headings, and rich
  inline-link examples are intentionally unchanged.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test typography_docs_surface typography_table_snippets_keep_fixed_cell_text_on_table_role --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed with `PYTHONIOENCODING=utf-8`.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery data-grid text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_data_grid_uses_table_cell_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery data paragraph text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_data_grid_uses_table_cell_text_roles gallery_data_table_torture_exposes_header_row_anchor gallery_tree_torture_uses_control_readout_for_status_text --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery inspector torture text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_inspector_torture_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery virtual-list torture text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews harness_virtual_list_torture_uses_fixed_row_text_roles harness_ui_kit_list_torture_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery harness header text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_table_retained_torture_uses_structured_table_debug_ids harness_hit_test_torture_uses_header_text_roles harness_virtual_list_torture_uses_fixed_row_text_roles harness_ui_kit_list_torture_uses_fixed_row_text_roles harness_view_cache_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery view-cache list text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews harness_view_cache_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery view-cache control-label slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews harness_view_cache_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery view-cache popover body slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews harness_view_cache_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery tree torture status text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_tree_torture_uses_control_readout_for_status_text --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery overlay status text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_overlay_status_text_uses_control_readout_roles gallery_menus_last_action_uses_control_readout_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery overlay scroll-row text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_overlay_scroll_rows_use_list_row_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery overlay body prose slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_overlay_body_copy_uses_paragraph_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery chrome torture control-label slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews page_chrome_torture_uses_control_label_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 tooltip body text role slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib tooltip_body_text_uses_compact_paragraph_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 collection proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test imui_editor_collection_text_roles_surface imui_editor_proof_collection_fixed_text_uses_shared_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test imui_editor_collection_select_all_surface imui_editor_proof_demo_keeps_collection_select_all_app_owned_and_explicit --test imui_editor_collection_rename_surface imui_editor_proof_demo_keeps_collection_inline_rename_app_owned_and_explicit --no-fail-fast` passed.
- `cargo check -p fret-examples` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 child-region auto-height gate slice:

- `cargo fmt -p fret-imui` passed.
- `cargo nextest run -p fret-imui child_region_without_height_constraint_auto_sizes_to_content --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-20 child-region auto-width gate slice:

- `cargo nextest run -p fret-imui child_region_without_width_constraint_auto_sizes_to_content --no-fail-fast` passed.

2026-05-20 menu item row-anchor and shared-role slice:

- `cargo nextest run -p fret-ui-kit --features imui --lib menu_item_root_pressable_owns_visible_row_children menu_item_label_text_uses_shared_list_row_text_role menu_item_shortcut_text_uses_shared_control_readout_role menu_item_indicator_text_uses_shared_chrome_glyph_role --no-fail-fast` passed.
- `cargo nextest run -p fret-imui interaction_press::lifecycle::menu_item interaction_menu_tabs::menu_activation interaction_menu_tabs::submenu_hover --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.

2026-05-18 AI AudioPlayer state-marker text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- First `cargo nextest run -p fret-ui-gallery --test ai_audio_player_text_role_surface audio_player_state_markers_use_non_text_spacers --no-fail-fast` timed out at 120s while waiting on/building the workspace.
- Retried with a longer timeout: `cargo nextest run -p fret-ui-gallery --test ai_audio_player_text_role_surface audio_player_state_markers_use_non_text_spacers --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI visible snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- First `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` failed because the source test matched rustfmt-sensitive one-line `text_paragraph(...)` calls.
- After making the source test check role calls and text payloads separately, `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Queue visible snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Command snippet chrome text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  command_snippet_chrome_text_uses_shared_roles --no-fail-fast` failed because the view-runtime
  closure already receives an `ElementContext`, so the attempted `cx.elements()` bridge was wrong.
- After switching the view-runtime snippet to pass the closure `cx` directly, the focused gate
  compiled but failed on a rustfmt-sensitive import marker. The source test now checks the stable
  role import and `IntoUiElementInExt` markers separately.
- A subsequent focused run timed out at 300s during compilation. Retried with a longer timeout:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  command_snippet_chrome_text_uses_shared_roles --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.

2026-05-18 gallery Accordion trigger text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  accordion_snippet_trigger_text_uses_button_label_role --no-fail-fast` passed.
- First `python tools/gate_imui_workstream_source.py` failed because the source marker required
  single-line `decl_text::text_button_label(cx, ...)` calls while rustfmt expanded several long
  labels. After loosening the marker to the stable helper call, the gate passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery ToggleGroup item text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  toggle_group_snippet_item_text_uses_button_label_role --no-fail-fast` timed out at 300s while a
  background `rustc` compile continued.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  toggle_group_snippet_item_text_uses_button_label_role --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Toggle item text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  toggle_snippet_item_text_uses_button_label_role --no-fail-fast` timed out at 300s while waiting
  on/building the focused test binary.
- Retried after background compilation finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  toggle_snippet_item_text_uses_button_label_role --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Button children text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  button_children_snippet_text_uses_button_label_role --no-fail-fast` timed out at 300s while
  background compilation continued.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  button_children_snippet_text_uses_button_label_role --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Tabs custom text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  tabs_snippet_custom_text_uses_shared_roles --no-fail-fast` timed out at 300s while background
  compilation continued.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  tabs_snippet_custom_text_uses_shared_roles --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Collapsible text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  collapsible_snippet_text_uses_shared_roles --no-fail-fast` timed out at 300s while background
  compilation continued.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  collapsible_snippet_text_uses_shared_roles --no-fail-fast` passed.
- After adding `collapsible/rtl.rs` and `collapsible/demo.rs` to the same source test, the final
  long-timeout run of
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  collapsible_snippet_text_uses_shared_roles --no-fail-fast` passed.
- First `python tools/gate_imui_workstream_source.py` failed on rustfmt-sensitive markers in
  `collapsible/usage.rs` and later `collapsible/demo.rs`; after splitting those markers into helper
  calls and text payloads, the gate passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Checkpoint visible snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI simple chrome visible snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- First `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast`
  timed out at 300s while a background `rustc` compile continued.
- Retried after the compile finished: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI selector/branch marker text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- First `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast`
  timed out at 300s while a background `rustc` compile continued.
- Retried after the compile finished: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI prompt/plan/commit-large text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- First `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast`
  timed out at 300s while a background `rustc` compile continued.
- Retried after the compile finished: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed after replacing a rustfmt-sensitive
  CommitLarge marker check with stable split markers.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI queue-prompt/transcription text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI WebPreview text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Chat text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI PromptInput provider/docs text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI chrome/readout text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Confirmation content text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Task content text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Conversation instrumentation text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI usage snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Message usage text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Canvas world spike text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Image demo text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI PromptInput referenced sources text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Artifact code-display status-marker slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `python -m json.tool tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-artifact-docs-run-action.json` passed.
- `git diff --check` passed.

2026-05-18 AI ChainOfThought composable text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI TestResults composable text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Workflow snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Suggestions/reasoning/transcript text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI custom-children text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Sidebar snippet chrome text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  sidebar_snippet_chrome_text_uses_shared_roles --no-fail-fast` timed out at 184s while a
  background `rustc` compile continued.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  sidebar_snippet_chrome_text_uses_shared_roles --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery AlertDialog custom text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  alert_dialog_snippet_custom_text_uses_shared_roles --no-fail-fast` failed before test execution:
  `rustc-LLVM ERROR: IO failure on output stream: no space on device`.
- `cargo clean -p fret-ui-gallery` removed 2726 generated build files, freeing about 6.2GiB.
- `python tools/gate_imui_workstream_source.py` passed.
- Retried after cleaning generated build artifacts:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  alert_dialog_snippet_custom_text_uses_shared_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery HoverCard text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  hover_card_snippet_text_uses_shared_roles --no-fail-fast` failed before test execution:
  `rustc-LLVM ERROR: IO failure on output stream: no space on device`.
- `cargo clean -p fret-ui-gallery`, `cargo clean -p fret-examples`,
  `cargo clean -p fret-examples-imui`, and `cargo clean -p fret-demo` removed generated build
  artifacts; free space recovered from under 1GiB to about 32GiB.
- Retried after cleaning generated build artifacts:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  hover_card_snippet_text_uses_shared_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Popover align text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  popover_align_snippet_text_uses_shared_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Tooltip keyboard shortcut text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  tooltip_keyboard_shortcut_text_uses_shared_role --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-19 gallery Dialog scroll-row text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  dialog_scroll_row_text_uses_shared_roles --no-fail-fast` failed before the fix because
  `dialog/scrollable_content.rs` was missing the shared `decl_text` role import.
- `apps/fret-ui-gallery/src/ui/snippets/dialog/scrollable_content.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/dialog/sticky_footer.rs` route scroll filler rows through
  `text_list_row_label(...)` instead of `ui::raw_text(format!(...))`.
- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  dialog_scroll_row_text_uses_shared_roles --no-fail-fast` timed out while contending with the
  parallel `cargo check` package-cache lock; Cargo/Rustc processes were allowed to exit.
- Retried after the lock cleared:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  dialog_scroll_row_text_uses_shared_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.

2026-05-19 extras marquee perf fixed title text-role slice:

- Source gap before fix: `apps/fret-examples/src/extras_marquee_perf_demo.rs` used local
  `ui::text(...).font_semibold()` for the fixed perf-probe title. The demo is a marquee animation
  perf surface, not a text rendering capability probe.
- `marquee_perf_title_text(...)` now routes that title through
  `decl_text::text_section_chrome_label(...)`, keeping fixed probe chrome single-line and
  shrinkable under resize.
- `apps/fret-examples/tests/extras_marquee_perf_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard the role mapping and forbid the old local title
  styling from returning.
- The first post-fix `cargo nextest run -p fret-examples --test
  extras_marquee_perf_demo_surface extras_marquee_perf_demo_keeps_title_on_chrome_role
  --no-fail-fast` exposed a missing `AnyElement` import; after the import was added, a later
  `nextest` attempt timed out while background Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test extras_marquee_perf_demo_surface
  extras_marquee_perf_demo_keeps_title_on_chrome_role --no-fail-fast` passed.
- `cargo check -p fret-demo --bin extras_marquee_perf_demo` passed.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 effect reference chrome text-role slice:

- Source gap before fix: `custom_effect_v3_demo.rs`, `postprocess_theme_demo.rs`, and
  `liquid_glass_demo.rs` still used local `TextProps` for fixed effect overlay/header/card titles.
  These are ordinary chrome/readout slots around renderer/effect proofs, not text-rendering
  capability probes.
- `custom_effect_v3_demo` now uses `overlay_label_text(...)` backed by
  `decl_text::text_section_chrome_label(...)`. `postprocess_theme_demo` uses
  `postprocess_title_text(...)` for the fixed header title and `postprocess_readout_text(...)` for
  the explanatory header readout. `liquid_glass_demo` uses section-chrome helpers for both overlay
  pill labels and card titles.
- App-owned foreground inheritance and absolute/header/card container geometry remain local; the
  shrink/single-line text semantics now come from shared roles.
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`,
  `apps/fret-examples/tests/text_role_residual_surface.rs`, and
  `tools/gate_imui_workstream_source.py` guard the migration and remove these three demos from the
  residual direct text allowlist.
- First post-fix focused `cargo nextest` runs for `custom_effect_overlay_text_surface` and
  `text_role_residual_surface` timed out while background Cargo/Rustc compilation continued. Retried
  after Cargo/Rustc exited.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v3_and_effect_reference_chrome_use_shared_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.

2026-05-19 custom effect web template overlay/control text-role slice:

- Source gap before fix: `custom_effect_v2_identity_web_demo.rs`,
  `custom_effect_v2_lut_web_demo.rs`, and `custom_effect_v2_glass_chrome_web_demo.rs` remained in
  the residual direct-text allowlist for fixed Web template overlay/control labels. These were
  ordinary demo chrome/readouts, not text-rendering capability probes.
- Starter and LUT templates now use `overlay_label_text(...)` backed by
  `text_section_chrome_label(...)` for their badges and `overlay_readout_text(...)` backed by
  `text_control_readout(...)` for unsupported-state and keyboard-hint text. Their absolute hints
  position containers around shared readout text instead of constructing local `TextProps`.
- The glass/chrome template now routes slider row names through `text_control_label(...)`, values
  through `text_control_readout(...)`, and the unsupported-state message through the same readout
  helper. App-owned foreground colors are preserved through `inherit_foreground(...)`.
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`,
  `apps/fret-examples/tests/text_role_residual_surface.rs`, and
  `tools/gate_imui_workstream_source.py` guard the migration and remove the three Web template demos
  from the residual direct text allowlist.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v2_web_templates_use_shared_text_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v2_web_overlay_readouts_use_shared_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `cargo check -p fret-examples --lib --target wasm32-unknown-unknown` passed.
- `cargo check -p fret-demo-web --target wasm32-unknown-unknown` passed with existing unrelated
  `fret-platform-native` clipboard dead-code warnings.
- `cargo check -p fret-examples --lib` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.

2026-05-19 custom effect web overlay/readout text-role slice:

- Source gap before fix: `custom_effect_v2_web_demo.rs` still used three local `TextProps`
  constructions for fixed overlay/readout text: the unsupported-state message, the WebGPU badge
  label, and the bottom keyboard hint. These are ordinary UI chrome/readouts, not text-rendering
  capability probes.
- The demo now uses `overlay_label_text(...)` backed by `text_section_chrome_label(...)` for the
  badge and `overlay_readout_text(...)` backed by `text_control_readout(...)` for the unsupported
  state and keyboard hint. App-owned foreground colors are preserved with `inherit_foreground(...)`.
- The bottom hint keeps its absolute positioning by wrapping the shared readout role in a positioned
  container, avoiding a regression back to local text layout policy.
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`,
  `apps/fret-examples/tests/text_role_residual_surface.rs`, and
  `tools/gate_imui_workstream_source.py` guard the migration and remove `custom_effect_v2_web_demo`
  from the residual direct text allowlist.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v2_web_overlay_readouts_use_shared_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v1_v2_overlay_labels_use_shared_chrome_role --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-examples --lib --target wasm32-unknown-unknown` passed.
- `cargo check -p fret-demo-web --target wasm32-unknown-unknown` passed with existing unrelated
  `fret-platform-native` clipboard dead-code warnings.
- `cargo check -p fret-demo` passed with an existing unrelated warning in
  `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` about an unused `Result`.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 GenUI demo visible text-role slice:

- Source gap before fix: `apps/fret-examples/src/genui_demo.rs` used local
  `ui::text(...).text_sm()` builders for action queue lines, JSON state/spec/schema panes, prompt
  text, spec issue lines, toolbar switch labels, count/readout values, stream status text, and
  stream guidance. It also used `ui::text("")` as an empty spacer in the editor panel.
- `genui_code_line_text(...)` now routes queue/state/validation/spec/schema/prompt pane lines
  through `decl_text::text_code_block(...)`, keeping code/log content on the code-text role
  instead of ordinary prose.
- `genui_readout_text(...)` now routes fixed toolbar labels, count/status/issue lines, patch-only
  status, and stream summaries through `decl_text::text_control_readout(...)`.
- `genui_paragraph_text(...)` routes stream guidance through `decl_text::text_compact_paragraph(...)`;
  the old empty text spacer was removed rather than converted to another invisible text node.
- `apps/fret-examples/tests/genui_demo_surface.rs` and `tools/gate_imui_workstream_source.py` guard
  the role mapping and forbid local `ui::text(...)`, `.text_sm()`, and the empty text spacer from
  returning.
- First post-fix `cargo nextest run -p fret-examples --test genui_demo_surface
  genui_demo_keeps_tool_text_on_roles --no-fail-fast` timed out while background Cargo/Rustc
  compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test genui_demo_surface
  genui_demo_keeps_tool_text_on_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo` passed with the existing unrelated warning in
  `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` about an unused `Result`.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.

2026-05-19 gallery NavigationMenu link-label text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test navigation_menu_docs_surface
  navigation_menu_custom_link_labels_use_shared_button_label_role --no-fail-fast` failed before
  the fix because `navigation_menu/demo.rs` was missing the shared `decl_text` import and custom
  link labels still used `cx.text(label)`.
- `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/demo.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/docs_demo.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/rtl.rs` now route custom icon/text link
  labels through `text_button_label(...)`.
- Card title/body line-clamp text remains deliberately out of this slice because it likely needs a
  separate list/card description role decision instead of a mechanical button-label migration.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test navigation_menu_docs_surface
  navigation_menu_custom_link_labels_use_shared_button_label_role --no-fail-fast` passed.
- `cargo check -p fret-ui-gallery --test navigation_menu_docs_surface` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 compact paragraph line-clamp text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  compact_paragraph_line_clamp_text_uses_two_line_clamped_layout --no-fail-fast` failed before the
  fix because `text_compact_paragraph_line_clamp(...)` did not exist.
- Red repro:
  `cargo nextest run -p fret-ui-gallery --test navigation_menu_docs_surface
  navigation_menu_list_item_copy_uses_shared_title_and_clamped_paragraph_roles --no-fail-fast`
  first timed out while a background Cargo compile continued, then failed before the fix because
  `navigation_menu/demo.rs` list-item titles/descriptions still used local `TextProps` policy.
- `ecosystem/fret-ui-kit/src/declarative/text.rs` now provides
  `text_compact_paragraph_line_clamp(...)` as a paragraph-family dense clamp helper with fill width,
  min-width-zero flex behavior, `max-height` derived from the theme line height, word wrapping, and
  ellipsis overflow.
- `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/demo.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/docs_demo.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/rtl.rs` now route ordinary list-item titles
  through `text_button_label(...)` and descriptions through
  `text_compact_paragraph_line_clamp(..., 2)`. Featured home-card brand copy remains explicit
  visual styling outside this slice.
- `cargo nextest run -p fret-ui-kit --features imui --lib
  compact_paragraph_line_clamp_text_uses_two_line_clamped_layout --no-fail-fast` passed.
- `cargo nextest run -p fret-ui-gallery --test navigation_menu_docs_surface
  navigation_menu_list_item_copy_uses_shared_title_and_clamped_paragraph_roles --no-fail-fast`
  passed.
- `cargo fmt -p fret-ui-kit -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-kit --features imui --lib` passed.
- `cargo check -p fret-ui-gallery --test navigation_menu_docs_surface` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Kbd custom-copy text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  kbd_snippets_route_custom_copy_through_shared_text_roles --no-fail-fast` failed before the fix
  because `kbd/demo.rs` did not import shared text roles and still rendered the `+` key separator
  through `ui::text("+")`.
- `apps/fret-ui-gallery/src/ui/snippets/kbd/demo.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/kbd/rtl.rs` now route `+` separators through
  `text_chrome_glyph(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/kbd/group.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/kbd/tooltip.rs` now route inline helper/tooltip copy
  through `text_control_readout(...)` instead of local `ui::text(...).text_sm()` policy.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  kbd_snippets_route_custom_copy_through_shared_text_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Table children custom text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  table_children_snippet_routes_custom_text_through_shared_roles --no-fail-fast` failed before the
  fix because `table/children.rs` did not import shared text roles and still rendered custom header
  child text through `ui::text(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/table/children.rs` now routes custom header child text
  through `super::table_cell_text(...)` and caption prose through
  `decl_text::text_paragraph(...)`.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  table_children_snippet_routes_custom_text_through_shared_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `python tools\gate_imui_workstream_source.py` attempt failed because the older table
  source check still required the intentional `ui::text("Status ")` / `ui::text("Amount ")` /
  `ui::text("(USD)")` exception. The gate was updated to require the new table-cell markers and
  forbid the old bare text markers.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Separator menu text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test separator_docs_surface
  separator_menu_snippet_routes_section_copy_through_shared_roles --no-fail-fast` failed before the
  fix because `separator/menu.rs` still used local `Theme` plus `ui::text(...).fixed_line_box_px`
  policy.
- `apps/fret-ui-gallery/src/ui/snippets/separator/menu.rs` now routes section titles through
  `text_section_chrome_label(...)` and descriptions through `text_control_readout(...)`.
- `cargo nextest run -p fret-ui-gallery --test separator_docs_surface
  separator_menu_snippet_routes_section_copy_through_shared_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test separator_docs_surface` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Item slotted text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  item_snippets_route_slotted_copy_through_shared_text_roles --no-fail-fast` failed before the fix
  because `item/dropdown.rs` still used local `ui::text("Select").text_sm()` for a custom trigger
  child.
- `apps/fret-ui-gallery/src/ui/snippets/item/dropdown.rs` now routes the custom trigger child
  through `text_button_label(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/item/gallery.rs` now routes the download header through
  `text_section_chrome_label(...)` and issue-number side columns through
  `text_control_readout(...)`.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  item_snippets_route_slotted_copy_through_shared_text_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Spinner amount readout text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  spinner_item_amount_text_uses_control_readout_role --no-fail-fast` failed before the fix because
  `spinner/demo.rs` did not import shared text roles and still rendered the payment amount through
  `ui::text("$100.00")`.
- `apps/fret-ui-gallery/src/ui/snippets/spinner/demo.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/spinner/rtl.rs` now route item amount/status values through
  `text_control_readout(...)`.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  spinner_item_amount_text_uses_control_readout_role --no-fail-fast` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `PYTHONIOENCODING=utf-8 python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery AvatarStack direction label text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_avatar_stack_direction_labels_use_shared_chrome_text_role --no-fail-fast` failed
  before the fix because `shadcn_extras/avatar_stack.rs` did not import shared text roles and still
  rendered direction labels through `ui::text("LTR").font_medium()`.
- `apps/fret-ui-gallery/src/ui/snippets/shadcn_extras/avatar_stack.rs` now routes LTR/RTL
  direction labels through `text_section_chrome_label(...)`.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_avatar_stack_direction_labels_use_shared_chrome_text_role --no-fail-fast` timed out
  without a capturable result while Cargo/Rustc processes continued; those processes were allowed
  to finish before retrying.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_avatar_stack_direction_labels_use_shared_chrome_text_role --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `PYTHONIOENCODING=utf-8 python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Kanban card title text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_kanban_card_titles_use_shared_button_label_role --no-fail-fast` failed before the
  fix because `shadcn_extras/kanban.rs` did not import shared text roles and still rendered card
  titles through local `ui::text(item.name.clone()).font_medium().truncate()` policy.
- `apps/fret-ui-gallery/src/ui/snippets/shadcn_extras/kanban.rs` now routes the app-owned Kanban
  card title slot through `text_button_label(...)`.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_kanban_card_titles_use_shared_button_label_role --no-fail-fast` timed out without a
  capturable result while Cargo/Rustc processes continued; those processes were allowed to finish
  before retrying.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_kanban_card_titles_use_shared_button_label_role --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `PYTHONIOENCODING=utf-8 python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 Shadcn Extras AnnouncementTitle composable-title slice:

- Decision: do not mechanically rewrite
  `AnnouncementTitle::new([cx.text("Shadcn Extras landed in Fret")])` at the gallery call site.
  Upstream `repo-ref/kibo/packages/announcement/index.tsx` keeps `AnnouncementTitle` children-first
  while applying `truncate` on the title container, so Fret keeps the composable surface and moves
  the resize contract into `fret-ui-shadcn`.
- `ecosystem/fret-ui-shadcn/src/extras/announcement.rs` now gives `AnnouncementTitle` a
  shrinkable/min-width-zero clipped title container, scopes medium `text-sm` inherited typography,
  and recursively forces nested text children to single-line ellipsis.
- `apps/fret-ui-gallery/src/ui/snippets/shadcn_extras/announcement.rs` intentionally keeps the raw
  composable title call; the gallery source test and IMUI workstream gate prevent it from being
  mistaken for an unowned fixed-row text role.
- First `cargo nextest run -p fret-ui-shadcn
  announcement_title_keeps_composable_children_on_truncated_title_contract --no-fail-fast`
  attempts timed out while cold Cargo/Rustc compilation continued in the background; those
  processes were allowed to finish before retrying. A later unscoped nextest attempt failed during
  test-list enumeration with Windows `os error 740` from an unrelated
  `extras_relative_time_auto_update` integration-test executable, so the component gate was scoped
  to `--lib`.
- `cargo nextest run -p fret-ui-shadcn --lib
  announcement_title_keeps_composable_children_on_truncated_title_contract --no-fail-fast` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_announcement_title_keeps_composable_title_surface --no-fail-fast` passed after an
  earlier cold-compile timeout was allowed to finish.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `PYTHONIOENCODING=utf-8 python tools\gate_imui_workstream_source.py` passed.

2026-05-19 gallery ContextMenu trigger text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  context_menu_trigger_copy_uses_shared_readout_text_role --no-fail-fast` failed before the fix
  because `context_menu/demo.rs` was missing the shared `decl_text` role import.
- `apps/fret-ui-gallery/src/ui/snippets/context_menu/demo.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/basic.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/submenu.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/shortcuts.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/groups.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/icons.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/checkboxes.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/radio.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/destructive.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/sides.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/rtl.rs` now route dashed trigger copy through
  `text_control_readout(...)`.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  context_menu_trigger_copy_uses_shared_readout_text_role --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- A parallel post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  context_menu_trigger_copy_uses_shared_readout_text_role --no-fail-fast` attempt printed PASS but
  hit the tool timeout while contending with Cargo locks from `cargo check`.
- Retried after Cargo locks cleared:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  context_menu_trigger_copy_uses_shared_readout_text_role --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery ScrollArea visible text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test scroll_area_docs_surface
  scroll_area_snippets_route_visible_text_through_shared_roles --no-fail-fast` failed before the fix
  because `scroll_area/demo.rs` was missing the shared `decl_text` role import.
- `apps/fret-ui-gallery/src/ui/snippets/scroll_area/demo.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/scroll_area/rtl.rs` route fixed tag/RTL rows through
  `text_list_row_label(...)` and headings through `text_section_chrome_label(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/scroll_area/usage.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/scroll_area/compact_helper.rs` route body copy through
  `text_paragraph(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/scroll_area/horizontal.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/scroll_area/nested_scroll_routing.rs` route figure/card
  captions through `text_control_readout(...)`.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test scroll_area_docs_surface
  scroll_area_snippets_route_visible_text_through_shared_roles --no-fail-fast` timed out while a
  background Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test scroll_area_docs_surface
  scroll_area_snippets_route_visible_text_through_shared_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test scroll_area_docs_surface` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Drawer goal/diagnostics text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  drawer_remaining_custom_text_uses_shared_roles --no-fail-fast` failed before the fix because
  `drawer/demo.rs` was missing the shared `decl_text` role import.
- `apps/fret-ui-gallery/src/ui/snippets/drawer/demo.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/drawer/rtl.rs` now route goal readout and unit labels
  through `text_control_readout(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/drawer/nested.rs` now routes guidance copy through
  `text_paragraph(...)`; `apps/fret-ui-gallery/src/ui/snippets/drawer/outside_press.rs` routes
  deterministic probe description through `text_paragraph(...)` and activation count through
  `text_control_readout(...)`.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  drawer_remaining_custom_text_uses_shared_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.
- `python tools\gate_imui_workstream_source.py` passed after tightening the source gate to use
  stable rustfmt-resistant anchors for the multiline readout call.

2026-05-19 gallery Drawer scroll/side text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  drawer_scroll_and_side_body_text_uses_shared_roles --no-fail-fast` failed before the fix because
  `drawer/scrollable_content.rs` was missing the shared `decl_text` role import.
- `apps/fret-ui-gallery/src/ui/snippets/drawer/scrollable_content.rs` now routes scroll rows
  through `text_list_row_label(...)`, and `apps/fret-ui-gallery/src/ui/snippets/drawer/sides.rs`
  routes side body copy through `text_paragraph(...)`.
- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  drawer_scroll_and_side_body_text_uses_shared_roles --no-fail-fast` timed out while a background
  Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  drawer_scroll_and_side_body_text_uses_shared_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Pagination text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  selected_pagination_page_number_helpers_use_shared_button_label_role --no-fail-fast` failed
  before the fix because pagination page-number helpers still used no-context
  `ui::text(...).tabular_nums()` builders and the RTL snippet used bare
  `cx.text(to_arabic_numerals(...))`.
- `apps/fret-ui-gallery/src/ui/snippets/pagination/demo.rs`,
  `compact_builder.rs`, `custom_text.rs`, `simple.rs`, `usage.rs`, `routing.rs`, `extras.rs`, and
  `rtl.rs` now route page labels through `decl_text::text_button_label(...)` via a context-bound
  helper. `extras.rs` also routes Fret-specific explanatory copy through `text_paragraph(...)`.
- `ecosystem/fret-ui-shadcn/src/pagination.rs` routes `PaginationPrevious` and `PaginationNext`
  visible labels through shared `text_button_label(...)` instead of bare `cx.text(...)`.
- `cargo fmt -p fret-ui-gallery -p fret-ui-shadcn` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  selected_pagination_page_number_helpers_use_shared_button_label_role --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First
  `cargo nextest run -p fret-ui-shadcn pagination_root_is_w_full_and_labeled
  pagination_content_and_item_emit_list_semantics pagination_link_active_stamps_selected
  pagination_link_without_action_keeps_enabled_visual_chrome pagination_disabled_link_wraps_in_opacity
  --no-fail-fast` attempt timed out at 304s while a background Cargo/Rustc compile continued.
- The timed-out Cargo/Rustc process group from that attempt was stopped after it continued compiling
  without a capturable result.
- `cargo check -p fret-ui-shadcn --lib` passed as the direct compile gate for the changed shadcn
  Pagination implementation.
- `cargo fmt --check -p fret-ui-gallery -p fret-ui-shadcn` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Carousel status/readout text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  selected_carousel_status_readouts_use_shared_control_readout_role --no-fail-fast` failed before
  the fix because `carousel/api.rs` was missing the shared `decl_text` role import and still built
  status text through ad-hoc `TextProps`.
- `apps/fret-ui-gallery/src/ui/snippets/carousel/api.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/carousel/events.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/carousel/plugin_autoplay_stop_on_focus.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/carousel/plugin_autoplay_stop_on_last_snap.rs` now route
  centered diagnostic/status lines through `text_control_readout(...)` instead of local
  word-wrapping `TextProps` blocks.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  selected_carousel_status_readouts_use_shared_control_readout_role --no-fail-fast` timed out
  without a capturable result; no background Cargo/Rustc processes remained afterward.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- Retried after the compile/check path completed:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  selected_carousel_status_readouts_use_shared_control_readout_role --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 AI Attachments inline hover-card text-role slice:

- Source gap before fix: `attachments_inline.rs` did not import shared text roles and still
  rendered hover-card attachment labels/media types through default `ui::text(...)` builders. This
  slice added the focused source test and gate marker with the implementation change.
- `apps/fret-ui-gallery/src/ui/snippets/ai/attachments_inline.rs` now routes hover-card attachment
  labels through `decl_text::text_list_row_label(...)` and media-type values through
  `decl_text::text_control_readout(...)`.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_attachments_inline_hover_card_uses_shared_text_roles --no-fail-fast` timed out while a
  background Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_attachments_inline_hover_card_uses_shared_text_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 AI PlanContent text-role slice:

- Source gap before fix: `plan_demo.rs` outer title/body already used shared roles, but inner
  PlanContent still rendered section headings, bullet rows, and the custom Build button child
  through local `ui::text(...)` styling. The overview body also carried local `text_sm().wrap(...)`
  paragraph policy instead of the shared paragraph role.
- `apps/fret-ui-gallery/src/ui/snippets/ai/plan_demo.rs` now routes inner section headings through
  `decl_text::text_section_chrome_label(...)`, overview body copy through
  `decl_text::text_paragraph(...)`, bullet rows through `decl_text::text_list_row_label(...)`, and
  the custom Build child through `decl_text::text_button_label(...)`.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_prompt_and_plan_snippets_use_shared_outer_text_roles --no-fail-fast` timed out while a
  background Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- Retried after the compile finished; the test failed once because the source-test marker expected
  the runtime-concatenated long paragraph string while `include_str!` sees Rust's split source
  literal. The marker was corrected to source-level stable fragments.
- Final focused test:
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_prompt_and_plan_snippets_use_shared_outer_text_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 AI PromptInput cursor custom-text slice:

- Source gap before fix: `prompt_input_cursor_demo.rs` still used local `ui::text(...)` styling for
  command item labels, filenames/paths, hover-card rules text, trigger counts, and the tabs footer
  readout. Those are fixed chrome/readout/identifier roles rather than free-form text capability
  probes.
- `apps/fret-ui-gallery/src/ui/snippets/ai/prompt_input_cursor_demo.rs` now routes those custom
  child text nodes through shared list-row, code-label, control-readout, section-chrome, and
  button-label roles.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_prompt_input_cursor_custom_text_uses_shared_roles --no-fail-fast` timed out while a background
  Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- `cargo fmt -p fret-ui-gallery` passed and applied the expected rustfmt wrapping.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_prompt_input_cursor_custom_text_uses_shared_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 AI Shimmer demo chrome text-role slice:

- Source gap before fix: Shimmer typography/duration/elements demos still used local
  `ui::text(...)` styling for demo labels and inline non-shimmer text. These nodes are gallery
  chrome/readout text, not the Shimmer text capability itself.
- `apps/fret-ui-gallery/src/ui/snippets/ai/shimmer_typography_demo.rs`,
  `shimmer_duration_demo.rs`, and `shimmer_elements_demo.rs` now route demo labels through
  `decl_text::text_control_readout(...)`; the inline non-shimmer prefix/suffix in
  `shimmer_elements_demo.rs` uses section-chrome/control-readout roles. `Shimmer::new(...)` calls
  remain as the explicit animated text capability surface.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_shimmer_demo_chrome_text_uses_shared_roles --no-fail-fast` timed out while a background
  Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- `cargo fmt -p fret-ui-gallery` passed and applied the expected rustfmt wrapping.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_shimmer_demo_chrome_text_uses_shared_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.
- `rg -n "ui::text\(|cx\.text\(" apps\fret-ui-gallery\src\ui\snippets\ai -g "*.rs"` returned no
  matches after this slice; remaining AI snippet text rendering is component-owned surfaces such as
  `Shimmer::new(...)` or other explicit AI text-capability APIs.

2026-05-19 imui node-graph compatibility title text-role slice:

- Source gap before fix: `apps/fret-examples/src/imui_node_graph_demo.rs` is explicitly a retained
  bridge compatibility proof for `fret-node`, but its fixed title still used local
  `fret_ui_kit::ui::text("imui node-graph compatibility proof").font_semibold()` styling. That
  made the compatibility demo teach an ad-hoc text policy under resize instead of the shared text
  role vocabulary.
- `apps/fret-examples/src/imui_node_graph_demo.rs` now routes the title through
  `compat_section_text(...)`, backed by `decl_text::text_section_chrome_label(...)`. The demo still
  imports the IMUI writer trait through `fret_imui::prelude::UiWriter` and keeps the retained
  bridge posture explicit; no direct `fret_authoring` dependency or new `fret-imui` API was added.
- `apps/fret-examples/tests/imui_node_graph_demo_surface.rs`,
  `tools/gate_imui_facade_teaching_source.py`, and `tools/gate_imui_workstream_source.py` now guard
  the compatibility-only wording, shared section role, and absence of the old local title styling.
- `tools/gate_imui_facade_teaching_source.py` no longer uses the stale exact-count check for
  `) -> fret_ui::element::AnyElement {` in `imui_editor_proof_demo.rs`. The gate now checks the
  actual proof-local shared text role helpers and role calls, so adding legitimate role helpers does
  not force the proof back to a worse shape.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_facade_teaching_source.py tools\gate_imui_workstream_source.py`
  passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-demo --features node-graph-demos-legacy --bin imui_node_graph_demo` passed
  with pre-existing `private_interfaces` warnings from node-graph legacy/domain driver visibility.
- First post-fix `cargo nextest run -p fret-examples --features node-graph-demos-legacy --test
  imui_node_graph_demo_surface imui_node_graph_demo_keeps_compat_title_on_shared_role
  --no-fail-fast` timed out while background Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --features node-graph-demos-legacy --test
  imui_node_graph_demo_surface imui_node_graph_demo_keeps_compat_title_on_shared_role
  --no-fail-fast` passed, with the same pre-existing node-graph legacy/domain
  `private_interfaces` warnings.

2026-05-19 embedded viewport chrome/readout text-role slice:

- Source gap before fix: `apps/fret-examples/src/embedded_viewport_demo.rs` used local
  `ui::text(...)` children for fixed ToggleGroup viewport-size labels and local
  `ui::text(format!(...)).text_sm()` builders for compact target/click/input status lines. Under
  narrow resize those are button/readout chrome, not paragraph text.
- `embedded_viewport_button_label_text(...)` now routes the size labels through
  `decl_text::text_button_label(...)`, and `embedded_viewport_readout_text(...)` routes target,
  click, and last-input status through `decl_text::text_control_readout(...)`.
- `apps/fret-examples/tests/embedded_viewport_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard the shared roles and forbid the old local
  `ui::text(...)` builders for those fixed chrome/readout slots.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test embedded_viewport_demo_surface
  embedded_viewport_demo_keeps_fixed_chrome_text_on_roles --no-fail-fast` passed.
- `cargo check -p fret-demo --bin embedded_viewport_demo` passed.

2026-05-19 window hit-test probe fixed text-role slice:

- Source gap before fix: `apps/fret-examples/src/window_hit_test_probe_demo.rs` had a fixed 44px
  header title and compact diagnostic/status lines built with local
  `ui::text(...).font_semibold().text_sm()`, `ui::text(...).font_monospace().text_sm()`, and
  `ui::text(status).text_sm()` policy. These are fixed chrome/readout slots under resize, not
  paragraph text.
- `window_hit_test_title_text(...)` now routes the header through
  `decl_text::text_section_chrome_label(...)`, `window_hit_test_code_label_text(...)` routes the
  logical-window diagnostic identifier through `decl_text::text_code_label(...)`, and
  `window_hit_test_readout_text(...)` routes status through `decl_text::text_control_readout(...)`.
- `apps/fret-examples/tests/window_hit_test_probe_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard those role mappings and forbid the old local fixed
  text policy from returning.
- `cargo fmt --check -p fret-examples` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- First post-fix `cargo nextest run -p fret-examples --test window_hit_test_probe_demo_surface
  window_hit_test_probe_demo_keeps_fixed_text_on_roles --no-fail-fast` timed out while background
  Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test window_hit_test_probe_demo_surface
  window_hit_test_probe_demo_keeps_fixed_text_on_roles --no-fail-fast` passed.
- `cargo check -p fret-examples --test window_hit_test_probe_demo_surface` passed.

2026-05-19 launcher utility window fixed text-role slice:

- Source gap before fix: `apps/fret-examples/src/launcher_utility_window_demo.rs` used local
  `ui::text(...)` builders for the frameless-window drag title, effective-style diagnostic line,
  status readout, and resize-handle glyph. These are fixed chrome/readout/glyph slots under
  resize, not paragraph text.
- `launcher_utility_title_text(...)` now routes the drag title through
  `decl_text::text_section_chrome_label(...)`, `launcher_utility_code_label_text(...)` routes the
  effective-style diagnostic through `decl_text::text_code_label(...)`,
  `launcher_utility_readout_text(...)` routes status through `decl_text::text_control_readout(...)`,
  and `launcher_utility_glyph_text(...)` routes the resize arrow through
  `decl_text::text_chrome_glyph(...)`.
- `apps/fret-examples/tests/launcher_utility_window_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard those role mappings and forbid the old local fixed
  text policy from returning.
- First post-fix `cargo nextest run -p fret-examples --test launcher_utility_window_demo_surface
  launcher_utility_window_demo_keeps_fixed_text_on_roles --no-fail-fast` timed out while background
  Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test launcher_utility_window_demo_surface
  launcher_utility_window_demo_keeps_fixed_text_on_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.

2026-05-19 launcher utility window materials fixed text-role slice:

- Source gap before fix: `apps/fret-examples/src/launcher_utility_window_materials_demo.rs` used
  local `ui::text(...)` builders for the material-window title, effective material/style diagnostic
  line, and status readout. These are fixed chrome/readout slots under resize, not paragraph text.
- `launcher_utility_materials_title_text(...)` now routes the title through
  `decl_text::text_section_chrome_label(...)`, `launcher_utility_materials_code_label_text(...)`
  routes the effective-style diagnostic through `decl_text::text_code_label(...)`, and
  `launcher_utility_materials_readout_text(...)` routes status through
  `decl_text::text_control_readout(...)`.
- `apps/fret-examples/tests/launcher_utility_window_materials_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard those role mappings and forbid the old local fixed
  text policy from returning.
- First post-fix `cargo nextest run -p fret-examples --test
  launcher_utility_window_materials_demo_surface
  launcher_utility_window_materials_demo_keeps_fixed_text_on_roles --no-fail-fast` timed out while
  background Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test launcher_utility_window_materials_demo_surface
  launcher_utility_window_materials_demo_keeps_fixed_text_on_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.

2026-05-19 API workbench lite fixed text-role slice:

- Source gap before fix: `apps/fret-examples/src/api_workbench_lite_demo.rs` used local
  `ui::text(...)` builders and a `ThemeSnapshot` parameter on `shell_frame(...)` for app title,
  sidebar labels, first-contact copy, active base URL, and history loading/error/empty states.
  These are fixed app chrome/readout/identifier or paragraph slots, not ad-hoc text policy.
- `api_workbench_section_text(...)`, `api_workbench_readout_text(...)`,
  `api_workbench_code_label_text(...)`, and `api_workbench_paragraph_text(...)` now bridge the
  app-facing `AppRenderContext` to shared declarative text roles through `cx.elements()`.
- The app title/sidebar labels route through `decl_text::text_section_chrome_label(...)`,
  first-contact copy routes through `decl_text::text_paragraph(...)`, the active base URL routes
  through `decl_text::text_code_label(...)`, and history loading/error/empty states route through
  `decl_text::text_control_readout(...)`.
- The old `shell_frame` `ThemeSnapshot` parameter was removed because it only existed to support
  local muted text color policy.
- `apps/fret-examples/tests/api_workbench_lite_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard those role mappings and forbid the old local fixed
  text policy from returning.
- First post-fix `cargo nextest run -p fret-examples --test api_workbench_lite_demo_surface
  api_workbench_lite_demo_keeps_fixed_text_on_roles --no-fail-fast` timed out while background
  Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test api_workbench_lite_demo_surface
  api_workbench_lite_demo_keeps_fixed_text_on_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-demo --bin api_workbench_lite_demo` passed.

2026-05-19 hello counter status/help text-role slice:

- Source gap before fix: `apps/fret-examples/src/hello_counter_demo.rs` used local text policy for
  the compact status line and the step help copy. Those are resize-sensitive readout/body slots,
  not text-rendering capability probes.
- `hello_counter_status_text(...)` now routes the status line through
  `decl_text::text_control_readout(...)`, and `hello_counter_paragraph_text(...)` routes the step
  help copy through `decl_text::text_paragraph(...)`.
- The large `ui::text(count.to_string()).text_size_px(Px(72.0))` counter display intentionally
  remains local for this slice. It is a visual display value, not a compact control readout; forcing
  it onto the existing compact role would encode the wrong semantics before a dedicated large
  readout/display-number role exists.
- `apps/fret-examples/tests/hello_counter_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard the shared status/help roles, forbid the old local
  status/help builders, and preserve the explicit large counter display exception.
- First post-fix `cargo nextest run -p fret-examples --test hello_counter_demo_surface
  hello_counter_demo_keeps_status_and_help_text_on_roles --no-fail-fast` timed out while background
  Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test hello_counter_demo_surface
  hello_counter_demo_keeps_status_and_help_text_on_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-examples --lib` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 simple todo visible text-role slice:

- Source gap before fix: `apps/fret-examples/src/simple_todo_demo.rs` used local
  `ui::text(...)` builders for the title summary, empty-state body copy, footer remaining-count
  readout, and todo row labels. This is an ordinary app proof, not a text rendering capability
  surface, so fixed/list/chrome text should not own ad-hoc wrap/truncate policy.
- `simple_todo_readout_text(...)` now routes summary/footer status through
  `decl_text::text_control_readout(...)`, `simple_todo_compact_paragraph_text(...)` routes the
  empty-state copy through `decl_text::text_compact_paragraph(...)`, and
  `simple_todo_row_label_text(...)` routes todo row labels through
  `decl_text::text_list_row_label(...)`.
- Row done/active foreground remains app-owned state policy via `inherit_foreground(...)`; the
  shared role owns the single-line shrink/truncate layout semantics.
- `apps/fret-examples/tests/simple_todo_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` now guard the helper mapping and require
  `simple_todo_demo.rs` to stay free of `ui::text(...)` residuals.
- `cargo nextest run -p fret-examples --test simple_todo_demo_surface
  simple_todo_demo_keeps_visible_text_on_roles --no-fail-fast` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo --bin simple_todo_demo` passed.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 todo demo visible text-role slice:

- Source gap before fix: `apps/fret-examples/src/todo_demo.rs` still used local `ui::text(...)`
  builders for title, status, progress readouts, empty-state labels, filter labels, and active row
  labels, plus local `ui::rich_text(...)` layout policy for completed rows with strikethrough. This
  is a responsive app proof, not a text-rendering capability surface.
- `ecosystem/fret-ui-kit/src/declarative/text.rs` now exposes
  `text_list_row_label_attributed(...)`, an attributed-text variant of the shared list-row label
  role. It keeps fill-width, shrinkable, single-line ellipsis semantics while allowing row labels
  to carry per-span decoration such as strikethrough.
- `todo_demo` now routes title/status/progress/empty/filter/row labels through local helpers backed
  by shared `decl_text` roles: chrome title, control readout, compact paragraph, button label,
  list-row label, and attributed list-row label. Done/active foreground and strikethrough remain
  app state/decoration policy; fixed-row text layout is role-owned.
- `apps/fret-examples/tests/todo_demo_surface.rs` and `tools/gate_imui_workstream_source.py` guard
  the role mapping and require `todo_demo.rs` to stay free of local `ui::text(...)`,
  `ui::rich_text(...)`, and `typography::` text layout policy.
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_TEXT_ROLE_MATRIX_2026-05-17.md` now records the
  attributed list-row derived role and focused gate.
- `cargo nextest run -p fret-ui-kit --lib
  attributed_list_row_label_text_uses_fill_width_single_line_truncation --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test todo_demo_surface
  todo_demo_keeps_visible_text_on_roles --no-fail-fast` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo --bin todo_demo` passed.
- `cargo fmt --check -p fret-ui-kit -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 async playground visible text-role slice:

- Source gap before fix: `apps/fret-examples/src/async_playground_demo.rs` used local
  `ui::text(...)` builders for fixed app chrome, catalog item labels, inspector status/policy
  readouts, policy switch labels, query parameter guidance, query identifiers, and result body
  text. Several of those slots also carried local muted color, weight, or truncation policy.
- `async_chrome_title_text(...)`, `async_section_text(...)`, `async_list_row_text(...)`,
  `async_readout_text(...)`, `async_code_label_text(...)`, and
  `async_compact_paragraph_text(...)` now route the proof through shared declarative text roles.
  Fixed chrome/control rows stay single-line and shrinkable; result/guidance body copy uses the
  compact paragraph role where wrapping is intentional.
- The query panel helper chain no longer takes `ThemeSnapshot` just to style text. Theme ownership
  remains where it still controls panel/card/background chrome; text resize semantics are owned by
  the role helpers.
- `apps/fret-examples/tests/async_playground_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard the role mapping and forbid the old local
  `ui::text(...)`, weight/truncate, muted text-color, and redundant theme-parameter patterns from
  returning.
- First post-fix `cargo nextest run -p fret-examples --test async_playground_demo_surface
  async_playground_demo_keeps_visible_text_on_roles --no-fail-fast` timed out while background
  Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test async_playground_demo_surface
  async_playground_demo_keeps_visible_text_on_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo` passed with an existing unrelated warning in
  `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` about an unused `Result`.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 residual builder-text capability gate tightening:

- Source gap before fix: `apps/fret-examples/tests/text_role_residual_surface.rs` counted
  `cx.text(...)`, `TextProps::new(...)`, and direct `TextProps { ... }` residuals, but missed
  builder-style `ui::text(...)` / `ui::rich_text(...)` calls. That left the proof-app text-role
  contract weaker than the current resize-semantics policy.
- `text_role_residual_surface` now counts `ui::text(...)` and `ui::rich_text(...)` too. The
  remaining `ui::text(...)` entries in `fret-examples` are explicitly documented as
  capability/display payloads: `hello_counter_demo`'s large numeric display and
  `hello_world_compare_demo`'s GPUI/Fret comparison title.
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_TEXT_ROLE_MATRIX_2026-05-17.md` now records the
  current allowed residual classes, including text/IME/conformance/rendering probes and the two
  display/performance payload exceptions.
- `tools/gate_imui_workstream_source.py` now requires the stricter residual test shape so the
  gate cannot drift back to missing builder-style text construction.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `git diff --check` passed.

2026-05-19 query detail raw-text role cleanup:

- Source gap before fix: `query_demo.rs` and `query_async_tokio_demo.rs` used `ui::raw_text(...)`
  for query detail rows. `text_role_residual_surface` did not count `ui::raw_text(...)`,
  `ui::text_block(...)`, or all `cx.text_props(...)` calls, so proof apps could bypass the text-role
  contract even after builder-style `ui::text(...)` started being counted.
- `query_readout_text(...)`, `query_readout_text_with_color(...)`, and `query_data_text(...)` now
  route the query detail rows through shared `decl_text` roles. Status/error/duration/retry lines
  use `text_control_readout(...)`; fetched data values use `text_code_label(...)`; destructive or
  muted foreground remains app-owned state policy through `inherit_foreground(...)`.
- `imui_editor_proof_demo` deleted the old `EditorCompactReadoutStyle::text_props(...)` readout
  helper path and now uses the shared control-readout role for those proof readouts.
- `text_role_residual_surface` now counts `ui::raw_text(...)`, `ui::text_block(...)`, and all
  `cx.text_props(...)` calls in addition to the earlier residual counters.
- `apps/fret-examples/tests/query_demo_surface.rs` and `tools/gate_imui_workstream_source.py` guard
  the query role mapping and forbid the old raw-text policy from returning.
- First post-fix parallel nextest runs for `query_demo_surface` and `text_role_residual_surface`
  timed out while background Cargo/Rustc compilation continued. Retried after Cargo/Rustc exited.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test query_demo_surface
  query_demos_keep_detail_text_on_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `cargo nextest run -p fret-examples proof_drag_preview_card_uses_single_line_text_roles
  --no-fail-fast` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo` passed with an existing unrelated warning in
  `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` about an unused `Result`.
- `git diff --check` passed.

2026-05-19 custom effect overlay chrome text-role slice:

- Source gap before fix: `custom_effect_v1_demo.rs` and `custom_effect_v2_demo.rs` were still in the
  direct-text residual allowlist for a single fixed overlay pill label. Those labels are ordinary
  fixed chrome over an effect preview, not text-rendering capability probes.
- Both demos now use `custom_effect_label_text(...)`, backed by
  `decl_text::text_section_chrome_label(...)`, and preserve the white overlay foreground with
  `inherit_foreground(...)`. Custom effect ABI/runtime ownership is unchanged.
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`,
  `apps/fret-examples/tests/text_role_residual_surface.rs`, and
  `tools/gate_imui_workstream_source.py` guard the migration and remove the two demos from the
  residual direct text allowlist.
- First post-fix `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v1_v2_overlay_labels_use_shared_chrome_role --no-fail-fast` timed out while
  background Cargo/Rustc compilation continued. Retried after Cargo/Rustc exited.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v1_v2_overlay_labels_use_shared_chrome_role --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo` passed with an existing unrelated warning in
  `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` about an unused `Result`.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 AI Agent text-role and Accordion role-preservation slice:

- Source gap before fix: the real `fret-ui-ai` `Agent` component still carried local `TextProps`
  for header names, section labels, instruction body copy, and output labels; tool trigger
  descriptions used bare `cx.text(...)`. The shadcn `AccordionTrigger` recipe then unconditionally
  rewrote text children to wrapping trigger text, so a caller-supplied shared role could still lose
  its single-line resize contract.
- `AgentHeader` now routes the name through `decl_text::text_chrome_title(...)`;
  `AgentInstructions`, `AgentTools`, and `AgentOutput` route fixed labels through
  `text_section_chrome_label(...)`; instruction body copy uses `text_compact_paragraph(...)`; and
  `AgentTool` trigger descriptions use `text_list_row_label(...)`.
- `AccordionTrigger` now applies its wrapping trigger defaults only to bare text children. Children
  that already carry inherited text-role metadata keep their role-owned style, wrap, overflow, and
  inherited metadata; hover underline conversion also preserves that metadata.
- `tools/gate_imui_workstream_source.py` now guards the Agent role mapping and the Accordion
  role-preservation markers.
- `cargo fmt --check -p fret-ui-ai -p fret-ui-shadcn` passed.
- `cargo nextest run -p fret-ui-ai agent_header_label_uses_chrome_title_text_role
  agent_default_text_uses_shared_resize_roles
  agent_tools_multiple_uncontrolled_renders_label_and_item_text --no-fail-fast` passed.
- Unbounded `cargo nextest run -p fret-ui-shadcn ...` listed all package test binaries and hit an
  unrelated `extras_relative_time_auto_update` elevation manifest (`os error 740`). Retried with
  the intended lib-only scope.
- `cargo nextest run -p fret-ui-shadcn --lib
  accordion_trigger_label_defaults_do_not_force_foreground_color
  accordion_trigger_label_defaults_preserve_shared_text_role_contracts
  accordion_trigger_hover_underline_preserves_shared_text_role_metadata
  accordion_trigger_label_defaults_keep_bare_text_wrapping_policy --no-fail-fast` passed.
- `cargo check -p fret-ui-ai --lib` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.

2026-05-19 shadcn Table role-preservation slice:

- Source gap before fix: `TableCell` / `TableHead` could still rewrite shared text-role children
  by setting leaf `TextStyle`, `TextWrap::None`, and `TextOverflow::Clip` as recipe defaults. That
  meant a caller could correctly use `text_table_cell(...)` and still lose its single-line
  ellipsis resize contract under shadcn table composition.
- `apply_table_cell_text_defaults(...)`, `apply_table_inherited_text_style(...)`,
  `apply_table_footer_inherited_style(...)`, and `apply_table_head_inherited_style(...)` now treat
  inherited text-role metadata as a protected role scope. Recipe defaults continue to apply to bare
  text, but shared role children keep their role-owned style, wrap, overflow, and inherited
  metadata.
- `table_cell_preserves_shared_text_role_contracts` and
  `table_head_children_preserve_shared_text_role_contracts` cover the role-child path; the existing
  bare-text tests remain in the same focused run to prove table defaults still apply where intended.
- `tools/gate_imui_workstream_source.py` now guards the Table role-preservation helper shape and
  tests so the recipe cannot drift back to unconditional leaf typography rewrites.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib table_cell_preserves_shared_text_role_contracts
  table_head_children_preserve_shared_text_role_contracts
  table_applies_text_sm_defaults_to_unstyled_text_cells
  table_head_children_apply_header_typography_to_plain_text --no-fail-fast` failed because both
  role-preservation tests observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib table_cell_preserves_shared_text_role_contracts
  table_head_children_preserve_shared_text_role_contracts
  table_applies_text_sm_defaults_to_unstyled_text_cells
  table_head_children_apply_header_typography_to_plain_text --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn ItemTitle role-preservation slice:

- Source gap before fix: `ItemTitle::new_children(...)` intentionally patched child text leaves to
  apply the title slot contract. That is correct for bare text and ordinary rich text, but it also
  overwrote shared role children such as `text_chrome_title(...)`, even though those already carry
  a title-family role with single-line ellipsis semantics.
- `patch_item_title_text_style_recursive(...)` now delegates to a role-scope-aware helper. It still
  patches bare `Text`, `StyledText`, and `SelectableText` children so custom ItemTitle content gets
  the title fallback, but it skips any subtree that already carries inherited text-role metadata.
- `item_title_children_patch_bare_text_with_title_typography` and the existing
  `item_title_children_patch_rich_text_with_title_typography` cover the strong slot fallback;
  `item_title_children_preserve_shared_text_role_contracts` proves shared title-role children keep
  `style: None`, ellipsis overflow, and role metadata.
- `tools/gate_imui_workstream_source.py` now guards the ItemTitle helper/test shape.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  item_title_children_patch_bare_text_with_title_typography
  item_title_children_preserve_shared_text_role_contracts
  item_title_children_patch_rich_text_with_title_typography --no-fail-fast` failed because the
  role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  item_title_children_patch_bare_text_with_title_typography
  item_title_children_preserve_shared_text_role_contracts
  item_title_children_patch_rich_text_with_title_typography --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-20 shadcn SelectLabel menu-group text role slice:

- Source gap before fix: `SelectLabel` rendered fixed select/listbox group labels with local
  `ui::text(...)` sizing, muted foreground, and `nowrap()` policy inside the overlay row renderer.
  That kept menu/select group labels outside the shared text-role vocabulary and made the same
  fixed-row resize policy likely to be duplicated by ContextMenu, DropdownMenu, and Menubar.
- `fret-ui-kit::declarative::text::text_menu_group_label(...)` now owns the muted `text-xs`,
  fill-width, shrinkable, single-line ellipsis contract for non-interactive menu/listbox group
  headings. It is a derived role, not a control readout.
- `SelectLabel` now consumes `decl_text::text_menu_group_label(...)` while keeping its existing
  entry data model cloneable. This avoids pushing move-only `AnyElement` children into
  `SelectEntry` just to fix text policy.
- `menu_group_label_text_uses_muted_xs_single_line_truncation` proves the role contract; existing
  Select tests keep label/separator focus positions and group semantics in the focused run; and
  `tools/gate_imui_workstream_source.py` guards the role helper plus the Select consumption path.
- Red run before fix:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  menu_group_label_text_uses_muted_xs_single_line_truncation --no-fail-fast` failed because
  `text_menu_group_label(...)` did not exist.
- Post-fix focused runs passed:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  menu_group_label_text_uses_muted_xs_single_line_truncation --no-fail-fast`; and
  `cargo nextest run -p fret-ui-shadcn --lib
  select_label_and_separator_do_not_affect_positions_or_initial_focus
  select_group_renders_group_semantics_node --no-fail-fast`.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-kit --features imui --lib` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-20 shadcn menu-family group-label text role slice:

- Source gap before fix: `DropdownMenuLabel`, `ContextMenuLabel`, and `MenubarLabel` still rendered
  non-interactive menu group headings with local `ui::text(...)` sizing, foreground, line-height,
  and `nowrap()` policy. That duplicated the `SelectLabel` fixed-row text contract and left three
  menu-family surfaces open to resize drift.
- Each menu component now has a local `*_label_element(...)` helper that owns the row container
  padding/inset shape while delegating the text leaf to
  `fret-ui-kit::declarative::text::text_menu_group_label(...)`. Menu item labels, shortcuts,
  submenu chevrons, and icon/indicator foreground policy remain menu-owned.
- Focused unit tests prove all three helpers produce shared role text leaves with no leaf-local
  `TextStyle`/color, fill-width shrinkable layout, `TextWrap::None`, and ellipsis overflow:
  `dropdown_menu_label_element_uses_shared_menu_group_text_role`,
  `context_menu_label_element_uses_shared_menu_group_text_role`, and
  `menubar_label_element_uses_shared_menu_group_text_role`.
- `tools/gate_imui_workstream_source.py` now guards the DropdownMenu, ContextMenu, and Menubar
  helper/import/test shape and rejects the old local group-label `ui::text(...)` builders from
  returning.
- First focused run timed out during compilation after 244 seconds. No cargo/rustc process was
  killed; after the background compile completed naturally, the same focused run was repeated and
  passed.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  dropdown_menu_label_element_uses_shared_menu_group_text_role
  context_menu_label_element_uses_shared_menu_group_text_role
  menubar_label_element_uses_shared_menu_group_text_role --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-20 shadcn CommandGroup heading text role slice:

- Source gap before fix: `CommandGroup::heading(...)` rendered command/listbox group headings in
  both `CommandList` and `CommandPalette` through local `ui::text(...)` builders with command-local
  heading typography, muted foreground, and `nowrap()` policy. That duplicated the same fixed-row
  group-label contract already used by Select and menu-family labels.
- `ecosystem/fret-ui-shadcn/src/command.rs` now uses a private
  `command_group_heading_element(...)` helper for both heading render paths. The helper owns the
  row padding/container shape and delegates the text leaf to
  `fret-ui-kit::declarative::text::text_menu_group_label(...)`.
- This intentionally keeps `CommandItem` label/highlight rendering command-owned. The slice only
  moves non-interactive group heading text policy onto the shared role and indirectly covers
  combobox, native select, and data-table recipe heading consumers through `CommandGroup::heading`.
- `command_group_heading_uses_shared_menu_group_text_role` proves the heading text leaf has no
  leaf-local `TextStyle`/color, uses fill-width/min-width-zero/shrink layout, and stays
  `TextWrap::None` with ellipsis overflow. `tools/gate_imui_workstream_source.py` guards the
  helper/import/test shape and rejects the old command-local heading text helper from returning.
- Planned focused gates for this slice:
  `cargo nextest run -p fret-ui-shadcn --lib
  command_group_heading_uses_shared_menu_group_text_role --no-fail-fast`, `cargo check -p
  fret-ui-shadcn --lib`, `cargo fmt --check -p fret-ui-shadcn`, `python -m py_compile
  tools\gate_imui_workstream_source.py`, `python -m json.tool
  docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`, `python
  tools\gate_imui_workstream_source.py`, and `git diff --check`.
- Verification note: `cargo fmt --check -p fret-ui-shadcn` initially reported the new test block
  formatting around `command_group_heading_uses_shared_menu_group_text_role`; `cargo fmt -p
  fret-ui-shadcn` normalized the block, after which `cargo fmt --check -p fret-ui-shadcn` passed.
  The first focused `cargo nextest run -p fret-ui-shadcn --lib
  command_group_heading_uses_shared_menu_group_text_role --no-fail-fast` attempt timed out while
  Cargo/Rustc still held build locks. No cargo/rustc process was killed; after the background
  compile chain exited naturally, the same focused nextest command passed. `cargo check -p
  fret-ui-shadcn --lib`, `python -m py_compile tools\gate_imui_workstream_source.py`, `python -m
  json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`, `python
  tools\gate_imui_workstream_source.py`, and `git diff --check` passed.

2026-05-20 shared status-message / CommandEmpty text role slice:

- Source gap before fix: shadcn `CommandEmpty` and `CommandLoading` rendered non-interactive
  empty/loading status messages through local `ui::text(...)` builders with duplicated
  text-sm/line-height/muted foreground and `nowrap()` policy. This kept command status messages
  outside the shared resize role vocabulary even after group headings moved to
  `text_menu_group_label(...)`.
- `fret-ui-kit::declarative::text::text_status_message(...)` now owns the muted `text-sm`,
  shrinkable, single-line ellipsis role for non-interactive empty/loading/status messages. It is a
  derived role distinct from `text_menu_group_label(...)` (`text-xs` group headings) and
  `text_control_readout(...)` (`text-xs` compact auxiliary values).
- `CommandEmpty` and `CommandLoading` keep their existing `py-6` container and centered row shape,
  but their text leaves now consume `decl_text::text_status_message(...)`. Command item labels and
  highlighted query spans remain command-owned.
- `status_message_text_uses_muted_sm_single_line_truncation` proves the shared role contract.
  `command_empty_and_loading_use_shared_status_message_text_role` proves both shadcn command status
  consumers use shared-role text leaves with no leaf-local `TextStyle`/color and with
  shrink/nowrap/ellipsis behavior.
- Focused gates passed: `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn`; `cargo nextest run
  -p fret-ui-kit --features imui --lib status_message_text_uses_muted_sm_single_line_truncation
  --no-fail-fast`; `cargo nextest run -p fret-ui-shadcn --lib
  command_empty_and_loading_use_shared_status_message_text_role --no-fail-fast`; `cargo check -p
  fret-ui-kit --features imui --lib`; `cargo check -p fret-ui-shadcn --lib`; `python -m
  py_compile tools\gate_imui_workstream_source.py`; `python -m json.tool
  docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`; `python
  tools\gate_imui_workstream_source.py`; and `git diff --check`.
- Verification note: the first focused shadcn nextest attempt timed out during background
  compilation. No cargo/rustc process was killed; after the compile chain exited naturally, the
  same command passed.

2026-05-20 shadcn DataTable toolbar text role slice:

- Source gap before fix: `DataTableToolbar` still owned local `ui::text(...)` /
  `ui::raw_text(...)` policy for fixed toolbar text: faceted-trigger labels, faceted option
  labels, count chips, clear/reset action labels, and selected-count readouts. Those are fixed-row
  control texts, so resize should truncate them through shared roles instead of allowing local
  builder policy to drift.
- `data_table_toolbar_button_label(...)`, `data_table_toolbar_option_label(...)`, and
  `data_table_toolbar_readout(...)` now keep that recipe-local semantic split while delegating to
  `text_button_label(...)`, `text_list_row_label(...)`, and `text_control_readout(...)`. The
  helper names are intentionally local to the recipe; `fret-imui` stays policy-light and no new
  public text-role enum was added.
- `data_table_toolbar_fixed_text_uses_shared_roles` proves these helper outputs keep leaf
  `style`/`color` empty, carry inherited text-role typography, stay shrinkable with
  `min-width: 0`, and use single-line ellipsis. The readout path also proves muted foreground is
  inherited.
- Focused gates passed: `cargo fmt --check -p fret-ui-shadcn`; `cargo nextest run -p
  fret-ui-shadcn --lib data_table_toolbar_fixed_text_uses_shared_roles --no-fail-fast`; `cargo
  check -p fret-ui-shadcn --lib`; `python -m py_compile tools\gate_imui_workstream_source.py`;
  `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`;
  `python tools\gate_imui_workstream_source.py`; and `git diff --check`.
- Verification note: focused shadcn nextest timed out twice while Cargo/Rustc still held build
  locks. No cargo/rustc process was killed; after each compile chain exited naturally, the same
  focused nextest command passed.

2026-05-20 inherited text feature + DataTable tabular readout slice:

- Source gap before fix: the remaining DataTable pagination footer used local
  `ui::text(...).tabular_nums()` builders because the shared inherited text-role refinement could
  not express OpenType numeric features. That left a resize-sensitive fixed footer row outside the
  shared control-readout contract.
- `TextStyleRefinement` now carries subtree-default OpenType feature settings. `TextStyle::refine`,
  `TextStyleRefinement::merge`, `text_style_refinement_fingerprint(...)`, and
  `fret-ui-kit` typography bridge helpers preserve those feature settings so passive text
  measurement/cache and role helpers see the same numeric shaping policy.
- `text_control_readout_tabular(...)` and `text_control_readout_tabular_emphasis(...)` are narrow
  control-readout variants, not new stable text-role categories. They keep single-line ellipsis,
  `min-width: 0`, inherited role typography, and tabular numeric features; the emphasis variant
  adds the medium page-summary weight.
- `DataTablePagination` now consumes recipe-local `data_table_pagination_readout(...)` and
  `data_table_pagination_summary(...)` helpers backed by those shared variants, closing the last
  local pagination footer text policy without widening `fret-imui`.
- Focused gates passed: `cargo check -p fret-core`; `cargo check -p fret-ui`; `cargo check -p
  fret-ui-kit`; `cargo check -p fret-ui-shadcn`; `cargo nextest run -p fret-core --lib
  text_style_refinement_merges_font_features_in_parent_child_order
  text_style_refine_applies_inherited_font_features_after_leaf_defaults --no-fail-fast`; `cargo
  nextest run -p fret-ui --lib inherited_text_style_features_affect_passive_text_measurement
  inherited_text_style_fingerprint_tracks_feature_overrides --no-fail-fast`; `cargo nextest run -p
  fret-ui-kit --lib control_readout_tabular_text_uses_muted_single_line_truncation
  control_readout_tabular_emphasis_text_uses_medium_single_line_truncation --no-fail-fast`; `cargo
  nextest run -p fret-ui-shadcn --lib data_table_toolbar_fixed_text_uses_shared_roles
  --no-fail-fast`; `python -m py_compile tools\gate_imui_workstream_source.py`; `python -m
  json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`; `python
  tools\gate_imui_workstream_source.py`; `rustfmt --edition 2024 --check
  crates/fret-core/src/text/mod.rs crates/fret-ui/src/text/props.rs
  crates/fret-ui/src/declarative/tests/text_style_inheritance.rs
  ecosystem/fret-ui-kit/src/typography.rs ecosystem/fret-ui-kit/src/declarative/text.rs
  ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`; and `git diff --check`.
- Verification note: broader package-level `cargo fmt --check -p ...` and the first
  `fret-ui-shadcn` focused nextest attempt timed out while Cargo/Rustc still held build locks. No
  cargo/rustc process was killed; after each compile chain exited naturally, focused checks were
  rerun and passed.

2026-05-20 tabular control-readout resize gate slice:

- Source gap before fix: `text_control_readout_tabular(...)` and
  `text_control_readout_tabular_emphasis(...)` had role-local structure tests, but they were not
  included in the shared narrow-layout single-line role gate or the text-role matrix's derived-role
  catalog.
- `base_single_line_text_roles_stay_single_line_under_narrow_layout` now asserts both tabular
  control-readout variants stay one measured line under a narrow resize probe. This keeps numeric
  DataTable/page/footer readouts on the same anti-wrap contract as ordinary readouts, button
  labels, table cells, and code labels.
- `P3_TEXT_ROLE_MATRIX_2026-05-17.md` now records the variants as control-readout derivatives
  rather than a sixth stable role, and `tools/gate_imui_workstream_source.py` guards their
  narrow-layout assertions.
- Focused gates passed: `cargo nextest run -p fret-ui-kit --lib
  base_single_line_text_roles_stay_single_line_under_narrow_layout --no-fail-fast`; `cargo fmt
  --check -p fret-ui-kit`; `python tools\gate_imui_workstream_source.py`; `python -m py_compile
  tools\gate_imui_workstream_source.py`; `python -m json.tool
  docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`; and `git diff --check`.

2026-05-19 shadcn EmptyTitle children-role slice:

- Source gap before fix: `EmptyTitle` only accepted a string payload, so empty-state title slots
  could not carry caller-supplied shared title/chrome role children. That kept empty states outside
  the same explicit-role contract used by overlay, card, item, and field title slots.
- `EmptyTitle` now exposes `new_children(...)`. The string path preserves the existing shadcn
  empty-title defaults, including centered balanced text; the children path patches bare `Text`,
  `StyledText`, and `SelectableText` children with empty-title typography/foreground/centered
  balance fallback, but skips subtrees that already carry inherited text-role metadata.
- `empty_title_children_patch_rich_text_with_title_typography` proves the strong fallback remains;
  `empty_title_children_preserve_shared_text_role_contracts` proves a `text_chrome_title(...)`
  child keeps `style: None`, `color: None`, ellipsis overflow, and role metadata; and
  `empty_description_scopes_inherited_text_style` proves description scopes are unchanged.
- `tools/gate_imui_workstream_source.py` now guards the EmptyTitle children API, scoped helper, and
  tests.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  empty_title_children_patch_rich_text_with_title_typography
  empty_title_children_preserve_shared_text_role_contracts --no-fail-fast` failed because
  `EmptyTitle::new_children(...)` did not exist.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  empty_title_children_patch_rich_text_with_title_typography
  empty_title_children_preserve_shared_text_role_contracts
  empty_description_scopes_inherited_text_style --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn FieldTitle children-role slice:

- Source gap before fix: `FieldTitle` only accepted a string payload, so field/property-panel title
  slots could not carry caller-supplied shared title/chrome role children. That kept field titles
  outside the same explicit-role contract now used by dialog, sheet, popover, alert, card, and item
  titles.
- `FieldTitle` now exposes `new_children(...)`. The string path preserves existing field-title
  defaults and `w-fit` behavior; the children path patches bare `Text`, `StyledText`, and
  `SelectableText` children with field-title typography/foreground/alignment, but skips subtrees
  that already carry inherited text-role metadata.
- `field_title_children_patch_rich_text_with_title_typography` proves the strong fallback remains;
  `field_title_children_preserve_shared_text_role_contracts` proves a `text_chrome_title(...)`
  child keeps `style: None`, `color: None`, ellipsis overflow, role metadata, and its own
  fill-width layout; `field_title_and_plain_label_approximate_upstream_w_fit_defaults` keeps the
  existing bare-title `w-fit` contract in the focused run; and
  `field_description_scopes_inherited_text_style` proves description scopes are unchanged.
- `tools/gate_imui_workstream_source.py` now guards the FieldTitle children API, scoped helper, and
  tests.
- Focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  field_title_children_patch_rich_text_with_title_typography
  field_title_children_preserve_shared_text_role_contracts
  field_title_and_plain_label_approximate_upstream_w_fit_defaults
  field_description_scopes_inherited_text_style --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn PopoverTitle children-role slice:

- Source gap before fix: `PopoverTitle` only accepted a string payload, so popover/panel titles
  could not carry caller-supplied shared title/chrome role children. That left another overlay
  surface behind the title-slot contract now used by dialog, sheet, alert, card, and item titles.
- `PopoverTitle` now exposes `new_children(...)`. The string path preserves existing shadcn
  popover-title defaults; the children path patches bare `Text`, `StyledText`, and
  `SelectableText` children with popover-title typography/foreground, but skips subtrees that
  already carry inherited text-role metadata.
- `popover_title_children_patch_rich_text_with_title_typography` proves the strong fallback
  remains; `popover_title_children_preserve_shared_text_role_contracts` proves a
  `text_chrome_title(...)` child keeps `style: None`, `color: None`, ellipsis overflow, and role
  metadata; and `popover_description_scopes_inherited_text_style` stays in the focused run to prove
  the existing description scope is unchanged.
- `tools/gate_imui_workstream_source.py` now guards the PopoverTitle children API, scoped helper,
  and tests.
- Focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  popover_title_children_patch_rich_text_with_title_typography
  popover_title_children_preserve_shared_text_role_contracts
  popover_description_scopes_inherited_text_style --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn SheetTitle children-role slice:

- Source gap before fix: `SheetTitle` only accepted a string payload, so sheet titles could not
  carry caller-supplied shared title/chrome role children. That left sheet composition behind the
  dialog/title-slot contract now used by `DialogTitle`, `AlertDialogTitle`, `CardTitle`, and
  `AlertTitle`.
- `SheetTitle` now exposes `new_children(...)`. The string path preserves existing shadcn
  sheet-title defaults; the children path patches bare `Text`, `StyledText`, and `SelectableText`
  children with sheet-title typography/foreground, but skips subtrees that already carry inherited
  text-role metadata.
- `sheet_title_children_patch_rich_text_with_title_typography` proves the strong fallback remains;
  `sheet_title_children_preserve_shared_text_role_contracts` proves a `text_chrome_title(...)`
  child keeps `style: None`, `color: None`, ellipsis overflow, and role metadata; and
  `sheet_description_scopes_inherited_text_style` stays in the focused run to prove the existing
  description scope is unchanged.
- `tools/gate_imui_workstream_source.py` now guards the SheetTitle children API, scoped helper, and
  tests.
- Focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  sheet_title_children_patch_rich_text_with_title_typography
  sheet_title_children_preserve_shared_text_role_contracts
  sheet_description_scopes_inherited_text_style --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn DialogTitle children-role slice:

- Source gap before fix: `DialogTitle` only accepted a string payload, so callers could not provide
  a shared title/chrome role child at all. That made dialog titles an exception to the emerging
  component contract: recipes may own bare-text defaults, but caller-supplied role children must be
  expressible and protected.
- `DialogTitle` now mirrors the other title slots with `new_children(...)`. The text path preserves
  the existing shadcn dialog-title defaults; the children path patches bare `Text`, `StyledText`,
  and `SelectableText` children with dialog-title typography/foreground, but skips subtrees that
  already carry inherited text-role metadata.
- `dialog_title_children_patch_rich_text_with_title_typography` proves the strong fallback remains;
  `dialog_title_children_preserve_shared_text_role_contracts` proves a
  `text_chrome_title(...)` child keeps `style: None`, `color: None`, ellipsis overflow, role
  metadata, and heading semantics; `dialog_description_children_scope_inherited_text_style` stays
  in the focused run to prove the existing description scope is unchanged.
- `tools/gate_imui_workstream_source.py` now guards the DialogTitle children API, scoped helper, and
  tests.
- Focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  dialog_title_children_patch_rich_text_with_title_typography
  dialog_title_children_preserve_shared_text_role_contracts
  dialog_description_children_scope_inherited_text_style --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn AlertDialogTitle role-preservation slice:

- Source gap before fix: `AlertDialogTitle` recursively wrote dialog-title typography, foreground,
  wrapping, and clipping into descendant text leaves. That kept bare dialog titles ergonomic, but it
  also overwrote shared role children such as `text_chrome_title(...)`, even though those already
  carry role-owned title-family resize semantics.
- `patch_alert_dialog_text_style_recursive(...)` now delegates to a role-scope-aware helper. It
  still patches bare `Text`, `StyledText`, and `SelectableText` children so custom
  AlertDialogTitle content gets the shadcn dialog-title fallback, but it skips any subtree that
  already carries inherited text-role metadata.
- `alert_dialog_title_children_patch_rich_text_with_title_typography` keeps the strong title-slot
  fallback; `alert_dialog_description_children_scope_rich_text_with_description_typography` stays
  in the focused run to prove the existing description scope is unchanged; and
  `alert_dialog_title_children_preserve_shared_text_role_contracts` proves shared title-role
  children keep `style: None`, `color: None`, ellipsis overflow, and role metadata.
- `tools/gate_imui_workstream_source.py` now guards the AlertDialogTitle helper/test shape.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  alert_dialog_title_children_preserve_shared_text_role_contracts
  alert_dialog_title_children_patch_rich_text_with_title_typography
  alert_dialog_description_children_scope_rich_text_with_description_typography --no-fail-fast`
  failed because the role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  alert_dialog_title_children_preserve_shared_text_role_contracts
  alert_dialog_title_children_patch_rich_text_with_title_typography
  alert_dialog_description_children_scope_rich_text_with_description_typography --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn AlertTitle role-preservation slice:

- Source gap before fix: `AlertTitle` recursively wrote title typography into every descendant
  text leaf. That kept bare alert titles ergonomic, but it also overwrote shared role children such
  as `text_chrome_title(...)`, even though those already carry title-family single-line ellipsis
  semantics.
- `patch_alert_text_style_recursive(...)` now delegates to a role-scope-aware helper. It still
  patches bare `Text`, `StyledText`, and `SelectableText` children so custom AlertTitle content gets
  the shadcn alert-title fallback, but it skips any subtree that already carries inherited text-role
  metadata.
- `alert_title_children_patch_rich_text_with_title_typography` and
  `alert_title_children_preserve_interactive_spans_under_title_scope` keep the strong slot
  fallback; `alert_title_children_preserve_shared_text_role_contracts` proves shared title-role
  children keep `style: None`, ellipsis overflow, and role metadata.
- `tools/gate_imui_workstream_source.py` now guards the AlertTitle helper/test shape.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  alert_title_children_preserve_shared_text_role_contracts
  alert_title_children_patch_rich_text_with_title_typography
  alert_title_children_preserve_interactive_spans_under_title_scope --no-fail-fast` failed because
  the role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  alert_title_children_preserve_shared_text_role_contracts
  alert_title_children_patch_rich_text_with_title_typography
  alert_title_children_preserve_interactive_spans_under_title_scope --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn CardTitle role-preservation slice:

- Source gap before fix: `CardTitle::new_children(...)` intentionally patched child text leaves to
  apply the card-title slot contract. That is correct for bare text and ordinary rich text, but it
  also overwrote shared role children such as `text_chrome_title(...)`, even though those already
  carry title-family single-line ellipsis semantics.
- `patch_card_title_text_style_recursive(...)` now delegates to a role-scope-aware helper. It still
  patches bare `Text`, `StyledText`, and `SelectableText` children so custom CardTitle content gets
  the shadcn card-title fallback, but it skips any subtree that already carries inherited text-role
  metadata.
- `card_title_children_patch_bare_text_with_title_typography` and the existing
  `card_title_children_patch_rich_text_with_title_typography` cover the strong slot fallback;
  `card_title_children_preserve_shared_text_role_contracts` proves shared title-role children keep
  `style: None`, ellipsis overflow, and role metadata.
- `tools/gate_imui_workstream_source.py` now guards the CardTitle helper/test shape.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  card_title_children_patch_bare_text_with_title_typography
  card_title_children_preserve_shared_text_role_contracts
  card_title_children_patch_rich_text_with_title_typography --no-fail-fast` failed because the
  role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  card_title_children_patch_bare_text_with_title_typography
  card_title_children_preserve_shared_text_role_contracts
  card_title_children_patch_rich_text_with_title_typography --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-20 shadcn CardDescription children role-preservation gate:

- Source gap before gate: `CardDescription::new_children(...)` scoped description typography on the
  children container, but it did not have a focused regression proving caller-supplied shared
  description/body roles keep their leaf wrap/overflow and inherited role metadata.
- `card_description_children_preserve_shared_text_role_contracts` now proves a
  `text_compact_paragraph_line_clamp(...)` child keeps `style: None`, role metadata,
  word wrapping, and ellipsis overflow when mounted through `CardDescription::new_children(...)`.
- This stays in `fret-ui-shadcn` as recipe/slot policy. It does not add a new text role or widen
  `fret-imui`.
- First focused `cargo nextest run -p fret-ui-shadcn --lib
  card_description_children_preserve_shared_text_role_contracts --no-fail-fast` timed out while
  Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally, the
  same focused command passed.

2026-05-20 shadcn Sheet/Popover description children role-preservation gate:

- Source gap before gate: `SheetDescription` and `PopoverDescription` were text-only leaves, so
  callers could not hand a composed shared paragraph role tree into the description slot without
  wrapping around the component boundary.
- `sheet_description_children_preserve_shared_text_role_contracts` and
  `popover_description_children_preserve_shared_text_role_contracts` now prove a
  `text_compact_paragraph(...)` child keeps `style: None`, wrap, fill-width layout, and inherited
  role metadata when mounted through the description slots.
- This widens the shadcn recipe/API surface only where the component already had a description slot;
  it does not add a new `fret-imui` text role or move policy into `fret-imui`.
- Focused `cargo nextest run -p fret-ui-shadcn --lib
  sheet_description_children_preserve_shared_text_role_contracts
  popover_description_children_preserve_shared_text_role_contracts --no-fail-fast` passed after the
  new children lanes landed.

2026-05-20 shadcn existing description children role-preservation gate:

- Source gap before gate: `AlertDescription`, `DialogDescription`, `AlertDialogDescription`, and
  `ItemDescription` already exposed `new_children(...)`, but their tests only covered inherited
  typography, rich/selectable text, or container layout. They did not prove shared paragraph-role
  children kept their own wrap/layout/overflow metadata under description composition.
- `alert_description_children_preserve_shared_text_role_contracts`,
  `dialog_description_children_preserve_shared_text_role_contracts`,
  `alert_dialog_description_children_preserve_shared_text_role_contracts`, and
  `item_description_children_preserve_shared_text_role_contracts` now prove shared paragraph/body
  roles keep `style: None`, `color: None`, role-owned wrapping/overflow, fill-width/min-width
  layout, and inherited role metadata when passed through the existing description children lanes.
- This is a gate-only contract slice for existing shadcn recipe surfaces. It does not add a new text
  role, widen `fret-imui`, or change description rendering policy.
- Focused `cargo nextest run -p fret-ui-shadcn --lib
  alert_description_children_preserve_shared_text_role_contracts
  dialog_description_children_preserve_shared_text_role_contracts
  alert_dialog_description_children_preserve_shared_text_role_contracts
  item_description_children_preserve_shared_text_role_contracts --no-fail-fast` passed.

2026-05-20 shadcn ButtonGroupText children role-preservation gate:

- Source gap before gate: `ButtonGroupText::new_children(...)` already passed custom children
  through, but the test suite only proved inline child count/layout. It did not prove a
  caller-supplied shared button-label role kept its own single-line shrink/ellipsis contract under
  button-group chrome composition.
- `button_group_text_children_preserve_shared_button_label_role_contracts` now proves a
  `text_button_label(...)` child keeps `style: None`, `color: None`, no wrapping, ellipsis
  overflow, zero minimum width, flex shrink, and inherited role metadata when mounted through
  `ButtonGroupText::new_children(...)`.
- This is a gate-only contract slice for an existing shadcn recipe slot. Component-owned
  `ButtonGroupText::new(...)` label styling remains local chrome policy, and this does not add a
  new text role or widen `fret-imui`.
- First focused `cargo nextest run -p fret-ui-shadcn --lib
  button_group_text_children_preserve_shared_button_label_role_contracts --no-fail-fast` timed out
  while Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally,
  the same focused command passed.
- Focused `cargo nextest run -p fret-ui-shadcn --lib
  button_group_text_children_preserve_shared_button_label_role_contracts --no-fail-fast` passed.

2026-05-20 shadcn TabsTrigger role-preservation slice:

- Source gap before fix: `TabsTrigger` recursively wrote trigger typography and foreground into
  descendant passive text leaves whenever `style` / `color` were empty. Shared text role helpers
  intentionally keep leaf `style` and `color` empty and carry typography through
  `inherited_text_style`, so trigger children built with `text_button_label(...)` lost their
  role-owned single-line shrink/ellipsis contract.
- `apply_trigger_inherited_style(...)` now treats `inherited_text_style` as a protected role scope
  and carries that scope through descendants. Bare trigger text still receives the shadcn trigger
  typography/foreground fallback; shared role children keep leaf `style: None`, `color: None`,
  no-wrap, ellipsis overflow, zero minimum width, shrink, and inherited role metadata. Trigger
  foreground still flows through the content root's inherited foreground instead of being stamped
  onto role-owned text leaves.
- `tabs_trigger_applies_default_style_to_bare_label_text` proves the bare-text fallback remains;
  `tabs_trigger_children_preserve_shared_button_label_role_contracts` proves a shared button-label
  role survives `TabsItem::trigger_children(...)`.
- First focused `cargo nextest run -p fret-ui-shadcn --lib
  tabs_trigger_applies_default_style_to_bare_label_text
  tabs_trigger_children_preserve_shared_button_label_role_contracts --no-fail-fast` timed out while
  Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally, the
  same focused command passed.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  tabs_trigger_applies_default_style_to_bare_label_text
  tabs_trigger_children_preserve_shared_button_label_role_contracts --no-fail-fast`.

2026-05-20 shadcn Toggle/ToggleGroup role-preservation slice:

- Source gap before fix: `Toggle` and `ToggleGroupItem` recursively wrote their resolved
  foreground into descendant passive text leaves. That kept bare custom text ergonomic, but it
  overwrote shared text role children such as `text_button_label(...)`, replacing the role-owned
  leaf color/inherited typography path that controls single-line shrink/ellipsis under narrow
  editor chrome.
- `apply_toggle_inherited_style(...)` and `apply_item_inherited_style(...)` now treat
  `inherited_text_style` as a protected role scope and propagate that scope through descendants.
  Bare text children still receive toggle/toggle-group foreground; shared role children keep
  `style: None`, `color: None`, no-wrap, ellipsis overflow, zero minimum width, shrink, and
  inherited role metadata. Foreground still flows through the content root's inherited foreground.
- `toggle_children_apply_foreground_to_bare_text` and
  `toggle_group_item_children_apply_foreground_to_bare_text` prove the bare-text fallback remains.
  `toggle_children_preserve_shared_button_label_role_contracts` and
  `toggle_group_item_children_preserve_shared_button_label_role_contracts` prove shared
  button-label roles survive explicit toggle/toggle-group children.
- Two early focused runs timed out while Cargo/Rustc was still compiling. No process was killed;
  after Cargo/Rustc exited naturally, the split focused runs passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  toggle_children_apply_foreground_to_bare_text
  toggle_children_preserve_shared_button_label_role_contracts --no-fail-fast` and
  `cargo nextest run -p fret-ui-shadcn --lib
  toggle_group_item_children_apply_foreground_to_bare_text
  toggle_group_item_children_preserve_shared_button_label_role_contracts --no-fail-fast`.

2026-05-20 shadcn Badge role-preservation slice:

- Source gap before fix: `Badge` recursively wrote its resolved variant foreground into leading
  and trailing child text leaves. That preserved color inheritance for bare text, but it overwrote
  shared text role children such as `text_button_label(...)`, replacing the role-owned leaf color
  path that controls single-line shrink/ellipsis in dense badge chrome.
- `apply_badge_inherited_fg(...)` now treats `inherited_text_style` as a protected role scope and
  carries that scope through descendants. Bare child text still receives the badge foreground; shared
  role children keep `style: None`, `color: None`, no-wrap, ellipsis overflow, zero minimum width,
  shrink, and inherited role metadata. Badge foreground still flows through the content root's
  inherited foreground.
- `badge_children_apply_foreground_to_bare_text` proves the bare-text fallback remains, and
  `badge_children_preserve_shared_button_label_role_contracts` proves a shared button-label role
  survives `Badge::leading_children(...)`.
- Early focused runs timed out while Cargo/Rustc was still compiling. No process was killed; after
  Cargo/Rustc exited naturally, the split focused runs passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  badge_children_apply_foreground_to_bare_text --no-fail-fast` and
  `cargo nextest run -p fret-ui-shadcn --lib
  badge_children_preserve_shared_button_label_role_contracts --no-fail-fast`.

2026-05-20 shadcn Button children role-preservation gate:

- Source gap before gate: `Button` already used `current_color::scope_children(...)` for custom
  content instead of recursively rewriting text leaves, but the test suite did not prove
  caller-supplied shared button-label roles kept their leaf text contract through the full
  `children(...)` override path or the `leading_children(...)` / `trailing_children(...)` inline
  slot path.
- `button_children_preserve_shared_button_label_role_contracts` now proves a `text_button_label(...)`
  child keeps `style: None`, `color: None`, no-wrap, ellipsis overflow, zero minimum width, shrink,
  inherited role metadata, and inherited foreground through `Button::children(...)`.
- `button_inline_children_preserve_shared_button_label_role_contracts` proves the same contract for
  `Button::leading_children(...)` and `Button::trailing_children(...)`, while the existing inline
  ordering test continues to cover label preservation and RTL order flipping.
- First focused `cargo nextest run -p fret-ui-shadcn --lib
  button_children_preserve_shared_button_label_role_contracts
  button_inline_children_preserve_shared_button_label_role_contracts --no-fail-fast` timed out while
  Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally, the
  split focused runs passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  button_children_preserve_shared_button_label_role_contracts --no-fail-fast` and
  `cargo nextest run -p fret-ui-shadcn --lib
  button_inline_children_preserve_shared_button_label_role_contracts --no-fail-fast`.

2026-05-20 shadcn TooltipContent role-preservation slice:

- Source gap before fix: `TooltipContent` recursively wrote tooltip foreground, tooltip text style,
  and max-width/min-width hints into descendant text leaves whenever those leaf props were empty.
  Shared text-role helpers intentionally keep leaf `style` and `color` empty while carrying
  typography through `inherited_text_style`, so rich tooltip content built with
  `text_control_readout(...)` could lose its role-owned single-line shrink/ellipsis contract under
  narrow overlay chrome.
- `apply_tooltip_inherited_defaults(...)` now delegates to a scoped helper that treats
  `inherited_text_style` as a protected role scope and propagates that scope through descendants.
  Bare tooltip text still receives tooltip `text-xs`, foreground, and max-width defaults; shared
  role children keep `style: None`, `color: None`, no-wrap, ellipsis overflow, zero minimum width,
  shrink, and inherited role metadata.
- `TooltipContent` now stamps tooltip foreground as inherited foreground on the content root. This
  preserves shadcn `text-background` behavior for shared text roles without mutating role-owned text
  leaves.
- `tooltip_content_applies_default_style_to_bare_text` proves the bare-text fallback remains.
  `tooltip_content_preserves_shared_control_readout_role_contracts` proves a shared control-readout
  role survives `TooltipContent::new(...)` rich content.
- First combined focused runs timed out while Cargo/Rustc was still compiling. No process was
  killed; after Cargo/Rustc exited naturally, the combined focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  tooltip_content_applies_default_style_to_bare_text
  tooltip_content_preserves_shared_control_readout_role_contracts --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.

2026-05-20 shadcn BreadcrumbList role-preservation slice:

- Source gap before fix: primitive `BreadcrumbList` applied list-level muted foreground and
  breadcrumb typography by writing direct `Text` leaf props for each custom child. Shared text-role
  helpers intentionally keep leaf `style` and `color` empty while carrying typography through
  `inherited_text_style`, so a primitive breadcrumb list child built with `text_button_label(...)`
  could lose its role-owned single-line shrink/ellipsis contract under breadcrumb composition.
- `apply_breadcrumb_list_text_style_defaults(...)` now treats `inherited_text_style` as a protected
  role scope and only applies breadcrumb typography to bare text leaves. `BreadcrumbList` now stamps
  the muted foreground through `current_color::scope_children(...)`, so visual color remains
  breadcrumb-list policy without mutating role-owned text leaves.
- `breadcrumb_list_applies_default_style_to_bare_text` proves the bare loose-text fallback remains.
  `breadcrumb_list_preserves_shared_button_label_role_contracts` proves a shared button-label role
  survives primitive `BreadcrumbList::into_element(...)` children.
- First focused run timed out while Cargo/Rustc was still compiling. No process was killed; after
  Cargo/Rustc exited naturally, the combined focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  breadcrumb_list_applies_default_style_to_bare_text
  breadcrumb_list_preserves_shared_button_label_role_contracts --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.

2026-05-20 shadcn AnnouncementTitle role-preservation slice:

- Source gap before fix: raw extras `AnnouncementTitle` owned the correct clipped title container
  and bare-text single-line ellipsis fallback, but its recursive title contract rewrote every
  descendant passive text leaf. Shared text-role helpers intentionally keep leaf `style` and
  `color` empty while carrying typography through `inherited_text_style`, so a custom title child
  built with `text_button_label(...)` could lose its role-owned leaf contract under resize.
- `apply_announcement_title_text_contract_recursive(...)` now delegates to a scoped helper that
  treats `inherited_text_style` as a protected role scope. Title typography is applied to bare text
  leaves instead of the title root, so shared role children are not polluted by parent inherited
  style merging. Bare `cx.text(...)` title children still receive the announcement-title
  single-line ellipsis contract, while shared role children keep `style: None`, `color: None`,
  no-wrap, ellipsis overflow, zero minimum width, shrink, and inherited role metadata.
- `announcement_title_keeps_composable_children_on_truncated_title_contract` proves the existing
  bare/composable title contract remains. `announcement_title_preserves_shared_button_label_role_contracts`
  proves a shared button-label role survives `AnnouncementTitle::new(...)` children.
- First focused `cargo nextest run -p fret-ui-shadcn --lib
  announcement_title_keeps_composable_children_on_truncated_title_contract
  announcement_title_preserves_shared_button_label_role_contracts --no-fail-fast` timed out while
  Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally, the
  same focused command passed.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-20 shadcn SidebarGroupLabel resize text-role slice:

- Source gap before fix: `SidebarGroupLabel` rendered its default fixed-height label with local
  `ui::text(...).text_size_px(...).line_height_px(...).font_medium().text_color(...).wrap(TextWrap::Word)`.
  The outer chrome row is fixed at 32px, so narrow sidebars could wrap group labels into multiple
  lines and let text exceed the row bottom, matching the resize failure mode reported in the UI.
- `text_menu_group_label(...)` now carries the complete shadcn menu/group label role:
  `text-xs font-medium`, fill-width, shrink, zero minimum width, no-wrap, and ellipsis overflow.
  It remains muted by default for menu/select/command surfaces.
- `SidebarGroupLabel` now consumes `decl_text::text_menu_group_label(...)` for its default label
  while overriding inherited foreground with the sidebar-specific 70% foreground. The `as_child`
  custom-child path remains unchanged.
- `menu_group_label_text_uses_muted_medium_xs_single_line_truncation` proves the shared role
  semantics. `sidebar_group_label_uses_shared_menu_group_text_role` proves the fixed sidebar label
  uses that role and keeps sidebar foreground ownership without leaf style/color writes.
- `cargo nextest run -p fret-ui-kit --lib
  menu_group_label_text_uses_muted_medium_xs_single_line_truncation --no-fail-fast` passed.
- First `cargo nextest run -p fret-ui-shadcn --lib
  sidebar_group_label_uses_shared_menu_group_text_role --no-fail-fast` timed out at 304s while
  Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally, the
  same focused command was rerun with a longer timeout and passed.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.

2026-05-20 shadcn SidebarMenuBadge compact tabular readout slice:

- Source gap before fix: `SidebarMenuBadge` is an upstream fixed `h-5 min-w-5 text-xs font-medium
  tabular-nums` counter slot, but the Fret port still rendered it with sidebar-local
  `ui::text(...).text_size_px(...).line_height_px(...).font_medium().text_color(...).nowrap()`.
  That duplicated text policy in the sidebar recipe and missed the shared readout role's
  shrink/min-width-0/ellipsis resize contract.
- `text_control_readout_compact_tabular_emphasis(...)` now exists as a derived control-readout
  role, not a new base role. It applies inherited `text-xs`, medium weight, `tnum`, no-wrap,
  shrink, zero minimum width, and ellipsis overflow for fixed badge/counter slots.
- `SidebarMenuBadge` now consumes that derived readout role and overrides inherited foreground with
  `sidebar.foreground`, while keeping the badge chrome (`h-5`, `min-w-5`, padding, rounded, inline
  end placement) recipe-owned.
- `compact_tabular_emphasis_readout_uses_xs_medium_single_line_truncation` proves the derived role.
  `sidebar_menu_badge_uses_shared_compact_tabular_readout_role` proves the sidebar badge no longer
  writes leaf style/color and keeps upstream `text-xs font-medium tabular-nums` semantics through
  inherited text metadata.
- `cargo nextest run -p fret-ui-kit --lib
  compact_tabular_emphasis_readout_uses_xs_medium_single_line_truncation
  base_single_line_text_roles_stay_single_line_under_narrow_layout --no-fail-fast` passed.
- `cargo nextest run -p fret-ui-shadcn --lib
  sidebar_menu_badge_uses_shared_compact_tabular_readout_role --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn` passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.

2026-05-20 shadcn SidebarMenuButton/SubButton fill button-label slice:

- Source gap before fix: default `SidebarMenuButton` and `SidebarMenuSubButton` labels lived in
  fixed-height sidebar rows but still hand-rolled local `ui::text(...).w_full().min_w_0().flex_1()
  .basis_0().text_size_px(...).font_weight(...).line_height_px(...).truncate()` policy. The
  collapsed sidebar tooltip label also used local `wrap(TextWrap::Word)` / `TextOverflow::Clip`.
  That duplicated role policy in the sidebar recipe and left the small-row `text-xs` shadcn axis
  outside the shared button-label family.
- `text_button_label_fill(...)` and `text_button_label_compact_fill(...)` now exist as derived
  button-label roles. They add fill/grow/basis-zero layout to the button-label resize contract,
  while the compact variant owns `text-xs font-medium` for small trigger rows.
- `SidebarMenuButton` and `SidebarMenuSubButton` default labels now consume those derived roles
  through a small sidebar helper and inherit sidebar foreground. `size=sm` maps to the compact
  role, default/lg and md rows map to the regular fill role, and collapsed tooltip labels use
  `text_button_label(...)` instead of local wrapping text. Sidebar still owns row chrome,
  foreground state, collapse opacity, RTL ordering, and tooltip placement.
- `fill_button_label_text_uses_growing_single_line_truncation` and
  `compact_fill_button_label_text_uses_xs_growing_single_line_truncation` prove the derived role
  contracts. `sidebar_menu_button_label_uses_shared_fill_button_role` and
  `sidebar_menu_sub_button_label_uses_shared_fill_button_role` prove sidebar default labels keep
  fill/grow/min-width-0/no-wrap/ellipsis semantics and the correct `text-sm` vs `text-xs` inherited
  typography.
- `cargo nextest run -p fret-ui-kit --lib
  fill_button_label_text_uses_growing_single_line_truncation
  compact_fill_button_label_text_uses_xs_growing_single_line_truncation
  base_single_line_text_roles_stay_single_line_under_narrow_layout --no-fail-fast` passed.
- First `cargo nextest run -p fret-ui-shadcn --lib
  sidebar_menu_button_label_uses_shared_fill_button_role
  sidebar_menu_sub_button_label_uses_shared_fill_button_role
  sidebar_menu_button_default_content_reorders_in_rtl
  sidebar_menu_sub_button_default_content_reorders_in_rtl --no-fail-fast` timed out while Cargo was
  still compiling. No process was killed; after Cargo exited naturally the same focused command was
  rerun. The first rerun exposed the expected test update for sub-button row signatures after the
  label became a shared role, and the post-fix run passed.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn` passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.

2026-05-20 inherited-axis + shadcn Button default-label role slice:

- Source gap before fix: shadcn `Button` default labels still used a local
  `ui::text(...).text_size_px(...).fixed_line_box_px(...).nowrap().text_color(...)` builder inside
  fixed-height button chrome. Migrating that path directly to `text_button_label(...)` exposed a
  mechanism gap: `TextStyleRefinement` carried OpenType features but not variable font axes, so
  existing `Button::label_font_axis(...)` could not stay on the inherited-role path.
- `TextStyleRefinement` now carries variable font axes as subtree defaults. Merge/refine,
  passive-text measurement, cache fingerprints, and `fret-ui-kit` typography refinements all carry
  axes alongside features.
- `Button` default labels now render through a small helper backed by
  `decl_text::text_button_label(...)`. Button-owned font, feature, axis, explicit weight,
  foreground, and label `test_id` suffix behavior are layered through inherited text/foreground
  metadata instead of leaf-local style/color. The default text size follows the shared shadcn
  `text-sm font-medium whitespace-nowrap` baseline, with the existing Fret `xs/icon-xs` extension
  mapped to the shared `text-xs` preset.
- Focused proof:
  `text_style_refinement_merges_font_axes_in_parent_child_order`,
  `text_style_refine_applies_inherited_font_axes_after_leaf_defaults`,
  `inherited_text_style_axes_affect_passive_text_measurement`,
  `inherited_text_style_fingerprint_tracks_axis_overrides`,
  `composable_refinement_keeps_font_features_and_axes`,
  `button_default_label_uses_shared_button_label_role`,
  `button_default_label_keeps_font_feature_and_axis_overrides_on_role`, and
  `button_default_label_keeps_weight_override_on_role`.

2026-05-20 shadcn CalendarDayButton shared text-role slice:

- Source gap before fix: `CalendarDayButton` day numbers and optional supporting text rendered with
  Calendar-local `ui::label(...).text_size_px(...).line_height_px(...).text_color(...).nowrap()`
  builders inside fixed-size day cells. That duplicated the single-line resize contract in the
  calendar recipe and left the same day-cell text policy outside the shared role/gate vocabulary.
- `calendar_day_label_text(...)` now backs day numbers with `decl_text::text_button_label(...)`,
  layered with Calendar-owned normal weight, foreground, and center alignment through inherited
  text/foreground metadata. `calendar_day_supporting_text(...)` backs optional supporting text with
  `decl_text::text_control_readout(...)`, keeping the auxiliary value in the readout family while
  preserving Calendar-owned center alignment and opacity behavior.
- `Calendar` and `CalendarRange` both feed the same `day_text_style` into
  `calendar_day_button_children(...)`, so single-date and range day cells share the role contract.
  Calendar still owns day-cell chrome, selected/today/range foregrounds, disabled opacity, and
  test-id/ARIA semantics.
- `calendar_day_button_text_uses_shared_roles` proves day and supporting text leaves keep
  `style: None`, `color: None`, fill width, shrink, `min-width: 0`, no-wrap, ellipsis, center
  alignment, inherited text style, and inherited foreground. Existing supporting-text visibility
  tests still prove out-of-month supporting text remains absent.
- Focused gates passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  calendar_day_button_text_uses_shared_roles
  calendar_day_button_supporting_text_renders_only_for_in_month_days --no-fail-fast`,
  `cargo nextest run -p fret-ui-shadcn --lib calendar_range --no-fail-fast`, and
  `cargo check -p fret-ui-shadcn --lib`.

2026-05-20 shadcn menu item label shared text-role slice:

- Source gap before fix: DropdownMenu, ContextMenu, and Menubar overlay item labels still rendered
  local `ui::text(label).text_size_px(...).font_weight(...).nowrap()` leaves inside fixed menu rows.
  That duplicated the resize contract already assigned to `text_list_row_label(...)`, and it made
  menu rows another place where text policy could drift from the IMUI/editor row vocabulary.
- `text_list_row_label(...)` and its attributed variant now use the same fill/grow/basis-zero
  single-line layout expected by dense row labels. The existing tests now assert grow and zero
  basis instead of only checking fill/shrink/min-width-0.
- `menu_text::menu_item_label(...)` backs shadcn menu-family item labels with
  `decl_text::text_list_row_label(...)` and layers menu-owned resolved typography plus foreground
  through inherited text/foreground metadata. DropdownMenu, ContextMenu, and Menubar overlay rows
  consume that helper. Menubar's top-level trigger remains out of this slice because it is
  button-like trigger text, not an overlay item row.
- DropdownMenu no longer wraps entire rows in icon `currentColor` after labels moved to inherited
  foreground. Icon/custom/trailing subtrees still receive icon foreground, while label foreground
  remains the menu item's resolved state foreground.
- Focused gates passed:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  list_row_label_text_uses_fill_width_single_line_truncation
  attributed_list_row_label_text_uses_fill_width_single_line_truncation --no-fail-fast`,
  `cargo nextest run -p fret-ui-shadcn --lib
  menu_item_label_uses_shared_list_row_role_with_menu_refinement
  dropdown_menu_label_element_uses_shared_menu_group_text_role
  context_menu_label_element_uses_shared_menu_group_text_role
  menubar_label_element_uses_shared_menu_group_text_role --no-fail-fast`, and
  `cargo check -p fret-ui-shadcn --lib`.
- Source/format gates passed: `python -m py_compile tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_facade_teaching_source.py`,
  `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn`, and `git diff --check`.

2026-05-20 shadcn CalendarMultiple shared text-role slice:

- Source gap before fix: `CalendarMultiple` still rendered multiple-selection day numbers with a
  local `ui::label(day_text).text_size_px(...).line_height_px(...).font_medium().w_full()
  .text_align(...).text_color(...).nowrap()` builder, even though `Calendar` and `CalendarRange`
  had already moved day-cell text into the shared button-label/readout role helper.
- `CalendarMultiple` now reuses `calendar_day_button_children(...)` with the same normal-weight
  `day_text_style` as the single-date/range calendar day cells. Multiple selection still owns
  fixed cell chrome, selected/today foregrounds, disabled opacity, and selection updates; the day
  number text contract is no longer component-local.
- `calendar_multiple_day_text_uses_shared_role` proves multiple calendar day text keeps leaf
  `style: None`, `color: None`, fill width, shrink, `min-width: 0`, no-wrap, ellipsis, center
  alignment, inherited text style, and inherited foreground.
- `tools/gate_imui_workstream_source.py` now requires the CalendarMultiple helper/test shape and
  forbids the old local `ui::label(day_text.clone())` fixed-style block from returning.
- Focused gates passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  calendar_multiple_day_text_uses_shared_role
  calendar_multiple_nav_buttons_render_svg_icons --no-fail-fast`,
  `cargo check -p fret-ui-shadcn --lib`,
  `cargo fmt --check -p fret-ui-shadcn`,
  `python -m py_compile tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_facade_teaching_source.py`, and `git diff --check`.

2026-05-19 shadcn NavigationMenuLink role-preservation slice:

- Source gap before fix: `NavigationMenuLink` recursively wrote link typography and foreground
  directly into descendant text leaves. That kept bare link text ergonomic, but it overwrote shared
  role children such as `text_button_label(...)`, replacing the role-owned typography path that
  controls single-line ellipsis under narrow navigation content.
- `apply_link_inherited_style(...)` now treats `inherited_text_style` as a protected role scope and
  only writes link default typography into bare text. Link foreground is stamped as inherited
  foreground on the child root, so visual state remains link-owned without mutating text-role leaf
  props. The existing default-icon behavior remains intact: default icons opt out of inheriting the
  link foreground.
- `navigation_menu_link_applies_default_style_to_bare_text` proves the fallback still applies to
  bare text; `navigation_menu_link_preserves_shared_text_role_contracts` proves shared button-label
  text keeps `style: None`, ellipsis overflow, and role metadata; and the existing
  `navigation_menu_link_default_icons_do_not_inherit_current_color` test stays in the focused gate.
- `tools/gate_imui_workstream_source.py` now guards the NavigationMenuLink helper/test shape and
  forbids the old leaf foreground write from returning.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  navigation_menu_link_applies_default_style_to_bare_text
  navigation_menu_link_preserves_shared_text_role_contracts --no-fail-fast` failed because the
  role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  navigation_menu_link_applies_default_style_to_bare_text
  navigation_menu_link_preserves_shared_text_role_contracts
  navigation_menu_link_default_icons_do_not_inherit_current_color --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn DataTable role-preservation slice:

- Source gap before fix: `DataTable` wrapped body-cell renderers with
  `apply_default_text_style_recursive(...)`, which recursively wrote a table `TextStyle` into leaf
  text when `props.style` was empty. Shared text-role helpers intentionally keep leaf `style` empty
  and carry typography through `inherited_text_style`, so body cells built with
  `text_table_cell(...)` could lose their role-owned typography/overflow contract.
- `apply_default_text_style_recursive(...)` now delegates to a scoped helper that treats
  `inherited_text_style` as a protected role scope. Bare body text still receives the default table
  style; caller-supplied shared text roles retain their leaf style, wrap, overflow, and role
  metadata.
- `data_table_default_text_style_applies_to_bare_body_text` proves the ergonomic fallback remains;
  `data_table_default_text_style_preserves_shared_text_role_contracts` proves shared table-cell
  text roles survive the wrapper.
- `tools/gate_imui_workstream_source.py` now guards the DataTable helper/test shape.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  data_table_default_text_style_applies_to_bare_body_text
  data_table_default_text_style_preserves_shared_text_role_contracts --no-fail-fast` failed because
  the role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  data_table_default_text_style_applies_to_bare_body_text
  data_table_default_text_style_preserves_shared_text_role_contracts --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.
