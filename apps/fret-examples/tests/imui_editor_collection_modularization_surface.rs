#[path = "imui_editor_collection_modularization_surface/asset_grid.rs"]
mod asset_grid;
#[path = "imui_editor_collection_modularization_surface/box_select.rs"]
mod box_select;
#[path = "imui_editor_collection_modularization_surface/browser_input_runtime.rs"]
mod browser_input_runtime;
#[path = "imui_editor_collection_modularization_surface/browser_scope.rs"]
mod browser_scope;
#[path = "imui_editor_collection_modularization_surface/command_buttons.rs"]
mod command_buttons;
#[path = "imui_editor_collection_modularization_surface/context_menu.rs"]
mod context_menu;
#[path = "imui_editor_collection_modularization_surface/drag_drop.rs"]
mod drag_drop;
#[path = "imui_editor_collection_modularization_surface/geometry.rs"]
mod geometry;
#[path = "imui_editor_collection_modularization_surface/keyboard.rs"]
mod keyboard;
#[path = "imui_editor_collection_modularization_surface/rename.rs"]
mod rename;
#[path = "imui_editor_collection_modularization_surface/selection.rs"]
mod selection;
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

    for needle in [
        "pub(super) struct ProofCollectionRuntimeState",
        "pub(super) struct ProofCollectionRuntimeModels",
        "pub(super) struct ProofCollectionRuntimeSnapshot",
        "pub(super) fn rename_session(&self) -> Option<&ProofCollectionRenameSession>",
        "pub(super) fn proof_collection_runtime_state(",
        "selection: authoring_parity_collection_selection_model(ui.cx_mut())",
        "assets: authoring_parity_collection_assets_model(ui.cx_mut())",
        "reverse_order: authoring_parity_collection_reverse_order_model(ui.cx_mut())",
        "box_select: authoring_parity_collection_box_select_model(ui.cx_mut())",
        "keyboard: authoring_parity_collection_keyboard_model(ui.cx_mut())",
        "zoom: authoring_parity_collection_zoom_model(ui.cx_mut())",
        "context_menu_anchor: authoring_parity_collection_context_menu_anchor_model(ui.cx_mut())",
        "rename_session: authoring_parity_collection_rename_session_model(ui.cx_mut())",
        "rename_draft: authoring_parity_collection_rename_draft_model(ui.cx_mut())",
        "rename_focus_pending: authoring_parity_collection_rename_focus_pending_model(ui.cx_mut())",
        "active_focus_target: authoring_parity_collection_active_focus_target_model(ui.cx_mut())",
        "rename_status: authoring_parity_collection_rename_status_model(ui.cx_mut())",
        "command_status: authoring_parity_collection_command_status_model(ui.cx_mut())",
        "scroll: authoring_parity_collection_scroll_handle(ui.cx_mut())",
        "fn proof_collection_runtime_snapshot(",
        "selector_model_paint(&models.assets, |state| state.clone())",
        "selector_model_paint(&models.selection, |state| state)",
        "selector_model_paint(&models.rename_status, |state| state.clone())",
        "proof_collection_layout_metrics(models.scroll.viewport_size().width, tile_extent)",
    ] {
        assert!(
            runtime_state_source.contains(needle),
            "the demo-local collection runtime-state owner should keep model handles, selector snapshots, and layout projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "authoring_parity_collection_selection_model(ui.cx_mut())",
        "authoring_parity_collection_assets_model(ui.cx_mut())",
        "authoring_parity_collection_reverse_order_model(ui.cx_mut())",
        "authoring_parity_collection_box_select_model(ui.cx_mut())",
        "authoring_parity_collection_keyboard_model(ui.cx_mut())",
        "authoring_parity_collection_zoom_model(ui.cx_mut())",
        "authoring_parity_collection_context_menu_anchor_model(ui.cx_mut())",
        "authoring_parity_collection_rename_session_model(ui.cx_mut())",
        "authoring_parity_collection_rename_draft_model(ui.cx_mut())",
        "authoring_parity_collection_rename_focus_pending_model(ui.cx_mut())",
        "authoring_parity_collection_active_focus_target_model(ui.cx_mut())",
        "authoring_parity_collection_rename_status_model(ui.cx_mut())",
        "authoring_parity_collection_command_status_model(ui.cx_mut())",
        "authoring_parity_collection_scroll_handle(ui.cx_mut())",
        "selector_model_paint(",
        "proof_collection_layout_metrics(",
        "use fret::advanced::view::AppRenderDataExt as _;",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route runtime model/snapshot reads through collection/runtime_state.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionChildModels",
        "pub(super) fn proof_collection_child_models(",
        "models: &ProofCollectionRuntimeModels",
        "command_buttons: ProofCollectionCommandButtonModels {",
        "browser_scope: ProofCollectionBrowserScopeModels {",
        "context_menu: ProofCollectionContextMenuModels {",
        "assets: models.assets.clone()",
        "selection: models.selection.clone()",
        "keyboard: models.keyboard.clone()",
        "rename_session: models.rename_session.clone()",
        "scroll: models.scroll.clone()",
    ] {
        assert!(
            child_models_source.contains(needle),
            "the demo-local collection child-model owner should keep child model bundle projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "ProofCollectionCommandButtonModels {",
        "ProofCollectionBrowserScopeModels {",
        "ProofCollectionContextMenuModels {",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route child model bundle projection through collection/child_models.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn clear_stale_collection_rename_session(",
        "models: &ProofCollectionRuntimeModels",
        "snapshot: &ProofCollectionRuntimeSnapshot",
        "assets: &[ProofCollectionAsset]",
        "snapshot.rename_session.as_ref()",
        "!assets.iter().any(|asset| asset.id == session.target_id)",
        ".update(&models.rename_session, |state| *state = None)",
        ".update(&models.rename_focus_pending, |state| *state = false)",
    ] {
        assert!(
            lifecycle_source.contains(needle),
            "the demo-local collection lifecycle owner should keep stale rename cleanup explicit; missing `{needle}`"
        );
    }

    for needle in [
        "snapshot.rename_session.as_ref()",
        "models.rename_session",
        "models.rename_focus_pending",
        ".update(&collection_runtime.models.rename_session",
        ".update(&collection_runtime.models.rename_focus_pending",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route stale rename cleanup through collection/lifecycle.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionRenderStates",
        "pub(super) fn proof_collection_render_states<'a>(",
        "runtime: &'a ProofCollectionRuntimeState",
        "state: &'a ProofCollectionDerivedState",
        "status_readouts: ProofCollectionStatusReadoutState {",
        "command_buttons: ProofCollectionCommandButtonState {",
        "browser_scope: ProofCollectionBrowserScopeState {",
        "rename_ready_session: state.rename_ready_session.as_ref()",
        "rename_session: runtime.snapshot.rename_session()",
        "rename_focus_pending: runtime.snapshot.rename_focus_pending",
    ] {
        assert!(
            render_states_source.contains(needle),
            "the demo-local collection render-state owner should keep child render-state projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "ProofCollectionStatusReadoutState {",
        "ProofCollectionCommandButtonState {",
        "ProofCollectionBrowserScopeState {",
        "collection_runtime.snapshot.rename_status.as_str()",
        "collection_runtime.snapshot.command_status.as_str()",
        "collection_runtime.snapshot.rename_session()",
        "collection_state.rename_ready_session.as_ref()",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route child render-state projection through collection/render_states.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn render_collection_order_toggle(",
        "reverse_order_model: &Model<bool>",
        "if reverse_order {",
        "\"Show folder order\"",
        "\"Reverse visible order\"",
        "ui.button_with_options(",
        "kit::ButtonOptions {",
        "\"imui-editor-proof.authoring.imui.collection.order-toggle\"",
        "if !order_toggle.clicked()",
        ".update(reverse_order_model, |value| *value = !*value)",
        "!reverse_order",
    ] {
        assert!(
            order_toggle_source.contains(needle),
            "the demo-local collection order-toggle owner should keep reverse-order button logic explicit; missing `{needle}`"
        );
    }

    for needle in [
        "\"Show folder order\"",
        "\"Reverse visible order\"",
        "\"imui-editor-proof.authoring.imui.collection.order-toggle\"",
        "ui.button_with_options(",
        "kit::ButtonOptions {",
        ".update(&collection_reverse_order_model, |value| *value = !*value)",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route reverse-order button UI through collection/order_toggle.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionStatusReadoutState",
        "pub(super) fn render_collection_status_readouts(",
        "proof_collection_assets_line(state.assets)",
        "proof_collection_visible_order_line(state.assets)",
        "proof_collection_selection_line(state.assets, state.selection)",
        "proof_collection_active_line(state.assets, state.selection, state.keyboard)",
        "proof_collection_zoom_line(state.layout)",
        "proof_collection_select_all_line()",
        "proof_collection_rename_line()",
        "proof_collection_context_menu_line()",
        "proof_collection_command_package_line()",
        "proof_collection_rename_status_line(state.rename_status)",
        "proof_collection_command_status_line(state.command_status)",
        "\"imui-editor-proof.authoring.imui.collection.assets-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.visible-order-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.selection-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.active-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.zoom-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.select-all-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-package-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-status-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-status-readout\"",
    ] {
        assert!(
            status_readouts_source.contains(needle),
            "the demo-local collection status-readouts owner should keep readout mounting explicit; missing `{needle}`"
        );
    }

    for needle in [
        "mod status;",
        "pub(super) use status::{",
        "proof_collection_command_status_line",
        "proof_collection_delete_status",
        "proof_collection_duplicate_status",
        "proof_collection_rename_cancel_status",
        "proof_collection_rename_commit_status",
        "proof_collection_rename_invalid_status",
        "proof_collection_rename_ready_status",
        "proof_collection_rename_status_line",
        "proof_collection_select_all_status",
        "pub(super) fn proof_collection_selection_line(",
        "pub(super) fn proof_collection_visible_order_line(",
        "pub(super) fn proof_collection_active_line(",
        "pub(super) fn proof_collection_assets_line(",
        "pub(super) fn proof_collection_command_package_line() -> String",
        "pub(super) fn proof_collection_select_all_line() -> String",
        "pub(super) fn proof_collection_rename_line() -> String",
        "pub(super) fn proof_collection_context_menu_line() -> String",
        "proof_collection_selected_assets",
        "proof_collection_active_id",
    ] {
        assert!(
            readouts_source.contains(needle),
            "the demo-local collection readouts hub should keep line readouts and status re-exports explicit; missing `{needle}`"
        );
    }

    for needle in [
        "fn proof_collection_command_status_line(",
        "fn proof_collection_select_all_status(",
        "fn proof_collection_rename_ready_status(",
        "fn proof_collection_rename_commit_status(",
        "fn proof_collection_rename_invalid_status(",
        "fn proof_collection_rename_cancel_status(",
        "fn proof_collection_rename_status_line(",
        "fn proof_collection_duplicate_status(",
        "fn proof_collection_delete_status(",
    ] {
        assert!(
            !readouts_source.contains(needle),
            "the demo-local collection readouts hub should route status formatting through readouts/status.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "use super::super::ProofCollectionAsset;",
        "pub(in super::super) fn proof_collection_command_status_line(",
        "pub(in super::super) fn proof_collection_select_all_status(",
        "pub(in super::super) fn proof_collection_rename_ready_status(",
        "pub(in super::super) fn proof_collection_rename_commit_status(",
        "pub(in super::super) fn proof_collection_rename_invalid_status(",
        "pub(in super::super) fn proof_collection_rename_cancel_status(",
        "pub(in super::super) fn proof_collection_rename_status_line(",
        "pub(in super::super) fn proof_collection_duplicate_status(",
        "pub(in super::super) fn proof_collection_delete_status(",
        "format!(\"Command status: {status}\")",
        "format!(\"Rename status: {status}\")",
        "Duplicated {} asset(s): {labels}",
        "Deleted {} asset(s): {labels}",
    ] {
        assert!(
            readout_status_source.contains(needle),
            "the demo-local collection readout status owner should keep command/rename/delete/duplicate status formatting explicit; missing `{needle}`"
        );
    }

    for needle in [
        "proof_collection_selection_line(",
        "proof_collection_visible_order_line(",
        "proof_collection_active_line(",
        "proof_collection_command_package_line(",
        "ImUiMultiSelectState",
        "ProofCollectionKeyboardState",
        "proof_collection_selected_assets",
        "proof_collection_active_id",
        "TextField",
        "kit::ButtonOptions",
    ] {
        assert!(
            !readout_status_source.contains(needle),
            "the demo-local collection readout status owner should not take line/projection/UI responsibilities; unexpected `{needle}`"
        );
    }

    for needle in [
        "proof_collection_assets_line(",
        "proof_collection_visible_order_line(",
        "proof_collection_selection_line(",
        "proof_collection_active_line(",
        "proof_collection_zoom_line(",
        "proof_collection_select_all_line(",
        "proof_collection_rename_line(",
        "proof_collection_context_menu_line(",
        "proof_collection_command_package_line(",
        "proof_collection_rename_status_line(",
        "proof_collection_command_status_line(",
        "\"imui-editor-proof.authoring.imui.collection.assets-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.visible-order-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.selection-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.active-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.zoom-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.select-all-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-package-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-status-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-status-readout\"",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route status readouts through collection/status_readouts.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn render_collection_import_target(",
        "authoring_parity_collection_drop_status_model(ui.cx_mut())",
        "ui.button_with_options(",
        "ui.drop_target::<ProofCollectionDragPayload>(import_trigger)",
        "proof_collection_drop_status(\"Delivered\", &payload)",
        "proof_collection_drop_status(\"Preview\", &payload)",
        "\"Compatible collection drag active\"",
        "\"imui-editor-proof.authoring.imui.collection.import-target\"",
        "\"imui-editor-proof.authoring.imui.collection.drop-status-readout\"",
    ] {
        assert!(
            import_target_source.contains(needle),
            "the demo-local collection import-target owner should keep import drop/status UI explicit; missing `{needle}`"
        );
    }

    for needle in [
        "ProofCollectionDragPayload",
        "proof_collection_drop_status(",
        "authoring_parity_collection_drop_status_model",
        "ui.drop_target::<",
        "\"imui-editor-proof.authoring.imui.collection.import-target\"",
        "\"imui-editor-proof.authoring.imui.collection.drop-status-readout\"",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route import target/drop-status UI through collection/import_target.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(in super::super) struct ProofCollectionAsset {",
        "pub(in super::super) fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
        "ProofCollectionAsset {",
        "id: Arc::from(\"stone-albedo\")",
        "path: Arc::from(\"textures/stone/albedo.ktx2\")",
    ] {
        assert!(
            assets_source.contains(needle),
            "the demo-local collection assets owner should keep asset fixtures explicit; missing `{needle}`"
        );
    }

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

    for needle in [
        "pub(super) fn authoring_parity_collection_selection_model<H: UiHost>(",
        "pub(super) fn authoring_parity_collection_assets_model<H: UiHost>(",
        "pub(super) fn authoring_parity_collection_scroll_handle<H: UiHost>(",
    ] {
        assert!(
            models_source.contains(needle),
            "the demo-local collection models owner should keep state slot registration explicit; missing `{needle}`"
        );
    }

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

    for needle in [
        "mod delete;",
        "mod duplicate;",
        "pub(in super::super) use delete::{",
    ] {
        assert!(
            selection_commands_source.contains(needle),
            "the demo-local collection selection command hub should keep sub-owner re-exports explicit; missing `{needle}`"
        );
    }

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
