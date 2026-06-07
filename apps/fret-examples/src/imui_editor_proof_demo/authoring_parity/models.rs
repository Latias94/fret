use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use super::super::proof_helpers::{ProofDragAsset, ProofOutlinerItem};

pub(in super::super) fn name_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    super::super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.name",
        |cx| cx.app.models_mut().insert("Shared Cube".to_string()),
    )
}

pub(in super::super) fn drag_value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    super::super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.drag_value",
        |cx| cx.app.models_mut().insert(1.250_f64),
    )
}

pub(in super::super) fn numeric_input_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<f64> {
    super::super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.numeric_input",
        |cx| cx.app.models_mut().insert(0.875_f64),
    )
}

pub(in super::super) fn slider_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    super::super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.slider",
        |cx| cx.app.models_mut().insert(0.35_f64),
    )
}

pub(in super::super) fn enabled_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    super::super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.enabled",
        |cx| cx.app.models_mut().insert(true),
    )
}

pub(in super::super) fn shading_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Arc<str>>> {
    super::super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.shading",
        |cx| {
            cx.app
                .models_mut()
                .insert(Some::<Arc<str>>(Arc::from("lit")))
        },
    )
}

pub(in super::super) fn gradient_angle_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<f64> {
    super::super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.gradient_angle",
        |cx| cx.app.models_mut().insert(90.0_f64),
    )
}

pub(in super::super) fn gradient_stops_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Vec<super::super::GradientDemoStop>> {
    super::super::named_demo_state(
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
                super::super::GradientDemoStop {
                    id: 1,
                    position: stop_0_pos,
                    color: stop_0_color,
                },
                super::super::GradientDemoStop {
                    id: 2,
                    position: stop_1_pos,
                    color: stop_1_color,
                },
            ])
        },
    )
}

pub(in super::super) fn gradient_next_id_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<u64> {
    super::super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.gradient_next_id",
        |cx| cx.app.models_mut().insert(3_u64),
    )
}

pub(in super::super) fn asset_slot_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    super::super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.asset_slot",
        |cx| {
            cx.app
                .models_mut()
                .insert(super::super::asset_ref::DEFAULT_ASSET.to_string())
        },
    )
}

pub(in super::super) fn drag_assets() -> Arc<[ProofDragAsset]> {
    super::super::collection::authoring_parity_collection_assets()
        .iter()
        .take(3)
        .map(|asset| ProofDragAsset {
            label: asset.label.clone(),
            path: asset.path.clone(),
        })
        .collect::<Vec<_>>()
        .into()
}

pub(in super::super) fn outliner_items() -> Arc<[ProofOutlinerItem]> {
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

pub(in super::super) fn outliner_items_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Vec<ProofOutlinerItem>> {
    super::super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.outliner_items",
        |cx| {
            cx.app
                .models_mut()
                .insert(outliner_items().iter().cloned().collect())
        },
    )
}

pub(in super::super) fn outliner_status_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    super::super::named_demo_state(
        cx,
        "imui_editor_proof_demo.model.authoring_parity.outliner_status",
        |cx| cx.app.models_mut().insert("Idle".to_string()),
    )
}
