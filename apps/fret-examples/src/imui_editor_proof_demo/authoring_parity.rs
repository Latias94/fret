use std::sync::Arc;

use fret::AppComponentCx;
use fret::advanced::view::AppRenderDataExt as _;
use fret::component::prelude::*;
use fret_core::Color;
use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use super::proof_helpers::{
    AuthoringParitySharedStateReadout, ProofDragAsset, ProofOutlinerItem,
    proof_compact_readout_element,
};

pub(super) fn render_shared_state(
    cx: &mut AppComponentCx<'_>,
    name_model: Model<String>,
    drag_value_model: Model<f64>,
    numeric_input_model: Model<f64>,
    slider_model: Model<f64>,
    enabled_model: Model<bool>,
    shading_model: Model<Option<Arc<str>>>,
    gradient_angle_model: Model<f64>,
    gradient_stops_model: Model<Vec<super::GradientDemoStop>>,
) -> impl IntoUiElement<super::KernelApp> + use<> {
    let shared = cx.data().selector_model_paint(
        (
            &name_model,
            &drag_value_model,
            &numeric_input_model,
            &slider_model,
            &enabled_model,
            &shading_model,
            &gradient_angle_model,
            &gradient_stops_model,
        ),
        |(name, value, numeric, blend, enabled, shading, gradient_angle, gradient_stops)| {
            AuthoringParitySharedStateReadout {
                name_line: if name.trim().is_empty() {
                    "shared name: <empty>".to_string()
                } else {
                    format!("shared name: {name}")
                },
                value_line: format!("shared value: {value:.3}"),
                numeric_line: format!("shared typed numeric: {numeric:.3}"),
                blend_line: format!("shared blend: {:.0}%", blend * 100.0),
                enabled_line: format!("shared enabled: {enabled}"),
                shading_line: match shading.as_deref() {
                    Some("lit") => "shared mode: lit (Lit)".to_string(),
                    Some("unlit") => "shared mode: unlit (Unlit)".to_string(),
                    Some("matcap") => "shared mode: matcap (Matcap)".to_string(),
                    Some(other) => format!("shared mode: {other}"),
                    None => "shared mode: <none>".to_string(),
                },
                gradient_line: format!(
                    "shared gradient: {} stops @ {:.0}°",
                    gradient_stops.len(),
                    gradient_angle
                ),
            }
        },
    );
    let name_line = shared.name_line;
    let value_line = shared.value_line;
    let numeric_line = shared.numeric_line;
    let blend_line = shared.blend_line;
    let enabled_line = shared.enabled_line;
    let shading_line = shared.shading_line;
    let gradient_line = shared.gradient_line;

    fret_ui_kit::ui::v_flex_build(move |cx, out| {
        let name_line_row = name_line.clone();
        let value_line_row = value_line.clone();
        let numeric_line_row = numeric_line.clone();
        out.push(
            fret_ui_kit::ui::h_flex_build(move |cx, out| {
                out.push(proof_compact_readout_element(
                    cx,
                    name_line_row,
                    "imui-editor-proof.authoring.shared.name",
                ));
                out.push(proof_compact_readout_element(
                    cx,
                    value_line_row,
                    "imui-editor-proof.authoring.shared.value",
                ));
                out.push(proof_compact_readout_element(
                    cx,
                    numeric_line_row,
                    "imui-editor-proof.authoring.shared.numeric",
                ));
            })
            .gap(fret_ui_kit::Space::N3)
            .into_element(cx),
        );
        out.push(
            fret_ui_kit::ui::h_flex_build(move |cx, out| {
                out.push(proof_compact_readout_element(
                    cx,
                    blend_line,
                    "imui-editor-proof.authoring.shared.blend",
                ));
                out.push(proof_compact_readout_element(
                    cx,
                    enabled_line,
                    "imui-editor-proof.authoring.shared.enabled",
                ));
                out.push(proof_compact_readout_element(
                    cx,
                    shading_line,
                    "imui-editor-proof.authoring.shared.mode",
                ));
            })
            .gap(fret_ui_kit::Space::N3)
            .into_element(cx),
        );
        out.push(proof_compact_readout_element(
            cx,
            gradient_line,
            "imui-editor-proof.authoring.shared.gradient",
        ));
    })
    .gap(fret_ui_kit::Space::N1)
    .into_element(cx)
}

pub(super) fn name_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.name",
        |cx| cx.app.models_mut().insert("Shared Cube".to_string()),
    )
}

pub(super) fn drag_value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.drag_value",
        |cx| cx.app.models_mut().insert(1.250_f64),
    )
}

pub(super) fn numeric_input_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.numeric_input",
        |cx| cx.app.models_mut().insert(0.875_f64),
    )
}

pub(super) fn slider_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.slider",
        |cx| cx.app.models_mut().insert(0.35_f64),
    )
}

pub(super) fn enabled_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.enabled",
        |cx| cx.app.models_mut().insert(true),
    )
}

pub(super) fn shading_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.shading",
        |cx| {
            cx.app
                .models_mut()
                .insert(Some::<Arc<str>>(Arc::from("lit")))
        },
    )
}

pub(super) fn gradient_angle_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.gradient_angle",
        |cx| cx.app.models_mut().insert(90.0_f64),
    )
}

pub(super) fn gradient_stops_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Vec<super::GradientDemoStop>> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.gradient_stops",
        |cx| {
            let stop_0_pos = cx.app.models_mut().insert(0.0_f64);
            let stop_0_color = cx.app.models_mut().insert(Color {
                a: 1.0,
                ..Color::from_srgb_hex_rgb(0x14_b8_a6)
            });
            let stop_1_pos = cx.app.models_mut().insert(1.0_f64);
            let stop_1_color = cx.app.models_mut().insert(Color {
                a: 1.0,
                ..Color::from_srgb_hex_rgb(0xf9_73_16)
            });

            cx.app.models_mut().insert(vec![
                super::GradientDemoStop {
                    id: 1,
                    position: stop_0_pos,
                    color: stop_0_color,
                },
                super::GradientDemoStop {
                    id: 2,
                    position: stop_1_pos,
                    color: stop_1_color,
                },
            ])
        },
    )
}

pub(super) fn gradient_next_id_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<u64> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.gradient_next_id",
        |cx| cx.app.models_mut().insert(3_u64),
    )
}

pub(super) fn asset_slot_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.asset_slot",
        |cx| {
            cx.app
                .models_mut()
                .insert(super::asset_ref::DEFAULT_ASSET.to_string())
        },
    )
}

pub(super) fn drag_assets() -> Arc<[ProofDragAsset]> {
    super::collection::authoring_parity_collection_assets()
        .iter()
        .take(3)
        .map(|asset| ProofDragAsset {
            label: asset.label.clone(),
            path: asset.path.clone(),
        })
        .collect::<Vec<_>>()
        .into()
}

pub(super) fn outliner_items() -> Arc<[ProofOutlinerItem]> {
    vec![
        ProofOutlinerItem {
            id: Arc::from("camera"),
            label: Arc::from("Camera"),
        },
        ProofOutlinerItem {
            id: Arc::from("cube"),
            label: Arc::from("Cube"),
        },
        ProofOutlinerItem {
            id: Arc::from("key-light"),
            label: Arc::from("Key light"),
        },
        ProofOutlinerItem {
            id: Arc::from("post-fx"),
            label: Arc::from("Post FX"),
        },
    ]
    .into()
}

pub(super) fn outliner_items_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Vec<ProofOutlinerItem>> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.outliner_items",
        |cx| {
            cx.app
                .models_mut()
                .insert(outliner_items().iter().cloned().collect())
        },
    )
}

pub(super) fn outliner_status_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.outliner_status",
        |cx| cx.app.models_mut().insert("Idle".to_string()),
    )
}
