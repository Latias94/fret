use std::sync::Arc;

use fret::AppComponentCx;
use fret::advanced::KernelApp;
use fret::component::prelude::*;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::AnyElement;
use fret_ui_editor::composites::{
    InspectorPanelCx, PropertyGrid, PropertyGroup, PropertyGroupOptions, PropertyRow,
    PropertyRowReset, PropertyRowResetOptions,
};
use fret_ui_editor::controls::{
    Checkbox, CheckboxOptions, ColorEdit, ColorEditOptions, DragValue, DragValueOutcome,
    EnumSelect, EnumSelectOptions, FieldStatus, FieldStatusBadge, NumericValidateFn,
    NumericValueConstraints, Slider, SliderOptions,
};

use super::asset_ref;
use super::editor_state::editor_material_shading_items;
use super::proof_helpers::{
    compact_edit_session_outcome_label, editor_fixed_decimals_presentation,
    editor_percent_presentation, editor_string_model_readout, proof_empty_state_text,
    proof_optional_outcome_readout,
};

#[derive(Clone)]
pub(super) struct EditorMaterialModels {
    pub(super) opacity: Model<f64>,
    pub(super) opacity_outcome: Model<String>,
    pub(super) roughness: Model<f64>,
    pub(super) metallic: Model<f64>,
    pub(super) base_color: Model<fret_core::Color>,
    pub(super) asset_slot: Model<String>,
    pub(super) asset_action: Model<String>,
    pub(super) shading: Model<Option<Arc<str>>>,
    pub(super) alpha_clip: Model<bool>,
    pub(super) cast_shadows: Model<Option<bool>>,
}

pub(super) struct EditorMaterialSurface {
    pub(super) element: AnyElement,
    pub(super) any_match: bool,
}

pub(super) fn render_editor_material_surface(
    cx: &mut ElementContext<'_, KernelApp>,
    panel_cx: &InspectorPanelCx,
    models: EditorMaterialModels,
) -> EditorMaterialSurface {
    let visibility = EditorMaterialVisibility::from_panel(panel_cx);
    let element = PropertyGroup::new("Material")
        .options(PropertyGroupOptions {
            test_id: Some(Arc::from("imui-editor-proof.editor.group.material")),
            header_test_id: Some(Arc::from("imui-editor-proof.editor.group.material.header")),
            content_test_id: Some(Arc::from("imui-editor-proof.editor.group.material.content")),
            ..Default::default()
        })
        .into_element(
            cx,
            |_cx| None,
            move |cx| render_editor_material_rows(cx, visibility, models),
        );

    EditorMaterialSurface {
        element,
        any_match: visibility.any_match(),
    }
}

#[derive(Clone, Copy)]
struct EditorMaterialVisibility {
    opacity: bool,
    roughness: bool,
    metallic: bool,
    base_color: bool,
    asset_ref: bool,
    shading_model: bool,
    alpha_clip: bool,
    cast_shadows: bool,
}

impl EditorMaterialVisibility {
    fn from_panel(panel_cx: &InspectorPanelCx) -> Self {
        let material_show_all = panel_cx.matches("material");
        Self {
            opacity: material_show_all || panel_cx.matches("opacity"),
            roughness: material_show_all || panel_cx.matches("roughness"),
            metallic: material_show_all || panel_cx.matches("metallic"),
            base_color: material_show_all || panel_cx.matches("base") || panel_cx.matches("color"),
            asset_ref: material_show_all
                || panel_cx.matches("asset")
                || panel_cx.matches("texture")
                || panel_cx.matches("map")
                || panel_cx.matches("base"),
            shading_model: material_show_all
                || panel_cx.matches("shading")
                || panel_cx.matches("model"),
            alpha_clip: material_show_all || panel_cx.matches("alpha") || panel_cx.matches("clip"),
            cast_shadows: material_show_all
                || panel_cx.matches("shadow")
                || panel_cx.matches("shadows"),
        }
    }

    fn any_match(self) -> bool {
        self.opacity
            || self.roughness
            || self.metallic
            || self.base_color
            || self.asset_ref
            || self.shading_model
            || self.alpha_clip
            || self.cast_shadows
    }
}

fn render_editor_material_rows(
    cx: &mut ElementContext<'_, KernelApp>,
    visibility: EditorMaterialVisibility,
    models: EditorMaterialModels,
) -> Vec<AnyElement> {
    let validate = material_unit_interval_validate();
    let fixed_presentation = editor_fixed_decimals_presentation();

    vec![PropertyGrid::new().into_element(cx, move |cx, row_cx| {
        let mut rows = Vec::new();

        if visibility.opacity {
            let on_reset = reset_f64_action(models.opacity.clone(), 1.0);
            rows.push(row_cx.row_with(
                cx,
                PropertyRow::new().reset(Some(reset_button(
                    on_reset,
                    "imui-editor-proof.editor.drag-value-reset",
                ))),
                |cx| row_cx.label_text(cx, "Opacity"),
                |cx| {
                    let outcome_model = models.opacity_outcome.clone();
                    DragValue::from_presentation(models.opacity.clone(), fixed_presentation.clone())
                        .validate(Some(validate.clone()))
                        .on_outcome(Some(Arc::new(move |host, action_cx, outcome| {
                            record_drag_value_outcome(host, action_cx, &outcome_model, outcome);
                        })))
                        .options(fret_ui_editor::controls::DragValueOptions {
                            constraints: NumericValueConstraints {
                                min: Some(0.0),
                                max: Some(1.0),
                                clamp: true,
                                step: Some(0.025),
                            },
                            test_id: Some(Arc::from("imui-editor-proof.editor.drag-value-demo")),
                            ..Default::default()
                        })
                        .into_element(cx)
                },
                |cx| {
                    let outcome = editor_string_model_readout(cx, &models.opacity_outcome);
                    proof_optional_outcome_readout(
                        cx,
                        outcome,
                        Arc::from("imui-editor-proof.editor.drag-value-demo.outcome"),
                    )
                },
            ));
        }

        if visibility.roughness {
            let on_reset = reset_f64_action(models.roughness.clone(), 0.5);
            rows.push(row_cx.row_with(
                cx,
                PropertyRow::new().reset(Some(reset_button(
                    on_reset,
                    "imui-editor-proof.editor.material.roughness.reset",
                ))),
                |cx| row_cx.label_text(cx, "Roughness"),
                |cx| {
                    Slider::from_presentation(
                        models.roughness.clone(),
                        0.0,
                        1.0,
                        editor_percent_presentation(),
                    )
                    .options(SliderOptions {
                        a11y_label: Some(Arc::from("Roughness")),
                        step: Some(0.01),
                        test_id: Some(Arc::from("imui-editor-proof.editor.material.roughness")),
                        ..Default::default()
                    })
                    .into_element(cx)
                },
                |cx| Some(FieldStatusBadge::new(FieldStatus::Mixed).into_element(cx)),
            ));
        }

        if visibility.metallic {
            let on_reset = reset_f64_action(models.metallic.clone(), 0.0);
            rows.push(row_cx.row_with(
                cx,
                PropertyRow::new().reset(Some(reset_button(
                    on_reset,
                    "imui-editor-proof.editor.material.metallic.reset",
                ))),
                |cx| row_cx.label_text(cx, "Metallic"),
                |cx| {
                    Slider::from_presentation(
                        models.metallic.clone(),
                        0.0,
                        1.0,
                        editor_percent_presentation(),
                    )
                    .options(SliderOptions {
                        a11y_label: Some(Arc::from("Metallic")),
                        step: Some(0.01),
                        test_id: Some(Arc::from("imui-editor-proof.editor.material.metallic")),
                        ..Default::default()
                    })
                    .into_element(cx)
                },
                |cx| Some(FieldStatusBadge::new(FieldStatus::Loading).into_element(cx)),
            ));
        }

        if visibility.base_color {
            rows.push(row_cx.row(
                cx,
                |cx| row_cx.label_text(cx, "Base color"),
                |cx| {
                    ColorEdit::new(models.base_color.clone())
                        .options(ColorEditOptions {
                            test_id: Some(Arc::from(
                                "imui-editor-proof.editor.material.base-color",
                            )),
                            swatch_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.material.base-color.swatch",
                            )),
                            input_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.material.base-color.hex",
                            )),
                            popup_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.material.base-color.popup",
                            )),
                            ..Default::default()
                        })
                        .into_element(cx)
                },
            ));
        }

        if visibility.asset_ref {
            asset_ref::push_material_rows(
                &mut rows,
                cx,
                &row_cx,
                models.asset_slot.clone(),
                models.asset_action.clone(),
            );
        }

        if visibility.shading_model {
            rows.push(row_cx.row(
                cx,
                |cx| row_cx.label_text(cx, "Shading model"),
                |cx| {
                    EnumSelect::new(models.shading.clone(), editor_material_shading_items())
                        .options(EnumSelectOptions {
                            a11y_label: Some(Arc::from("Shading model")),
                            test_id: Some(Arc::from(
                                "imui-editor-proof.editor.material.shading-model",
                            )),
                            list_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.material.shading-model.list",
                            )),
                            search_test_id: Some(Arc::from(
                                "imui-editor-proof.editor.material.shading-model.search",
                            )),
                            max_list_height: Some(fret_core::Px(144.0)),
                            ..Default::default()
                        })
                        .into_element(cx)
                },
            ));
        }

        if visibility.alpha_clip {
            rows.push(row_cx.row(
                cx,
                |cx| row_cx.label_text(cx, "Alpha clip"),
                |cx| {
                    Checkbox::new(models.alpha_clip.clone())
                        .options(CheckboxOptions {
                            a11y_label: Some(Arc::from("Alpha clip")),
                            ..Default::default()
                        })
                        .into_element(cx)
                        .test_id("imui-editor-proof.editor.material.alpha-clip")
                },
            ));
        }

        if visibility.cast_shadows {
            rows.push(row_cx.row(
                cx,
                |cx| row_cx.label_text(cx, "Cast shadows"),
                |cx| {
                    Checkbox::new_optional(models.cast_shadows.clone())
                        .options(CheckboxOptions {
                            a11y_label: Some(Arc::from("Cast shadows")),
                            ..Default::default()
                        })
                        .into_element(cx)
                        .test_id("imui-editor-proof.editor.material.cast-shadows")
                },
            ));
        }

        if rows.is_empty() {
            rows.push(proof_empty_state_text(
                cx,
                "No matches",
                "imui-editor-proof.editor.material.no-matches",
            ));
        }

        rows
    })]
}

fn material_unit_interval_validate() -> NumericValidateFn<f64> {
    Arc::new(|v| {
        if (0.0..=1.0).contains(&v) {
            None
        } else {
            Some(Arc::from("Expected 0.0..=1.0"))
        }
    })
}

fn reset_button(
    on_reset: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>,
    test_id: &'static str,
) -> PropertyRowReset {
    PropertyRowReset::new(on_reset).options(PropertyRowResetOptions {
        test_id: Some(Arc::from(test_id)),
        ..Default::default()
    })
}

fn reset_f64_action(
    model: Model<f64>,
    value: f64,
) -> Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static> {
    Arc::new(move |host, action_cx| {
        let _ = host.models_mut().update(&model, |v| *v = value);
        host.request_redraw(action_cx.window);
    })
}

fn record_drag_value_outcome(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    outcome_model: &Model<String>,
    outcome: DragValueOutcome,
) {
    let next = compact_edit_session_outcome_label(outcome);
    let _ = host.models_mut().update(outcome_model, |value| {
        value.clear();
        value.push_str(next);
    });
    host.request_redraw(action_cx.window);
}
