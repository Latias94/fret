use std::sync::Arc;

use fret::AppComponentCx;
use fret::app::AppRenderDataExt as _;
use fret::imui::{ImUi, prelude::*};
use fret_core::Color;
use fret_runtime::Model;
use fret_ui::UiHost;
use fret_ui_editor::composites::{GradientEditor, GradientEditorOptions, GradientStopBinding};
use fret_ui_editor::controls::EnumSelectItem;
use fret_ui_kit::IntoUiElement;

use super::super::GradientDemoStop;
use super::super::proof_helpers::{
    authoring_parity_blend_presentation, authoring_parity_blend_slider_options,
    authoring_parity_drag_value_options, authoring_parity_numeric_input_options,
    authoring_parity_value_presentation,
};

pub(super) fn authoring_parity_shading_items() -> Arc<[EnumSelectItem]> {
    vec![
        EnumSelectItem::new("lit", "Lit"),
        EnumSelectItem::new("unlit", "Unlit"),
        EnumSelectItem::new("matcap", "Matcap"),
    ]
    .into()
}

pub(super) fn build_authoring_parity_gradient_editor(
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

pub(super) fn render_authoring_parity_imui_host<H, F>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    f: F,
) -> impl IntoUiElement<H> + use<H, F>
where
    H: UiHost,
    F: for<'cx, 'a> FnOnce(&mut ImUi<'cx, 'a, H>) + 'static,
{
    fret_ui_kit::ui::v_flex_build(move |cx, out| {
        imui_build(cx, out, f);
    })
    .w_full()
    .into_element(cx)
}
