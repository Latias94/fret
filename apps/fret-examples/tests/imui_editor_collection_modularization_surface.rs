#[path = "imui_editor_collection_modularization_surface/asset_grid.rs"]
mod asset_grid;
#[path = "imui_editor_collection_modularization_surface/assets.rs"]
mod assets;
#[path = "imui_editor_collection_modularization_surface/box_select.rs"]
mod box_select;
#[path = "imui_editor_collection_modularization_surface/browser_input_runtime.rs"]
mod browser_input_runtime;
#[path = "imui_editor_collection_modularization_surface/browser_scope.rs"]
mod browser_scope;
#[path = "imui_editor_collection_modularization_surface/child_models.rs"]
mod child_models;
#[path = "imui_editor_collection_modularization_surface/command_buttons.rs"]
mod command_buttons;
#[path = "imui_editor_collection_modularization_surface/context_menu.rs"]
mod context_menu;
#[path = "imui_editor_collection_modularization_surface/drag_drop.rs"]
mod drag_drop;
#[path = "imui_editor_collection_modularization_surface/geometry.rs"]
mod geometry;
#[path = "imui_editor_collection_modularization_surface/import_target.rs"]
mod import_target;
#[path = "imui_editor_collection_modularization_surface/keyboard.rs"]
mod keyboard;
#[path = "imui_editor_collection_modularization_surface/lifecycle.rs"]
mod lifecycle;
#[path = "imui_editor_collection_modularization_surface/models.rs"]
mod models;
#[path = "imui_editor_collection_modularization_surface/order_toggle.rs"]
mod order_toggle;
#[path = "imui_editor_collection_modularization_surface/rename.rs"]
mod rename;
#[path = "imui_editor_collection_modularization_surface/render_states.rs"]
mod render_states;
#[path = "imui_editor_collection_modularization_surface/runtime_state.rs"]
mod runtime_state;
#[path = "imui_editor_collection_modularization_surface/selection.rs"]
mod selection;
#[path = "imui_editor_collection_modularization_surface/selection_commands.rs"]
mod selection_commands;
#[path = "imui_editor_collection_modularization_surface/selection_context_menu.rs"]
mod selection_context_menu;
#[path = "imui_editor_collection_modularization_surface/selection_delete.rs"]
mod selection_delete;
#[path = "imui_editor_collection_modularization_surface/selection_duplicate.rs"]
mod selection_duplicate;
#[path = "imui_editor_collection_modularization_surface/selection_duplicate_naming.rs"]
mod selection_duplicate_naming;
#[path = "imui_editor_collection_modularization_surface/selection_duplicate_selection.rs"]
mod selection_duplicate_selection;
#[path = "imui_editor_collection_modularization_surface/selection_keyboard.rs"]
mod selection_keyboard;
#[path = "imui_editor_collection_modularization_surface/selection_select_all.rs"]
mod selection_select_all;
#[path = "imui_editor_collection_modularization_surface/status_readouts.rs"]
mod status_readouts;

#[test]
fn imui_editor_proof_demo_routes_collection_proof_through_demo_local_module() {
    let demo_source = include_str!("../src/imui_editor_proof_demo.rs");
    let authoring_parity_source = include_str!("../src/imui_editor_proof_demo/authoring_parity.rs");
    let authoring_parity_models_source =
        include_str!("../src/imui_editor_proof_demo/authoring_parity/models.rs");
    let authoring_parity_surface_source =
        include_str!("../src/imui_editor_proof_demo/authoring_parity/surface.rs");
    let editor_state_source = include_str!("../src/imui_editor_proof_demo/editor_state.rs");
    let editor_advanced_source = include_str!("../src/imui_editor_proof_demo/editor_advanced.rs");
    let editor_gradient_source = include_str!("../src/imui_editor_proof_demo/editor_gradient.rs");
    let editor_inspector_source = include_str!("../src/imui_editor_proof_demo/editor_inspector.rs");
    let editor_material_source = include_str!("../src/imui_editor_proof_demo/editor_material.rs");
    let editor_object_source = include_str!("../src/imui_editor_proof_demo/editor_object.rs");
    let editor_text_assist_source =
        include_str!("../src/imui_editor_proof_demo/editor_text_assist.rs");
    let collection_source = include_str!("../src/imui_editor_proof_demo/collection.rs");
    let asset_grid_source = include_str!("../src/imui_editor_proof_demo/collection/asset_grid.rs");
    let asset_grid_tile_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/tile.rs");
    let asset_grid_actions_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/actions.rs");
    let asset_grid_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/chrome.rs");
    let asset_grid_inline_rename_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/inline_rename.rs");
    let asset_grid_inline_rename_actions_source = include_str!(
        "../src/imui_editor_proof_demo/collection/asset_grid/inline_rename/actions.rs"
    );
    let asset_grid_metadata_source =
        include_str!("../src/imui_editor_proof_demo/collection/asset_grid/metadata.rs");
    let assets_source = include_str!("../src/imui_editor_proof_demo/collection/assets.rs");
    let browser_scope_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope.rs");
    let browser_scope_asset_grid_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/asset_grid.rs");
    let browser_scope_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/chrome.rs");
    let child_models_source =
        include_str!("../src/imui_editor_proof_demo/collection/child_models.rs");
    let chrome_source = include_str!("../src/imui_editor_proof_demo/collection/chrome.rs");
    let browser_input_runtime_source =
        include_str!("../src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs");
    let browser_input_box_select_runtime_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select.rs"
    );
    let browser_input_box_select_session_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session.rs"
    );
    let browser_input_box_select_session_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session/tests.rs"
    );
    let browser_input_box_select_session_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/box_select/session/tests/fixtures.rs"
    );
    let browser_input_context_menu_runtime_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu.rs"
    );
    let browser_input_context_menu_runtime_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu/tests.rs"
    );
    let browser_input_context_menu_runtime_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/context_menu/tests/fixtures.rs"
    );
    let browser_input_zoom_runtime_source = include_str!(
        "../src/imui_editor_proof_demo/collection/browser_scope/input_runtime/zoom.rs"
    );
    let box_select_source = include_str!("../src/imui_editor_proof_demo/collection/box_select.rs");
    let box_select_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/box_select/tests.rs");
    let box_select_tests_fixtures_source =
        include_str!("../src/imui_editor_proof_demo/collection/box_select/tests/fixtures.rs");
    let command_buttons_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons.rs");
    let command_buttons_actions_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/actions.rs");
    let command_buttons_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/command_buttons/chrome.rs");
    let context_menu_source =
        include_str!("../src/imui_editor_proof_demo/collection/context_menu.rs");
    let context_menu_actions_source =
        include_str!("../src/imui_editor_proof_demo/collection/context_menu/actions.rs");
    let context_menu_chrome_source =
        include_str!("../src/imui_editor_proof_demo/collection/context_menu/chrome.rs");
    let derived_state_source =
        include_str!("../src/imui_editor_proof_demo/collection/derived_state.rs");
    let drag_drop_source = include_str!("../src/imui_editor_proof_demo/collection/drag_drop.rs");
    let drag_drop_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/drag_drop/tests.rs");
    let drag_drop_tests_fixtures_source =
        include_str!("../src/imui_editor_proof_demo/collection/drag_drop/tests/fixtures.rs");
    let geometry_source = include_str!("../src/imui_editor_proof_demo/collection/geometry.rs");
    let geometry_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/geometry/tests.rs");
    let geometry_zoom_source =
        include_str!("../src/imui_editor_proof_demo/collection/geometry/zoom.rs");
    let geometry_zoom_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/geometry/zoom/tests.rs");
    let geometry_zoom_tests_fixtures_source =
        include_str!("../src/imui_editor_proof_demo/collection/geometry/zoom/tests/fixtures.rs");
    let import_target_source =
        include_str!("../src/imui_editor_proof_demo/collection/import_target.rs");
    let keyboard_source = include_str!("../src/imui_editor_proof_demo/collection/keyboard.rs");
    let keyboard_actions_source =
        include_str!("../src/imui_editor_proof_demo/collection/keyboard/actions.rs");
    let lifecycle_source = include_str!("../src/imui_editor_proof_demo/collection/lifecycle.rs");
    let models_source = include_str!("../src/imui_editor_proof_demo/collection/models.rs");
    let order_toggle_source =
        include_str!("../src/imui_editor_proof_demo/collection/order_toggle.rs");
    let readouts_source = include_str!("../src/imui_editor_proof_demo/collection/readouts.rs");
    let readout_status_source =
        include_str!("../src/imui_editor_proof_demo/collection/readouts/status.rs");
    let rename_source = include_str!("../src/imui_editor_proof_demo/collection/rename.rs");
    let rename_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/tests.rs");
    let rename_tests_fixtures_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/tests/fixtures.rs");
    let rename_commit_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit.rs");
    let rename_commit_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit/tests.rs");
    let rename_commit_tests_fixtures_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/commit/tests/fixtures.rs");
    let rename_focus_source =
        include_str!("../src/imui_editor_proof_demo/collection/rename/focus.rs");
    let render_states_source =
        include_str!("../src/imui_editor_proof_demo/collection/render_states.rs");
    let runtime_state_source =
        include_str!("../src/imui_editor_proof_demo/collection/runtime_state.rs");
    let selection_source = include_str!("../src/imui_editor_proof_demo/collection/selection.rs");
    let selection_context_menu_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/context_menu.rs");
    let selection_context_menu_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/context_menu/tests.rs");
    let selection_context_menu_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/context_menu/tests/fixtures.rs"
    );
    let selection_keyboard_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard.rs");
    let selection_keyboard_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard/tests.rs");
    let selection_keyboard_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/keyboard/tests/fixtures.rs"
    );
    let selection_keyboard_navigation_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/keyboard/navigation.rs");
    let selection_keyboard_navigation_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/keyboard/navigation/tests.rs"
    );
    let selection_keyboard_navigation_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/keyboard/navigation/tests/fixtures.rs"
    );
    let selection_projection_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/projection.rs");
    let selection_select_all_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/select_all.rs");
    let selection_select_all_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/select_all/tests.rs");
    let selection_select_all_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/select_all/tests/fixtures.rs"
    );
    let selection_commands_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands.rs");
    let selection_delete_commands_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete.rs");
    let selection_delete_commands_tests_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/delete/tests.rs");
    let selection_delete_commands_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/delete/tests/fixtures.rs"
    );
    let selection_duplicate_commands_source =
        include_str!("../src/imui_editor_proof_demo/collection/selection/commands/duplicate.rs");
    let selection_duplicate_commands_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/tests.rs"
    );
    let selection_duplicate_naming_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/naming.rs"
    );
    let selection_duplicate_naming_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/naming/tests.rs"
    );
    let selection_duplicate_naming_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/naming/tests/fixtures.rs"
    );
    let selection_duplicate_selection_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection.rs"
    );
    let selection_duplicate_selection_tests_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection/tests.rs"
    );
    let selection_duplicate_selection_tests_fixtures_source = include_str!(
        "../src/imui_editor_proof_demo/collection/selection/commands/duplicate/selection/tests/fixtures.rs"
    );
    let status_readouts_source =
        include_str!("../src/imui_editor_proof_demo/collection/status_readouts.rs");

    for needle in [
        "mod authoring_parity;",
        "mod collection;",
        "mod editor_advanced;",
        "mod editor_gradient;",
        "mod editor_inspector;",
        "mod editor_material;",
        "mod editor_object;",
        "mod editor_state;",
        "mod editor_text_assist;",
        "use editor_advanced::*;",
        "use editor_gradient::*;",
        "use editor_inspector::*;",
        "use editor_material::*;",
        "use editor_object::*;",
        "use editor_state::*;",
        "use editor_text_assist::*;",
    ] {
        assert!(
            demo_source.contains(needle),
            "imui_editor_proof_demo should keep the collection proof routed through demo-local owners; missing `{needle}`"
        );
    }

    for needle in [
        "fn editor_material_shading_items() -> Arc<[EnumSelectItem]> {",
        "fn named_demo_state<H: UiHost, S: Clone + 'static>(",
        "fn editor_demo_value_model<H: UiHost>(",
        "fn editor_demo_transform_outcome_model<H: UiHost>(",
        "struct GradientDemoStop",
        "fn editor_demo_name_assist_items(",
        "fn editor_demo_search_assist_items(",
        "InspectorPanel::new(",
        "InspectorPanelOptions",
        "InspectorPanelSearchAssistOptions",
        "render_editor_object_surface(",
        "EditorObjectModels {",
        "render_editor_material_surface(",
        "EditorMaterialModels {",
        "render_editor_gradient_surface(",
        "EditorGradientModels {",
        "render_editor_advanced_surface(",
        "EditorAdvancedModels {",
        "proof_empty_state_text(",
        "let any_match = material_any_match || advanced_any_match;",
        "fn record_text_field_outcome(",
        "fn render_editor_name_assist_surface(",
        "PropertyGroup::new(\"Object\")",
        "TextField::new(",
        "TextFieldOptions",
        "TextFieldOutcome",
        "TextFieldBlurBehavior",
        "TextFieldMode",
        "EditorTextSelectionBehavior::SelectAllOnFocus",
        "PropertyGrid::new()",
        "PropertyRow::new()",
        "row_cx.label_text(cx, \"Name\")",
        "editor_text_field_readout(",
        "editor_text_assist_readout(",
        "record_text_field_outcome(",
        "render_editor_name_assist_surface(",
        "PropertyGroup::new(\"Material\")",
        "fn render_editor_material_rows(",
        "fn material_unit_interval_validate(",
        "fn record_drag_value_outcome(",
        "PropertyGroup::new(\"Advanced\")",
        "Vec3Edit::from_presentation(",
        "TransformEdit::from_presentations(",
        "DragValue::new(",
        "NumericInput::from_presentation(",
        "fn advanced_unit_interval_validate(",
        "fn record_vec_axis_outcome(",
        "fn record_transform_axis_outcome(",
        "PropertyGroup::new(\"Gradient\")",
        "GradientEditor::new(",
        "fn render_gradient_editor(",
        "fn remove_gradient_stop_action(",
        "fn add_gradient_stop_action(",
        "fn gradient_stop_bindings(",
        "fn proof_collection_assets_in_visible_order(",
        "fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
        "struct ProofCollectionAsset {",
        "fn proof_collection_drag_rect_normalizes_drag_direction()",
        "collection::authoring_parity_collection_assets()",
    ] {
        assert!(
            !demo_source.contains(needle),
            "imui_editor_proof_demo should not keep the collection implementation inline after modularization; unexpected `{needle}`"
        );
    }

    assert!(
        demo_source.contains("let editor_models = editor_inspector_models(cx);")
            && demo_source.contains("render_editor_inspector_surface("),
        "imui_editor_proof_demo should route the inspector through the demo-local editor inspector owner"
    );

    for needle in [
        "pub(super) struct EditorInspectorModels",
        "pub(super) fn editor_inspector_models(",
        "pub(super) fn render_editor_inspector_surface(",
        "fn render_editor_inspector_content(",
        "InspectorPanel::new(Some(models.search.clone()))",
        "InspectorPanelOptions",
        "InspectorPanelSearchAssistOptions",
        "editor_demo_search_assist_items(cx)",
        "\"imui-editor-proof.editor.inspector\"",
        "\"imui-editor-proof.editor.search.list\"",
        "render_editor_object_surface(",
        "EditorObjectModels {",
        "render_editor_material_surface(",
        "EditorMaterialModels {",
        "render_editor_gradient_surface(",
        "EditorGradientModels {",
        "render_editor_advanced_surface(",
        "EditorAdvancedModels {",
        "let any_match = material_any_match || advanced_any_match;",
        "proof_empty_state_text(",
        "\"imui-editor-proof.editor.no-matches\"",
    ] {
        assert!(
            editor_inspector_source.contains(needle),
            "the demo-local editor inspector owner should own inspector composition and child-owner routing; missing `{needle}`"
        );
    }

    assert!(
        editor_inspector_source.contains("render_editor_object_surface(")
            && editor_inspector_source.contains("EditorObjectModels {"),
        "the demo-local editor inspector owner should route Object through the editor object owner"
    );

    for needle in [
        "pub(super) struct EditorObjectModels",
        "pub(super) fn render_editor_object_surface(",
        "fn render_editor_object_grid(",
        "PropertyGroup::new(\"Object\")",
        "PropertyGrid::new().into_element",
        "PropertyRow::new()",
        "TextField::new(models.name.clone())",
        "TextFieldOptions",
        "TextFieldOutcome",
        "EditorTextSelectionBehavior::SelectAllOnFocus",
        "TextFieldBlurBehavior::Cancel",
        "TextFieldBlurBehavior::PreserveDraft",
        "TextFieldMode::Password",
        "record_text_field_outcome(host, action_cx, &outcome_model, outcome);",
        "render_editor_name_assist_surface(",
        "editor_text_field_readout(",
        "editor_text_assist_readout(",
        "committed_char_count_label(",
        "committed_line_count_label(",
        "\"imui-editor-proof.editor.group.object\"",
        "\"imui-editor-proof.editor.object.name\"",
        "\"imui-editor-proof.editor.object.inline-rename\"",
        "\"imui-editor-proof.editor.object.buffered-name\"",
        "\"imui-editor-proof.editor.object.password\"",
        "\"imui-editor-proof.editor.object.name-assist.state\"",
        "\"imui-editor-proof.editor.object.name-assist.active\"",
        "\"imui-editor-proof.editor.object.name-assist.accepted\"",
        "\"imui-editor-proof.editor.object.notes\"",
        "\"imui-editor-proof.editor.object.notes.committed-lines\"",
    ] {
        assert!(
            editor_object_source.contains(needle),
            "the demo-local editor object owner should own object text fields, readouts, and assist rows; missing `{needle}`"
        );
    }

    assert!(
        editor_inspector_source.contains("render_editor_advanced_surface(")
            && editor_inspector_source.contains("EditorAdvancedModels {"),
        "the demo-local editor inspector owner should route Advanced through the editor advanced owner"
    );

    for needle in [
        "pub(super) struct EditorAdvancedModels",
        "pub(super) struct EditorAdvancedSurface",
        "pub(super) fn render_editor_advanced_surface(",
        "struct EditorAdvancedVisibility",
        "fn from_panel(panel_cx: &InspectorPanelCx) -> Self",
        "PropertyGroup::new(\"Advanced\")",
        "Vec3Edit::from_presentation(",
        "TransformEdit::from_presentations(",
        "DragValue::new(",
        "NumericInput::from_presentation(",
        "vec_edit_axis_outcome_label(outcome)",
        "transform_edit_axis_outcome_label(outcome)",
        "\"imui-editor-proof.editor.advanced.no-matches\"",
        "fn advanced_unit_interval_validate(",
        "fn record_vec_axis_outcome(",
        "fn record_transform_axis_outcome(",
    ] {
        assert!(
            editor_advanced_source.contains(needle),
            "the demo-local editor advanced owner should own advanced search gating and rows; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct EditorGradientModels",
        "pub(super) fn render_editor_gradient_surface(",
        "PropertyGroup::new(\"Gradient\")",
        "fn render_gradient_editor(",
        "GradientEditor::new(bindings)",
        "fn remove_gradient_stop_action(",
        "fn add_gradient_stop_action(",
        "v.saturating_add(1)",
        "GradientDemoStop {",
        "fn gradient_stop_bindings(",
        "GradientStopBinding {",
        "\"imui-editor-proof.editor.gradient.add-stop\"",
    ] {
        assert!(
            editor_gradient_source.contains(needle),
            "the demo-local editor gradient owner should own gradient group actions and bindings; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct EditorMaterialModels",
        "pub(super) struct EditorMaterialSurface",
        "pub(super) fn render_editor_material_surface(",
        "struct EditorMaterialVisibility",
        "fn from_panel(panel_cx: &InspectorPanelCx) -> Self",
        "PropertyGroup::new(\"Material\")",
        "DragValue::from_presentation(",
        "record_drag_value_outcome(host, action_cx, &outcome_model, outcome);",
        "Slider::from_presentation(",
        "ColorEdit::new(models.base_color.clone())",
        "asset_ref::push_material_rows(",
        "EnumSelect::new(models.shading.clone(), editor_material_shading_items())",
        "Checkbox::new(models.alpha_clip.clone())",
        "Checkbox::new_optional(models.cast_shadows.clone())",
        "\"imui-editor-proof.editor.material.no-matches\"",
    ] {
        assert!(
            editor_material_source.contains(needle),
            "the demo-local editor material owner should own material search gating and rows; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn editor_material_shading_items() -> Arc<[EnumSelectItem]> {",
        "pub(super) fn named_demo_state<H: UiHost, S: Clone + 'static>(",
        "pub(super) struct GradientDemoStop",
        "pub(super) id: fret_ui::ItemKey",
        "pub(super) position: Model<f64>",
        "pub(super) color: Model<Color>",
        "pub(super) fn editor_demo_value_model<H: UiHost>(",
        "pub(super) fn editor_demo_roughness_model<H: UiHost>(",
        "pub(super) fn editor_demo_gradient_stops_model<H: UiHost>(",
        "pub(super) fn editor_demo_name_assist_accepted_model<H: UiHost>(",
        "pub(super) fn editor_demo_transform_outcome_model<H: UiHost>(",
        "\"imui_editor_proof_demo.model.transform_outcome\"",
    ] {
        assert!(
            editor_state_source.contains(needle),
            "the demo-local editor state owner should own main proof state fixtures; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn editor_demo_name_assist_items(",
        "pub(super) fn editor_demo_search_assist_items(",
        "fn record_editor_text_assist_accept(",
        "pub(super) fn record_text_field_outcome(",
        "pub(super) fn render_editor_name_assist_surface(",
        "TextAssistItem::new(\"cube\", \"Cube\")",
        "TextAssistItem::new(\"validation\", \"Validation\")",
        "TextAssistField::new(",
        "TextAssistFieldSurface::AnchoredOverlay",
        "record_editor_text_assist_accept(host, &accepted_label_model, active)",
        "let next = edit_session_outcome_label(outcome);",
    ] {
        assert!(
            editor_text_assist_source.contains(needle),
            "the demo-local editor text-assist owner should own assist fixtures and text-field outcome writes; missing `{needle}`"
        );
    }

    for needle in [
        "mod models;",
        "mod shared_state;",
        "mod surface;",
        "pub(super) use models::{",
        "AuthoringParityModels",
        "shared_models",
        "drag_assets",
        "outliner_items_model",
        "pub(super) use shared_state::render_shared_state;",
        "pub(super) use surface::render_surface;",
    ] {
        assert!(
            authoring_parity_source.contains(needle),
            "the demo-local authoring parity hub should re-export split owner surfaces; missing `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) struct AuthoringParityModels {",
        "pub(in super::super) fn shared_models<H: UiHost>(",
        "name: name_model(cx),",
        "gradient_next_id: gradient_next_id_model(cx),",
        "pub(in super::super) fn drag_assets() -> Arc<[ProofDragAsset]> {",
        "super::super::collection::authoring_parity_collection_assets()",
        "pub(in super::super) fn outliner_items() -> Arc<[ProofOutlinerItem]> {",
        "pub(in super::super) fn outliner_items_model<H: UiHost>(",
    ] {
        assert!(
            authoring_parity_models_source.contains(needle),
            "the demo-local authoring parity model owner should own shared proof fixtures; missing `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) fn render_surface(",
        "fn render_authoring_parity_declarative_group(",
        "fn render_authoring_parity_imui_group(",
        "fn build_authoring_parity_gradient_editor(",
        "fn render_authoring_parity_imui_host",
        "fn authoring_parity_shading_items() -> Arc<[EnumSelectItem]>",
        "let asset_chips = drag_assets();",
        "collection::render_collection_first_asset_browser_proof(ui);",
        "sortable_row(ui, row.response(), payload)",
        "publish_cross_window_drag_preview_ghost_with_options(",
    ] {
        assert!(
            authoring_parity_surface_source.contains(needle),
            "the demo-local authoring parity surface owner should own render composition; missing `{needle}`"
        );
        assert!(
            !demo_source.contains(needle),
            "imui_editor_proof_demo should delegate authoring parity render composition to the surface owner; unexpected `{needle}`"
        );
    }

    assert!(
        demo_source
            .contains("authoring_parity::render_surface(cx, parity_models_for_surface.clone())"),
        "imui_editor_proof_demo should mount the authoring parity surface through the child owner"
    );

    for needle in [
        "authoring_parity::name_model(cx)",
        "authoring_parity::drag_value_model(cx)",
        "authoring_parity::numeric_input_model(cx)",
        "authoring_parity::slider_model(cx)",
        "authoring_parity::enabled_model(cx)",
        "authoring_parity::shading_model(cx)",
        "authoring_parity::gradient_angle_model(cx)",
        "authoring_parity::gradient_stops_model(cx)",
        "authoring_parity::gradient_next_id_model(cx)",
    ] {
        assert!(
            !demo_source.contains(needle),
            "imui_editor_proof_demo should gather shared authoring parity models through the child owner bundle; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn render_collection_first_asset_browser_proof(",
        "ui: &mut ImUi<'_, '_, KernelApp>",
        "mod asset_grid;",
        "mod assets;",
        "mod browser_scope;",
        "mod box_select;",
        "mod child_models;",
        "mod chrome;",
        "mod command_buttons;",
        "mod context_menu;",
        "mod derived_state;",
        "mod drag_drop;",
        "mod geometry;",
        "mod import_target;",
        "mod keyboard;",
        "mod lifecycle;",
        "mod models;",
        "mod order_toggle;",
        "mod rename;",
        "mod render_states;",
        "mod runtime_state;",
        "mod selection;",
        "mod status_readouts;",
        "pub(super) use assets::{ProofCollectionAsset, authoring_parity_collection_assets};",
        "pub(super) use chrome::proof_collection_readout_text;",
        "use child_models::{ProofCollectionChildModels, proof_collection_child_models};",
        "use chrome::render_collection_header;",
        "use derived_state::proof_collection_derived_state;",
        "use import_target::render_collection_import_target;",
        "use lifecycle::clear_stale_collection_rename_session;",
        "use order_toggle::render_collection_order_toggle;",
        "use render_states::proof_collection_render_states;",
        "use runtime_state::proof_collection_runtime_state;",
        "render_collection_import_target(ui);",
        "render_collection_order_toggle(",
        "proof_collection_derived_state(",
        "proof_collection_runtime_state(",
        "proof_collection_child_models(&collection_runtime.models)",
        "proof_collection_render_states(",
        "clear_stale_collection_rename_session(",
        "use status_readouts::render_collection_status_readouts;",
        "render_collection_status_readouts(",
        "render_collection_header(ui);",
    ] {
        assert!(
            collection_source.contains(needle),
            "the demo-local collection module should keep the modularized implementation explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) fn proof_collection_readout_text(",
        "pub(super) fn render_collection_header(",
        "pub(super) fn proof_collection_section_label(",
        "Collection-first asset browser proof",
        "Stable keys keep browser selection pinned while visible order flips",
        "Background drag now draws a marquee and updates grid selection app-locally",
        "proof_compact_readout_element(cx, text, Arc::<str>::from(test_id))",
        "proof_section_chrome_label(cx, text, test_id)",
    ] {
        assert!(
            chrome_source.contains(needle),
            "the demo-local collection chrome owner should keep readout/title mounting explicit; missing `{needle}`"
        );
    }

    for needle in [
        "fn proof_collection_readout_text(",
        "fn render_collection_header(",
        "fn proof_collection_section_label(",
        "Collection-first asset browser proof",
        "Stable keys keep browser selection pinned while visible order flips",
        "Background drag now draws a marquee and updates grid selection app-locally",
        "proof_compact_readout_element(cx, text, Arc::<str>::from(test_id))",
        "proof_section_chrome_label(cx, text, test_id)",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route chrome/readout mounting through collection/chrome.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionDerivedState",
        "pub(super) fn proof_collection_derived_state(",
        "stored_assets: &[ProofCollectionAsset]",
        "reverse_order: bool",
        "proof_collection_assets_in_visible_order(",
        "Arc::<[ProofCollectionAsset]>::from(stored_assets.to_vec())",
        "let keys = assets",
        ".map(|asset| asset.id.clone())",
        ".collect::<Vec<_>>();",
        "proof_collection_active_id(&keys, selection, keyboard)",
        "proof_collection_begin_rename_session(&assets, selection, keyboard)",
        "rename_ready_session",
    ] {
        assert!(
            derived_state_source.contains(needle),
            "the demo-local collection derived-state owner should keep visible asset/key/active/rename-ready projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "proof_collection_assets_in_visible_order(",
        "proof_collection_active_id(",
        "proof_collection_begin_rename_session(",
        "let collection_keys =",
        "let collection_active_id =",
        "let collection_rename_ready_session =",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route derived visible state through collection/derived_state.rs; unexpected `{needle}`"
        );
    }

    runtime_state::assert_runtime_state_owner_split(collection_source, runtime_state_source);

    child_models::assert_child_models_owner_split(collection_source, child_models_source);

    lifecycle::assert_lifecycle_owner_split(collection_source, lifecycle_source);

    render_states::assert_render_states_owner_split(collection_source, render_states_source);

    order_toggle::assert_order_toggle_owner_split(collection_source, order_toggle_source);

    status_readouts::assert_status_readouts_owner_split(
        collection_source,
        readouts_source,
        readout_status_source,
        status_readouts_source,
    );

    import_target::assert_import_target_owner_split(collection_source, import_target_source);

    assets::assert_assets_owner_split(assets_source);

    browser_scope::assert_browser_scope_owner_split(
        browser_scope_source,
        browser_scope_chrome_source,
        browser_scope_asset_grid_source,
    );

    browser_input_runtime::assert_browser_input_runtime_owner_split(
        browser_input_runtime_source,
        browser_input_box_select_runtime_source,
        browser_input_box_select_session_source,
        browser_input_box_select_session_tests_source,
        browser_input_box_select_session_tests_fixtures_source,
        browser_input_context_menu_runtime_source,
        browser_input_context_menu_runtime_tests_source,
        browser_input_context_menu_runtime_tests_fixtures_source,
        browser_input_zoom_runtime_source,
    );

    asset_grid::assert_asset_grid_owner_split(
        asset_grid_source,
        asset_grid_tile_source,
        asset_grid_actions_source,
        asset_grid_chrome_source,
        asset_grid_inline_rename_source,
        asset_grid_inline_rename_actions_source,
        asset_grid_metadata_source,
    );

    box_select::assert_box_select_owner_split(
        box_select_source,
        box_select_tests_source,
        box_select_tests_fixtures_source,
    );

    command_buttons::assert_command_buttons_owner_split(
        command_buttons_source,
        command_buttons_actions_source,
        command_buttons_chrome_source,
    );

    context_menu::assert_context_menu_owner_split(
        context_menu_source,
        context_menu_actions_source,
        context_menu_chrome_source,
    );

    drag_drop::assert_drag_drop_owner_split(
        drag_drop_source,
        drag_drop_tests_source,
        drag_drop_tests_fixtures_source,
    );

    geometry::assert_geometry_owner_split(
        geometry_source,
        geometry_tests_source,
        geometry_zoom_source,
        geometry_zoom_tests_source,
        geometry_zoom_tests_fixtures_source,
    );

    keyboard::assert_keyboard_owner_split(keyboard_source, keyboard_actions_source);

    models::assert_models_owner_split(models_source);

    rename::assert_rename_owner_split(
        rename_source,
        rename_tests_source,
        rename_tests_fixtures_source,
        rename_commit_source,
        rename_commit_tests_source,
        rename_commit_tests_fixtures_source,
        rename_focus_source,
    );

    selection::assert_selection_owner_split(selection_source, selection_projection_source);
    selection_commands::assert_selection_commands_owner_split(selection_commands_source);
    selection_keyboard::assert_selection_keyboard_owner_split(
        selection_keyboard_source,
        selection_keyboard_tests_source,
        selection_keyboard_tests_fixtures_source,
        selection_keyboard_navigation_source,
        selection_keyboard_navigation_tests_source,
        selection_keyboard_navigation_tests_fixtures_source,
    );
    selection_context_menu::assert_selection_context_menu_owner_split(
        selection_context_menu_source,
        selection_context_menu_tests_source,
        selection_context_menu_tests_fixtures_source,
    );
    selection_select_all::assert_selection_select_all_owner_split(
        selection_source,
        selection_select_all_source,
        selection_select_all_tests_source,
        selection_select_all_tests_fixtures_source,
    );

    selection_delete::assert_selection_delete_owner_split(
        selection_delete_commands_source,
        selection_delete_commands_tests_source,
        selection_delete_commands_tests_fixtures_source,
    );

    selection_duplicate::assert_selection_duplicate_owner_split(
        selection_duplicate_commands_source,
        selection_duplicate_commands_tests_source,
    );

    selection_duplicate_naming::assert_selection_duplicate_naming_owner_split(
        selection_duplicate_naming_source,
        selection_duplicate_naming_tests_source,
        selection_duplicate_naming_tests_fixtures_source,
    );

    selection_duplicate_selection::assert_selection_duplicate_selection_owner_split(
        selection_duplicate_selection_source,
        selection_duplicate_selection_tests_source,
        selection_duplicate_selection_tests_fixtures_source,
    );
}
