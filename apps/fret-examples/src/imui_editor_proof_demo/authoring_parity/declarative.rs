use std::sync::Arc;

use fret::AppComponentCx;
use fret::advanced::KernelApp;
use fret::component::prelude::*;
use fret_core::Color;
use fret_ui_editor::composites::{
    GradientEditor, GradientEditorOptions, GradientStopBinding, PropertyGrid, PropertyGroup,
    PropertyRow,
};
use fret_ui_editor::controls::{
    Checkbox, DragValue, EditorTextSelectionBehavior, EnumSelect, EnumSelectItem,
    EnumSelectOptions, FieldStatus, FieldStatusBadge, NumericInput, Slider, TextField,
    TextFieldOptions,
};
use fret_ui_editor::imui as editor_imui;

use super::super::proof_helpers::{
    authoring_parity_blend_presentation, authoring_parity_blend_slider_options,
    authoring_parity_drag_value_options, authoring_parity_numeric_input_options,
    authoring_parity_theme_diag_lines, authoring_parity_value_presentation,
    editor_string_model_readout, proof_compact_readout, proof_drag_preview_card,
    proof_outliner_items_snapshot, proof_outliner_order_line_for_model, proof_section_chrome_label,
};
use super::super::{GradientDemoStop, collection, diag_enabled};
use super::common::build_authoring_parity_gradient_editor;
use super::{
    AuthoringParityModels, asset_slot_model, drag_assets, outliner_items_model,
    outliner_status_model,
};

pub(super) fn render_authoring_parity_declarative_group(
    cx: &mut AppComponentCx<'_>,
    models: AuthoringParityModels,
    shading_items: Arc<[EnumSelectItem]>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let value_presentation = authoring_parity_value_presentation();
    let blend_presentation = authoring_parity_blend_presentation();

    PropertyGroup::new("Declarative authoring")
        .options(fret_ui_editor::composites::PropertyGroupOptions {
            test_id: Some(Arc::from("imui-editor-proof.authoring.declarative.group")),
            header_test_id: Some(Arc::from(
                "imui-editor-proof.authoring.declarative.group.header",
            )),
            content_test_id: Some(Arc::from(
                "imui-editor-proof.authoring.declarative.group.content",
            )),
            ..Default::default()
        })
        .into_element(
            cx,
            |_cx| None,
            move |cx| {
                vec![
                    PropertyGrid::new().into_element(cx, move |cx, row_cx| {
                        let mut rows = Vec::new();

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new(),
                            |cx| row_cx.label_text(cx, "Name"),
                            |cx| {
                                TextField::new(models.name.clone())
                                    .options(TextFieldOptions {
                                        clear_button: true,
                                        selection_behavior:
                                            EditorTextSelectionBehavior::SelectAllOnFocus,
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.name",
                                        )),
                                        clear_test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.name.clear",
                                        )),
                                        ..Default::default()
                                    })
                                    .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new(),
                            |cx| row_cx.label_text(cx, "Drag value"),
                            |cx| {
                                DragValue::from_presentation(
                                    models.drag_value.clone(),
                                    value_presentation.clone(),
                                )
                                .options(authoring_parity_drag_value_options(
                                    &value_presentation,
                                    "authoring-parity.declarative.drag-value",
                                    "imui-editor-proof.authoring.declarative.value",
                                ))
                                .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new(),
                            |cx| row_cx.label_text(cx, "Typed numeric"),
                            |cx| {
                                NumericInput::from_presentation(
                                    models.numeric_input.clone(),
                                    value_presentation.clone(),
                                )
                                .options(authoring_parity_numeric_input_options(
                                    &value_presentation,
                                    "authoring-parity.declarative.numeric-input",
                                    "imui-editor-proof.authoring.declarative.numeric",
                                ))
                                .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new(),
                            |cx| row_cx.label_text(cx, "Blend slider"),
                            |cx| {
                                Slider::from_presentation(
                                    models.slider.clone(),
                                    0.0,
                                    1.0,
                                    blend_presentation.clone(),
                                )
                                .options(authoring_parity_blend_slider_options(
                                    "authoring-parity.declarative.slider",
                                    "imui-editor-proof.authoring.declarative.blend",
                                ))
                                .into_element(cx)
                            },
                            |cx| {
                                Some(
                                    FieldStatusBadge::new(FieldStatus::Dirty)
                                        .into_element(cx)
                                        .test_id(
                                            "imui-editor-proof.authoring.declarative.blend.status",
                                        ),
                                )
                            },
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new(),
                            |cx| row_cx.label_text(cx, "Enabled"),
                            |cx| {
                                Checkbox::new(models.enabled.clone())
                                    .options(fret_ui_editor::controls::CheckboxOptions {
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.enabled",
                                        )),
                                        ..Default::default()
                                    })
                                    .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new(),
                            |cx| row_cx.label_text(cx, "Mode"),
                            |cx| {
                                EnumSelect::new(models.shading.clone(), shading_items.clone())
                                    .options(EnumSelectOptions {
                                        id_source: Some(Arc::from(
                                            "authoring-parity.declarative.mode",
                                        )),
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.mode",
                                        )),
                                        list_test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.mode.list",
                                        )),
                                        search_test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.declarative.mode.search",
                                        )),
                                        ..Default::default()
                                    })
                                    .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows
                    }),
                    proof_section_chrome_label(
                        cx,
                        "Gradient editor",
                        "imui-editor-proof.authoring.declarative.gradient.label",
                    ),
                    build_authoring_parity_gradient_editor(
                        cx,
                        models.gradient_angle.clone(),
                        models.gradient_stops.clone(),
                        models.gradient_next_id.clone(),
                        "authoring-parity.declarative.gradient",
                        "imui-editor-proof.authoring.declarative.gradient",
                    )
                    .into_element(cx),
                ]
            },
        )
}
