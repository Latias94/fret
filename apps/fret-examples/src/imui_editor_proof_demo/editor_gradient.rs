use std::sync::Arc;

use fret::advanced::KernelApp;
use fret::advanced::view::AppRenderDataExt as _;
use fret::component::prelude::*;
use fret_core::Color;
use fret_runtime::Model;
use fret_ui::ElementContext;
use fret_ui_editor::composites::{
    GradientEditor, GradientEditorOptions, GradientStopBinding, OnGradientAction,
    OnGradientStopAction, PropertyGroup, PropertyGroupOptions,
};

use super::editor_state::GradientDemoStop;

#[derive(Clone)]
pub(super) struct EditorGradientModels {
    pub(super) angle_degrees: Model<f64>,
    pub(super) stops: Model<Vec<GradientDemoStop>>,
    pub(super) next_id: Model<u64>,
}

pub(super) fn render_editor_gradient_surface(
    cx: &mut ElementContext<'_, KernelApp>,
    models: EditorGradientModels,
) -> impl IntoUiElement<KernelApp> + use<> {
    PropertyGroup::new("Gradient")
        .options(PropertyGroupOptions {
            test_id: Some(Arc::from("imui-editor-proof.editor.group.gradient")),
            header_test_id: Some(Arc::from("imui-editor-proof.editor.group.gradient.header")),
            content_test_id: Some(Arc::from("imui-editor-proof.editor.group.gradient.content")),
            ..Default::default()
        })
        .into_element(
            cx,
            |_cx| None,
            move |cx| vec![render_gradient_editor(cx, models)],
        )
}

fn render_gradient_editor(
    cx: &mut ElementContext<'_, KernelApp>,
    models: EditorGradientModels,
) -> fret_ui::element::AnyElement {
    let stops = cx.data().selector_model_paint(&models.stops, |stops| stops);
    let on_remove = remove_gradient_stop_action(models.stops.clone());
    let on_add = add_gradient_stop_action(models.stops.clone(), models.next_id.clone());
    let bindings = gradient_stop_bindings(stops, on_remove);

    GradientEditor::new(bindings)
        .angle_degrees(Some(models.angle_degrees.clone()))
        .on_add_stop(Some(on_add))
        .options(GradientEditorOptions {
            id_source: Some(Arc::from("imui_editor_proof_demo.gradient")),
            test_id: Some(Arc::from("imui-editor-proof.editor.gradient")),
            preview_test_id: Some(Arc::from("imui-editor-proof.editor.gradient.preview")),
            stops_test_id: Some(Arc::from("imui-editor-proof.editor.gradient.stops")),
            add_stop_test_id: Some(Arc::from("imui-editor-proof.editor.gradient.add-stop")),
            ..Default::default()
        })
        .into_element(cx)
}

fn remove_gradient_stop_action(stops_model: Model<Vec<GradientDemoStop>>) -> OnGradientStopAction {
    Arc::new(move |host, action_cx, stop_id| {
        let _ = host
            .models_mut()
            .update(&stops_model, |stops| stops.retain(|s| s.id != stop_id));
        host.request_redraw(action_cx.window);
    })
}

fn add_gradient_stop_action(
    stops_model: Model<Vec<GradientDemoStop>>,
    next_id_model: Model<u64>,
) -> OnGradientAction {
    Arc::new(move |host, action_cx| {
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
    })
}

fn gradient_stop_bindings(
    stops: Vec<GradientDemoStop>,
    on_remove: OnGradientStopAction,
) -> Arc<[GradientStopBinding]> {
    stops
        .into_iter()
        .map(|s| GradientStopBinding {
            id: s.id,
            position: s.position,
            color: s.color,
            remove: Some(on_remove.clone()),
        })
        .collect::<Vec<_>>()
        .into()
}
