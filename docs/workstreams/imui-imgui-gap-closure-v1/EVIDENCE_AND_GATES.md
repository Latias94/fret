# ImUi Dear ImGui Gap Closure v1 - Evidence & Gates

Status: Active
Last updated: 2026-05-17

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
  follow-ons at this point.
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
- 2026-05-17: introduced `text_chrome_title(...)` as the shared fill-width chrome title role and
  routed floating window title-bar text through shared chrome text helpers. Resizable floating
  titles keep fill, grow, shrink, `min-width: 0`, and ellipsis behavior; non-resizable titles reuse
  `text_section_chrome_label(...)` instead of local `TextProps`. Gate: `cargo nextest run -p
  fret-ui-kit --features imui --lib chrome_title_text_uses_fill_width_single_line_truncation
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
- 2026-05-17: migrated the `workspace_shell_demo` editor rail to teach the shared text roles too.
  Rail command buttons now use `text_button_label(...)`, property labels use `row_cx.label_text(...)`,
  and compact property values route through `text_control_readout(...)` via a local proof helper.
  Gates: `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface
  --no-fail-fast`, `cargo check -p fret-demo --bin workspace_shell_demo`, and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: migrated the `editor_notes_demo` inspector metadata surface to the same resize-safe
  text roles. Property-grid labels now use `row_cx.label_text(...)`, inspector subtitle and compact
  committed/outcome/draft/summary status values route through a local
  `editor_notes_readout_text(...)` helper backed by `text_control_readout(...)`, and the
  `editor_notes_editor_rail_surface` test plus the IMUI workstream source gate reject those fixed
  property-row labels/readouts drifting back to bare `cx.text(...)`. Gates:
  `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --no-fail-fast`,
  `cargo check -p fret-demo --bin editor_notes_demo`, and
  `python tools/gate_imui_workstream_source.py`.
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
