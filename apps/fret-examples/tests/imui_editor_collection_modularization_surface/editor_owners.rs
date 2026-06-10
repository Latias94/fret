pub(super) fn assert_editor_owner_split(
    editor_inspector_source: &str,
    editor_object_router_source: &str,
    editor_object_surface_source: &str,
    editor_advanced_router_source: &str,
    editor_advanced_surface_source: &str,
    editor_gradient_source: &str,
    editor_material_router_source: &str,
    editor_material_surface_source: &str,
    editor_state_source: &str,
    editor_text_assist_source: &str,
) {
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
        "pub struct EditorObjectModels",
        "pub fn render_editor_object_surface(",
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
            editor_object_surface_source.contains(needle),
            "the demo-local editor object surface owner should own object text fields, readouts, and assist rows; missing `{needle}`"
        );
    }

    for needle in [
        "mod surface;",
        "pub use surface::{EditorObjectModels, render_editor_object_surface};",
    ] {
        assert!(
            editor_object_router_source.contains(needle),
            "the demo-local editor object router should only re-export the surface owner; missing `{needle}`"
        );
    }

    assert!(
        editor_inspector_source.contains("render_editor_advanced_surface(")
            && editor_inspector_source.contains("EditorAdvancedModels {"),
        "the demo-local editor inspector owner should route Advanced through the editor advanced owner"
    );

    for needle in [
        "pub struct EditorAdvancedModels",
        "pub struct EditorAdvancedSurface",
        "pub fn render_editor_advanced_surface(",
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
            editor_advanced_surface_source.contains(needle),
            "the demo-local editor advanced surface owner should own advanced search gating and rows; missing `{needle}`"
        );
    }

    for needle in [
        "mod surface;",
        "pub use surface::{EditorAdvancedModels, EditorAdvancedSurface, render_editor_advanced_surface};",
    ] {
        assert!(
            editor_advanced_router_source.contains(needle),
            "the demo-local editor advanced router should only re-export the surface owner; missing `{needle}`"
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
        "pub struct EditorMaterialModels",
        "pub struct EditorMaterialSurface",
        "pub fn render_editor_material_surface(",
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
            editor_material_surface_source.contains(needle),
            "the demo-local editor material surface owner should own material search gating and rows; missing `{needle}`"
        );
    }

    for needle in [
        "mod surface;",
        "pub use surface::{EditorMaterialModels, EditorMaterialSurface, render_editor_material_surface};",
    ] {
        assert!(
            editor_material_router_source.contains(needle),
            "the demo-local editor material router should only re-export the surface owner; missing `{needle}`"
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
}
