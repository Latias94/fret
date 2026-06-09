use std::sync::Arc;

use fret::AppComponentCx;
use fret::advanced::KernelApp;
use fret::advanced::view::AppRenderDataExt as _;
use fret::component::prelude::*;
use fret::imui::{kit, prelude::*};
use fret_core::Color;
use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};
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
use fret_ui_kit::recipes::imui_drag_preview::{
    DragPreviewGhostOptions, drag_preview_ghost_with_options,
    publish_cross_window_drag_preview_ghost_with_options,
};
use fret_ui_kit::recipes::imui_sortable::{
    SortableInsertionSide, reorder_vec_by_key, sortable_row,
};

use super::super::proof_helpers::{
    ProofDragAsset, ProofOutlinerDragItem, authoring_parity_blend_presentation,
    authoring_parity_blend_slider_options, authoring_parity_drag_value_options,
    authoring_parity_numeric_input_options, authoring_parity_theme_diag_lines,
    authoring_parity_value_presentation, editor_string_model_readout, proof_compact_readout,
    proof_drag_preview_card, proof_outliner_items_snapshot, proof_outliner_order_line_for_model,
    proof_section_chrome_label,
};
use super::super::{GradientDemoStop, collection, diag_enabled};
use super::{
    AuthoringParityModels, asset_slot_model, drag_assets, outliner_items_model,
    outliner_status_model,
};
pub(in super::super) fn render_surface(
    cx: &mut AppComponentCx<'_>,
    models: AuthoringParityModels,
) -> impl IntoUiElement<KernelApp> + use<> {
    let shading_items = authoring_parity_shading_items();

    fret_ui_kit::ui::v_flex_build(move |cx, out| {
        if diag_enabled() {
            let [theme_line, editor_line] = authoring_parity_theme_diag_lines(cx);
            out.push(proof_compact_readout(
                cx,
                theme_line,
                Some(Arc::from("imui-editor-proof.authoring.diag.theme")),
            ));
            out.push(proof_compact_readout(
                cx,
                editor_line,
                Some(Arc::from("imui-editor-proof.authoring.diag.editor")),
            ));
        }

        out.push(
            fret_ui_kit::ui::h_flex_build(move |cx, out| {
                out.push(
                    fret_ui_kit::ui::container_build({
                        let shading_items = shading_items.clone();
                        let models = models.clone();
                        move |cx, out| {
                            out.push(
                                render_authoring_parity_declarative_group(
                                    cx,
                                    models,
                                    shading_items,
                                )
                                .into_element(cx),
                            );
                        }
                    })
                    .basis_0()
                    .flex_1()
                    .into_element(cx),
                );

                out.push(
                    fret_ui_kit::ui::container_build(move |cx, out| {
                        out.push(
                            render_authoring_parity_imui_group(cx, models, shading_items)
                                .into_element(cx),
                        );
                    })
                    .basis_0()
                    .flex_1()
                    .into_element(cx),
                );
            })
            .gap(fret_ui_kit::Space::N3)
            .into_element(cx),
        );
    })
    .gap(fret_ui_kit::Space::N2)
    .into_element(cx)
}

fn render_authoring_parity_declarative_group(
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

fn render_authoring_parity_imui_group(
    cx: &mut AppComponentCx<'_>,
    models: AuthoringParityModels,
    shading_items: Arc<[EnumSelectItem]>,
) -> impl IntoUiElement<KernelApp> + use<> {
    let value_presentation = authoring_parity_value_presentation();
    let blend_presentation = authoring_parity_blend_presentation();

    render_authoring_parity_imui_host(cx, move |ui| {
        editor_imui::property_group(
            ui,
            PropertyGroup::new("imui authoring").options(
                fret_ui_editor::composites::PropertyGroupOptions {
                    test_id: Some(Arc::from("imui-editor-proof.authoring.imui.group")),
                    header_test_id: Some(Arc::from(
                        "imui-editor-proof.authoring.imui.group.header",
                    )),
                    content_test_id: Some(Arc::from(
                        "imui-editor-proof.authoring.imui.group.content",
                    )),
                    ..Default::default()
                },
            ),
            |_cx| None,
            move |cx| {
                let mut out = Vec::new();
                imui_build(cx, &mut out, move |ui| {
                    editor_imui::property_grid(ui, PropertyGrid::new(), move |cx, row_cx| {
                        let mut rows = Vec::new();

                        rows.push(row_cx.row(
                            cx,
                            |cx| row_cx.label_text(cx, "Name"),
                            |cx| {
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    editor_imui::text_field(
                                        ui,
                                        TextField::new(models.name.clone()).options(
                                            TextFieldOptions {
                                                clear_button: true,
                                                selection_behavior:
                                                    EditorTextSelectionBehavior::SelectAllOnFocus,
                                                test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.name",
                                                )),
                                                clear_test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.name.clear",
                                                )),
                                                ..Default::default()
                                            },
                                        ),
                                    );
                                })
                                .into_element(cx)
                            },
                        ));

                        rows.push(row_cx.row(
                            cx,
                            |cx| row_cx.label_text(cx, "Drag value"),
                            |cx| {
                                let value_presentation = value_presentation.clone();
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    let options = authoring_parity_drag_value_options(
                                        &value_presentation,
                                        "authoring-parity.imui.drag-value",
                                        "imui-editor-proof.authoring.imui.value",
                                    );
                                    editor_imui::drag_value(
                                        ui,
                                        DragValue::from_presentation(
                                            models.drag_value.clone(),
                                            value_presentation.clone(),
                                        )
                                        .options(options),
                                    );
                                })
                                .into_element(cx)
                            },
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new(),
                            |cx| row_cx.label_text(cx, "Typed numeric"),
                            |cx| {
                                let value_presentation = value_presentation.clone();
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    let options = authoring_parity_numeric_input_options(
                                        &value_presentation,
                                        "authoring-parity.imui.numeric-input",
                                        "imui-editor-proof.authoring.imui.numeric",
                                    );
                                    editor_imui::numeric_input(
                                        ui,
                                        NumericInput::from_presentation(
                                            models.numeric_input.clone(),
                                            value_presentation.clone(),
                                        )
                                        .options(options),
                                    );
                                })
                                .into_element(cx)
                            },
                            |_cx| None,
                        ));

                        rows.push(row_cx.row_with(
                            cx,
                            PropertyRow::new(),
                            |cx| row_cx.label_text(cx, "Blend slider"),
                            |cx| {
                                let blend_presentation = blend_presentation.clone();
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    editor_imui::slider(
                                        ui,
                                        Slider::from_presentation(
                                            models.slider.clone(),
                                            0.0,
                                            1.0,
                                            blend_presentation.clone(),
                                        )
                                        .options(
                                            authoring_parity_blend_slider_options(
                                                "authoring-parity.imui.slider",
                                                "imui-editor-proof.authoring.imui.blend",
                                            ),
                                        ),
                                    );
                                })
                                .into_element(cx)
                            },
                            |cx| {
                                Some(
                                    render_authoring_parity_imui_host(cx, move |ui| {
                                        editor_imui::field_status_badge(
                                            ui,
                                            FieldStatusBadge::new(FieldStatus::Dirty),
                                        );
                                    })
                                    .into_element(cx)
                                    .test_id("imui-editor-proof.authoring.imui.blend.status"),
                                )
                            },
                        ));

                        rows.push(row_cx.row(
                            cx,
                            |cx| row_cx.label_text(cx, "Enabled"),
                            |cx| {
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    editor_imui::checkbox(
                                        ui,
                                        Checkbox::new(models.enabled.clone()).options(
                                            fret_ui_editor::controls::CheckboxOptions {
                                                test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.enabled",
                                                )),
                                                ..Default::default()
                                            },
                                        ),
                                    );
                                })
                                .into_element(cx)
                            },
                        ));

                        rows.push(row_cx.row(
                            cx,
                            |cx| row_cx.label_text(cx, "Mode"),
                            |cx| {
                                render_authoring_parity_imui_host(cx, move |ui| {
                                    editor_imui::enum_select(
                                        ui,
                                        EnumSelect::new(
                                            models.shading.clone(),
                                            shading_items.clone(),
                                        )
                                        .options(
                                            EnumSelectOptions {
                                                id_source: Some(Arc::from(
                                                    "authoring-parity.imui.mode",
                                                )),
                                                test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.mode",
                                                )),
                                                list_test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.mode.list",
                                                )),
                                                search_test_id: Some(Arc::from(
                                                    "imui-editor-proof.authoring.imui.mode.search",
                                                )),
                                                ..Default::default()
                                            },
                                        ),
                                    );
                                })
                                .into_element(cx)
                            },
                        ));

                        rows
                    });

                    ui.text("Gradient editor");
                    let gradient_editor = build_authoring_parity_gradient_editor(
                        ui.cx_mut(),
                        models.gradient_angle.clone(),
                        models.gradient_stops.clone(),
                        models.gradient_next_id.clone(),
                        "authoring-parity.imui.gradient",
                        "imui-editor-proof.authoring.imui.gradient",
                    );
                    editor_imui::gradient_editor(ui, gradient_editor);
                });
                out
            },
        );

        ui.separator();
        ui.text("Generic tree/collapsing helpers");
        let _ = ui.collapsing_header_with_options(
            "imui-editor-proof.authoring.imui.outliner.section",
            "Scene outliner",
            kit::CollapsingHeaderOptions {
                default_open: true,
                test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.outliner.section",
                )),
                header_test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.outliner.section.header",
                )),
                content_test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.outliner.section.content",
                )),
                ..Default::default()
            },
            |ui| {
                let _ = ui.tree_node_with_options(
                    "imui-editor-proof.authoring.imui.outliner.scene",
                    "Scene",
                    kit::TreeNodeOptions {
                        default_open: true,
                        test_id: Some(Arc::from(
                            "imui-editor-proof.authoring.imui.outliner.scene",
                        )),
                        content_test_id: Some(Arc::from(
                            "imui-editor-proof.authoring.imui.outliner.scene.content",
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.tree_node_with_options(
                            "imui-editor-proof.authoring.imui.outliner.scene.camera",
                            "Camera",
                            kit::TreeNodeOptions {
                                leaf: true,
                                level: 2,
                                selected: true,
                                test_id: Some(Arc::from(
                                    "imui-editor-proof.authoring.imui.outliner.scene.camera",
                                )),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                        let _ = ui.tree_node_with_options(
                            "imui-editor-proof.authoring.imui.outliner.scene.geometry",
                            "Geometry",
                            kit::TreeNodeOptions {
                                default_open: true,
                                level: 2,
                                test_id: Some(Arc::from(
                                    "imui-editor-proof.authoring.imui.outliner.scene.geometry",
                                )),
                                content_test_id: Some(Arc::from(
                                    "imui-editor-proof.authoring.imui.outliner.scene.geometry.content",
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                let _ = ui.tree_node_with_options(
                                    "imui-editor-proof.authoring.imui.outliner.scene.geometry.cube",
                                    "Cube",
                                    kit::TreeNodeOptions {
                                        leaf: true,
                                        level: 3,
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.imui.outliner.scene.geometry.cube",
                                        )),
                                        ..Default::default()
                                    },
                                    |_ui| {},
                                );
                                let _ = ui.tree_node_with_options(
                                    "imui-editor-proof.authoring.imui.outliner.scene.geometry.key-light",
                                    "Key light",
                                    kit::TreeNodeOptions {
                                        leaf: true,
                                        level: 3,
                                        test_id: Some(Arc::from(
                                            "imui-editor-proof.authoring.imui.outliner.scene.geometry.key-light",
                                        )),
                                        ..Default::default()
                                    },
                                    |_ui| {},
                                );
                            },
                        );
                        let _ = ui.tree_node_with_options(
                            "imui-editor-proof.authoring.imui.outliner.scene.postfx",
                            "Post FX",
                            kit::TreeNodeOptions {
                                leaf: true,
                                level: 2,
                                test_id: Some(Arc::from(
                                    "imui-editor-proof.authoring.imui.outliner.scene.postfx",
                                )),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                    },
                );
            },
        );

        ui.separator();
        ui.text("Typed drag/drop helpers");
        ui.text_wrapped(
            "Drag an asset chip onto the material slot. Payload and preview stay app-defined.",
        );

        let asset_slot_model = asset_slot_model(ui.cx_mut());
        let asset_chips = drag_assets();

        ui.horizontal(|ui| {
            for (ix, asset) in asset_chips.iter().enumerate() {
                let trigger = ui.button_with_options(
                    asset.label.clone(),
                    kit::ButtonOptions {
                        test_id: Some(Arc::from(format!(
                            "imui-editor-proof.authoring.imui.drag-drop.asset.{ix}"
                        ))),
                        ..Default::default()
                    },
                );
                let source = ui.drag_source_with_options(
                    trigger,
                    asset.clone(),
                    kit::DragSourceOptions {
                        cross_window: true,
                        ..Default::default()
                    },
                );
                let ghost_id =
                    format!("imui-editor-proof.authoring.imui.drag-drop.asset.{ix}.ghost");
                let _ = publish_cross_window_drag_preview_ghost_with_options(
                    ui,
                    ghost_id.as_str(),
                    source,
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from(format!(
                            "imui-editor-proof.authoring.imui.drag-drop.asset.{ix}.ghost"
                        ))),
                        ..Default::default()
                    },
                    {
                        let label = asset.label.clone();
                        let path = asset.path.clone();
                        move |_cx| proof_drag_preview_card(label.clone(), Some(path.clone()))
                    },
                );
            }
        });

        let assigned_asset = editor_string_model_readout(ui.cx_mut(), &asset_slot_model);
        let slot_trigger = ui.button_with_options(
            format!("Base color slot: {assigned_asset}"),
            kit::ButtonOptions {
                test_id: Some(Arc::from("imui-editor-proof.authoring.imui.drag-drop.slot")),
                ..Default::default()
            },
        );
        let slot_drop = ui.drop_target::<ProofDragAsset>(slot_trigger);
        if let Some(payload) = slot_drop.delivered_payload() {
            let delivered = payload.path.as_ref().to_string();
            let cx = ui.cx_mut();
            let _ = cx
                .app
                .models_mut()
                .update(&asset_slot_model, |value: &mut String| {
                    value.clear();
                    value.push_str(delivered.as_str());
                });
        }

        let drag_drop_status = if let Some(payload) = slot_drop.delivered_payload() {
            format!("Delivered {}", payload.path)
        } else if let Some(payload) = slot_drop.preview_payload() {
            format!("Preview {}", payload.path)
        } else if slot_drop.active() {
            "Compatible drag active".to_string()
        } else {
            "Idle".to_string()
        };
        ui.text(drag_drop_status);

        ui.separator();
        collection::render_collection_first_asset_browser_proof(ui);

        ui.separator();
        ui.text("Reorderable outliner proof");
        ui.text_wrapped(
            "Sortable math stays app-owned. `imui` only provides typed payloads + drop positions.",
        );

        let outliner_items_model = outliner_items_model(ui.cx_mut());
        let outliner_status_model = outliner_status_model(ui.cx_mut());
        let outliner_items = proof_outliner_items_snapshot(ui.cx_mut().app, &outliner_items_model);
        let mut pending_reorder: Option<(
            Arc<str>,
            Arc<str>,
            Arc<str>,
            Arc<str>,
            SortableInsertionSide,
        )> = None;
        let mut preview_status: Option<String> = None;

        let _ = ui.tree_node_with_options(
            "imui-editor-proof.authoring.imui.outliner.reorder.scene",
            "Scene",
            kit::TreeNodeOptions {
                default_open: true,
                test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.outliner.reorder.scene",
                )),
                content_test_id: Some(Arc::from(
                    "imui-editor-proof.authoring.imui.outliner.reorder.scene.content",
                )),
                ..Default::default()
            },
            |ui| {
                for item in &outliner_items {
                    let row = ui.tree_node_with_options(
                        item.id.as_ref(),
                        item.label.clone(),
                        kit::TreeNodeOptions {
                            leaf: true,
                            level: 2,
                            test_id: Some(Arc::from(format!(
                                "imui-editor-proof.authoring.imui.outliner.reorder.row.{}",
                                item.id
                            ))),
                            ..Default::default()
                        },
                        |_ui| {},
                    );

                    let payload = ProofOutlinerDragItem {
                        id: item.id.clone(),
                        label: item.label.clone(),
                    };
                    let sortable = sortable_row(ui, row.response(), payload);
                    let ghost_id = format!(
                        "imui-editor-proof.authoring.imui.outliner.reorder.row.{}.ghost",
                        item.id
                    );
                    let _ = drag_preview_ghost_with_options(
                        ui,
                        ghost_id.as_str(),
                        sortable.source(),
                        DragPreviewGhostOptions {
                            test_id: Some(Arc::from(format!(
                                "imui-editor-proof.authoring.imui.outliner.reorder.row.{}.ghost",
                                item.id
                            ))),
                            ..Default::default()
                        },
                        proof_drag_preview_card(item.label.clone(), None),
                    );

                    if let Some(signal) = sortable.delivered_reorder() {
                        let dragged = signal.payload();
                        if dragged.id != item.id {
                            pending_reorder = Some((
                                dragged.id.clone(),
                                dragged.label.clone(),
                                item.id.clone(),
                                item.label.clone(),
                                signal.side(),
                            ));
                        }
                    } else if let Some(signal) = sortable.preview_reorder() {
                        let dragged = signal.payload();
                        let side = signal.side();
                        if dragged.id != item.id {
                            preview_status = Some(format!(
                                "Preview: move {} {} {}",
                                dragged.label,
                                side.label(),
                                item.label
                            ));
                        }
                    }
                }
            },
        );

        if let Some((active_id, active_label, over_id, over_label, side)) = pending_reorder {
            let moved = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&outliner_items_model, |items| {
                    reorder_vec_by_key(items, active_id.as_ref(), over_id.as_ref(), side, |item| {
                        item.id.as_ref()
                    })
                })
                .unwrap_or(false);
            let next_status = if moved {
                format!("Moved {} {} {}", active_label, side.label(), over_label)
            } else {
                "Drop ignored".to_string()
            };
            let _ = ui
                .cx_mut()
                .app
                .models_mut()
                .update(&outliner_status_model, |status| {
                    status.clear();
                    status.push_str(&next_status);
                });
        }

        let outliner_order =
            proof_outliner_order_line_for_model(ui.cx_mut().app, &outliner_items_model);
        let persisted_outliner_status =
            editor_string_model_readout(ui.cx_mut(), &outliner_status_model);
        let visible_outliner_status = preview_status.unwrap_or_else(|| persisted_outliner_status);
        ui.text(outliner_order);
        ui.text(format!("Status: {visible_outliner_status}"));
    })
}

fn build_authoring_parity_gradient_editor(
    cx: &mut AppComponentCx<'_>,
    angle_model: Model<f64>,
    stops_model: Model<Vec<GradientDemoStop>>,
    next_id_model: Model<u64>,
    id_source: &'static str,
    test_id_prefix: &'static str,
) -> GradientEditor {
    let stops = cx.data().selector_model_paint(&stops_model, |stops| stops);

    let on_remove: fret_ui_editor::composites::OnGradientStopAction = Arc::new({
        let stops_model = stops_model.clone();
        move |host, action_cx, stop_id| {
            let _ = host
                .models_mut()
                .update(&stops_model, |stops| stops.retain(|s| s.id != stop_id));
            host.request_redraw(action_cx.window);
        }
    });

    let on_add: fret_ui_editor::composites::OnGradientAction = Arc::new({
        let stops_model = stops_model.clone();
        move |host, action_cx| {
            let id = host
                .models_mut()
                .update(&next_id_model, |v| {
                    let out = *v;
                    *v = v.saturating_add(1);
                    out
                })
                .unwrap_or(1);

            let position = host.models_mut().insert(0.5_f64);
            let color = host.models_mut().insert(Color {
                r: 0.85,
                g: 0.85,
                b: 0.85,
                a: 1.0,
            });
            let stop = GradientDemoStop {
                id,
                position,
                color,
            };

            let _ = host
                .models_mut()
                .update(&stops_model, |stops| stops.push(stop));
            host.request_redraw(action_cx.window);
        }
    });

    let bindings: Arc<[GradientStopBinding]> = stops
        .into_iter()
        .map(|s| GradientStopBinding {
            id: s.id,
            position: s.position,
            color: s.color,
            remove: Some(on_remove.clone()),
        })
        .collect::<Vec<_>>()
        .into();

    GradientEditor::new(bindings)
        .angle_degrees(Some(angle_model))
        .on_add_stop(Some(on_add))
        .options(GradientEditorOptions {
            id_source: Some(Arc::from(id_source)),
            test_id: Some(Arc::from(test_id_prefix)),
            preview_test_id: Some(Arc::<str>::from(format!("{test_id_prefix}.preview"))),
            stops_test_id: Some(Arc::<str>::from(format!("{test_id_prefix}.stops"))),
            add_stop_test_id: Some(Arc::<str>::from(format!("{test_id_prefix}.add-stop"))),
            ..Default::default()
        })
}

fn render_authoring_parity_imui_host<H, F>(
    cx: &mut ElementContext<'_, H>,
    f: F,
) -> impl IntoUiElement<H> + use<H, F>
where
    H: UiHost,
    F: for<'cx, 'a> FnOnce(&mut ImUi<'cx, 'a, H>) + 'static,
{
    // Authoring-parity IMUI content can emit multiple siblings, so the proof-local host must own
    // vertical flow explicitly instead of forwarding them through a non-layout container box.
    fret_ui_kit::ui::v_flex_build(move |cx, out| {
        imui_build(cx, out, f);
    })
    .w_full()
    .into_element(cx)
}

fn authoring_parity_shading_items() -> Arc<[EnumSelectItem]> {
    vec![
        EnumSelectItem::new("lit", "Lit"),
        EnumSelectItem::new("unlit", "Unlit"),
        EnumSelectItem::new("matcap", "Matcap"),
    ]
    .into()
}
