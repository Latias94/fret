use std::sync::Arc;

use fret::advanced::KernelApp;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui::element::AnyElement;
use fret_ui_editor::composites::{
    InspectorPanelCx, PropertyGrid, PropertyGroup, PropertyGroupOptions, PropertyRow,
    PropertyRowReset, PropertyRowResetOptions,
};
use fret_ui_editor::controls::{
    DragValue, FieldStatus, FieldStatusBadge, NumericFormatFn, NumericInput, NumericInputOptions,
    NumericParseFn, NumericValidateFn, TransformEdit, TransformEditAxisOutcome,
    TransformEditOptions, Vec3Edit, VecEditAxisOutcome, VecEditOptions,
};
use fret_ui_kit::IntoUiElement as _;

use super::super::editor_model_owner::EditorProofModelOwner;
use super::super::editor_state::{
    editor_demo_exposure_model, editor_demo_iterations_model, editor_demo_position_models,
    editor_demo_position_outcome_model, editor_demo_rotation_models, editor_demo_scale_models,
    editor_demo_transform_outcome_model,
};
use super::super::proof_helpers::{
    editor_fixed_decimals_presentation, editor_position_presentation, editor_string_model_readout,
    editor_transform_presentations, proof_empty_state_text, proof_optional_outcome_readout,
};

#[derive(Clone)]
pub struct EditorAdvancedModels {
    pub pos_x: Model<f64>,
    pub pos_y: Model<f64>,
    pub pos_z: Model<f64>,
    pub position_outcome: Model<String>,
    pub rot_x: Model<f64>,
    pub rot_y: Model<f64>,
    pub rot_z: Model<f64>,
    pub scl_x: Model<f64>,
    pub scl_y: Model<f64>,
    pub scl_z: Model<f64>,
    pub transform_outcome: Model<String>,
    pub iterations: Model<i32>,
    pub exposure: Model<f64>,
}

pub struct EditorAdvancedSurface {
    pub element: Option<AnyElement>,
    pub any_match: bool,
}

fn editor_advanced_models(cx: &mut ElementContext<'_, KernelApp>) -> EditorAdvancedModels {
    let (pos_x, pos_y, pos_z) = editor_demo_position_models(cx);
    let (rot_x, rot_y, rot_z) = editor_demo_rotation_models(cx);
    let (scl_x, scl_y, scl_z) = editor_demo_scale_models(cx);

    EditorAdvancedModels {
        pos_x,
        pos_y,
        pos_z,
        position_outcome: editor_demo_position_outcome_model(cx),
        rot_x,
        rot_y,
        rot_z,
        scl_x,
        scl_y,
        scl_z,
        transform_outcome: editor_demo_transform_outcome_model(cx),
        iterations: editor_demo_iterations_model(cx),
        exposure: editor_demo_exposure_model(cx),
    }
}

#[derive(Clone, Copy)]
struct EditorAdvancedVisibility {
    position: bool,
    transform: bool,
    iterations: bool,
    exposure: bool,
}

impl EditorAdvancedVisibility {
    fn from_panel(panel_cx: &InspectorPanelCx) -> Self {
        let advanced_show_all = panel_cx.matches("advanced");
        Self {
            position: advanced_show_all || panel_cx.matches("position") || panel_cx.matches("pos"),
            transform: advanced_show_all
                || panel_cx.matches("transform")
                || panel_cx.matches("xform")
                || panel_cx.matches("rotation")
                || panel_cx.matches("rot")
                || panel_cx.matches("scale"),
            iterations: advanced_show_all || panel_cx.matches("iterations"),
            exposure: advanced_show_all
                || panel_cx.matches("exposure")
                || panel_cx.matches("validate"),
        }
    }

    fn any_match(self) -> bool {
        self.position || self.transform || self.iterations || self.exposure
    }
}

pub fn render_editor_advanced_surface(
    cx: &mut ElementContext<'_, KernelApp>,
    panel_cx: &InspectorPanelCx,
) -> EditorAdvancedSurface {
    let visibility = EditorAdvancedVisibility::from_panel(panel_cx);
    let any_match = visibility.any_match();
    let element = if any_match {
        let models = editor_advanced_models(cx);
        Some(
            PropertyGroup::new("Advanced")
                .options(PropertyGroupOptions {
                    collapsible: false,
                    test_id: Some(Arc::from("imui-editor-proof.editor.group.advanced")),
                    header_test_id: Some(Arc::from(
                        "imui-editor-proof.editor.group.advanced.header",
                    )),
                    ..Default::default()
                })
                .into_element(
                    cx,
                    |_cx| None,
                    move |cx| render_editor_advanced_rows(cx, visibility, models),
                ),
        )
    } else {
        None
    };

    EditorAdvancedSurface { element, any_match }
}

fn render_editor_advanced_rows(
    cx: &mut ElementContext<'_, KernelApp>,
    visibility: EditorAdvancedVisibility,
    models: EditorAdvancedModels,
) -> Vec<AnyElement> {
    let validate = advanced_unit_interval_validate();
    let fixed_presentation = editor_fixed_decimals_presentation();
    let position_presentation = editor_position_presentation();
    let transform_presentations = editor_transform_presentations();
    let fmt_i32: NumericFormatFn<i32> = Arc::new(|v| Arc::from(format!("{v}")));
    let parse_i32: NumericParseFn<i32> = Arc::new(|s| s.trim().parse::<i32>().ok());

    vec![PropertyGrid::new().into_element(cx, move |cx, row_cx| {
        let mut rows = Vec::new();

        if visibility.position {
            let on_reset = reset_position_action(
                models.pos_x.clone(),
                models.pos_y.clone(),
                models.pos_z.clone(),
            );
            rows.push(row_cx.row_with(
                cx,
                PropertyRow::new().reset(Some(reset_button(
                    on_reset,
                    "imui-editor-proof.editor.advanced.position.reset",
                ))),
                |cx| row_cx.label_text(cx, "Position"),
                |cx| {
                    let outcome_model = models.position_outcome.clone();
                    Vec3Edit::from_presentation(
                        models.pos_x.clone(),
                        models.pos_y.clone(),
                        models.pos_z.clone(),
                        position_presentation.clone(),
                    )
                    .on_axis_outcome(Some(Arc::new(move |host, action_cx, outcome| {
                        record_vec_axis_outcome(host, action_cx, &outcome_model, outcome);
                    })))
                    .options(VecEditOptions {
                        test_id: Some(Arc::from("imui-editor-proof.editor.advanced.position")),
                        ..Default::default()
                    })
                    .into_element(cx)
                },
                |cx| {
                    let outcome = editor_string_model_readout(cx, &models.position_outcome);
                    proof_optional_outcome_readout(
                        cx,
                        outcome,
                        Arc::from("imui-editor-proof.editor.advanced.position.outcome"),
                    )
                },
            ));
        }

        if visibility.transform {
            let on_reset = reset_transform_action(
                (
                    models.pos_x.clone(),
                    models.pos_y.clone(),
                    models.pos_z.clone(),
                ),
                (
                    models.rot_x.clone(),
                    models.rot_y.clone(),
                    models.rot_z.clone(),
                ),
                (
                    models.scl_x.clone(),
                    models.scl_y.clone(),
                    models.scl_z.clone(),
                ),
            );
            rows.push(row_cx.row_with(
                cx,
                PropertyRow::new().reset(Some(reset_button(
                    on_reset,
                    "imui-editor-proof.editor.advanced.transform.reset",
                ))),
                |cx| row_cx.label_text(cx, "Transform"),
                |cx| {
                    let outcome_model = models.transform_outcome.clone();
                    TransformEdit::from_presentations(
                        (
                            models.pos_x.clone(),
                            models.pos_y.clone(),
                            models.pos_z.clone(),
                        ),
                        (
                            models.rot_x.clone(),
                            models.rot_y.clone(),
                            models.rot_z.clone(),
                        ),
                        (
                            models.scl_x.clone(),
                            models.scl_y.clone(),
                            models.scl_z.clone(),
                        ),
                        transform_presentations.clone(),
                    )
                    .on_axis_outcome(Some(Arc::new(move |host, action_cx, outcome| {
                        record_transform_axis_outcome(host, action_cx, &outcome_model, outcome);
                    })))
                    .options(TransformEditOptions {
                        test_id: Some(Arc::from("imui-editor-proof.editor.advanced.transform")),
                        link_test_id: Some(Arc::from(
                            "imui-editor-proof.editor.advanced.transform.link-scale",
                        )),
                        ..Default::default()
                    })
                    .into_element(cx)
                },
                |cx| {
                    let outcome = editor_string_model_readout(cx, &models.transform_outcome);
                    proof_optional_outcome_readout(
                        cx,
                        outcome,
                        Arc::from("imui-editor-proof.editor.advanced.transform.outcome"),
                    )
                },
            ));
        }

        if visibility.iterations {
            let on_reset = reset_i32_action(models.iterations.clone(), 8);
            rows.push(row_cx.row_with(
                cx,
                PropertyRow::new().reset(Some(reset_button(
                    on_reset,
                    "imui-editor-proof.editor.advanced.iterations.reset",
                ))),
                |cx| row_cx.label_text(cx, "Iterations"),
                |cx| {
                    DragValue::new(
                        models.iterations.clone(),
                        fmt_i32.clone(),
                        parse_i32.clone(),
                    )
                    .options(fret_ui_editor::controls::DragValueOptions {
                        test_id: Some(Arc::from("imui-editor-proof.editor.advanced.iterations")),
                        ..Default::default()
                    })
                    .into_element(cx)
                },
                |cx| {
                    Some(
                        FieldStatusBadge::new(FieldStatus::Error(Arc::from("stub")))
                            .into_element(cx),
                    )
                },
            ));
        }

        if visibility.exposure {
            let on_reset = reset_f64_action(models.exposure.clone(), 0.75);
            rows.push(row_cx.row_with(
                cx,
                PropertyRow::new().reset(Some(reset_button(
                    on_reset,
                    "imui-editor-proof.editor.advanced.exposure.reset",
                ))),
                |cx| row_cx.label_text(cx, "Exposure"),
                |cx| {
                    NumericInput::from_presentation(
                        models.exposure.clone(),
                        fixed_presentation.clone(),
                    )
                    .validate(Some(validate.clone()))
                    .options(NumericInputOptions {
                        test_id: Some(Arc::from("imui-editor-proof.editor.advanced.exposure")),
                        ..Default::default()
                    })
                    .into_element(cx)
                },
                |_cx| None,
            ));
        }

        if rows.is_empty() {
            rows.push(proof_empty_state_text(
                cx,
                "No matches",
                "imui-editor-proof.editor.advanced.no-matches",
            ));
        }

        rows
    })]
}

fn advanced_unit_interval_validate() -> NumericValidateFn<f64> {
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
        EditorProofModelOwner::new(host.models_mut()).set_f64(&model, value);
        host.request_redraw(action_cx.window);
    })
}

fn reset_i32_action(
    model: Model<i32>,
    value: i32,
) -> Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static> {
    Arc::new(move |host, action_cx| {
        EditorProofModelOwner::new(host.models_mut()).set_i32(&model, value);
        host.request_redraw(action_cx.window);
    })
}

fn reset_position_action(
    x: Model<f64>,
    y: Model<f64>,
    z: Model<f64>,
) -> Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static> {
    Arc::new(move |host, action_cx| {
        let mut owner = EditorProofModelOwner::new(host.models_mut());
        owner.set_f64(&x, 0.0);
        owner.set_f64(&y, 0.0);
        owner.set_f64(&z, 0.0);
        host.request_redraw(action_cx.window);
    })
}

fn reset_transform_action(
    position: (Model<f64>, Model<f64>, Model<f64>),
    rotation: (Model<f64>, Model<f64>, Model<f64>),
    scale: (Model<f64>, Model<f64>, Model<f64>),
) -> Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static> {
    Arc::new(move |host, action_cx| {
        let mut owner = EditorProofModelOwner::new(host.models_mut());
        owner.set_f64(&position.0, 0.0);
        owner.set_f64(&position.1, 0.0);
        owner.set_f64(&position.2, 0.0);
        owner.set_f64(&rotation.0, 0.0);
        owner.set_f64(&rotation.1, 0.0);
        owner.set_f64(&rotation.2, 0.0);
        owner.set_f64(&scale.0, 1.0);
        owner.set_f64(&scale.1, 1.0);
        owner.set_f64(&scale.2, 1.0);
        host.request_redraw(action_cx.window);
    })
}

fn record_vec_axis_outcome(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    outcome_model: &Model<String>,
    outcome: VecEditAxisOutcome,
) {
    EditorProofModelOwner::new(host.models_mut()).record_vec_axis_outcome(outcome_model, outcome);
    host.request_redraw(action_cx.window);
}

fn record_transform_axis_outcome(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    outcome_model: &Model<String>,
    outcome: TransformEditAxisOutcome,
) {
    EditorProofModelOwner::new(host.models_mut())
        .record_transform_axis_outcome(outcome_model, outcome);
    host.request_redraw(action_cx.window);
}
