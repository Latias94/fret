pub(super) fn assert_demo_module_routing(demo_source: &str) {
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
}
