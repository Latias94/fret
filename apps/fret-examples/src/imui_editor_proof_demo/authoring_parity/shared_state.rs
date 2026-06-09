use fret::AppComponentCx;
use fret::advanced::view::AppRenderDataExt as _;
use fret::component::prelude::*;

use super::super::proof_helpers::{
    AuthoringParitySharedStateReadout, proof_compact_readout_element,
};
use super::AuthoringParityModels;

pub(in super::super) fn render_shared_state(
    cx: &mut AppComponentCx<'_>,
    models: AuthoringParityModels,
) -> impl IntoUiElement<super::super::KernelApp> + use<> {
    let shared = cx.data().selector_model_paint(
        (
            &models.name,
            &models.drag_value,
            &models.numeric_input,
            &models.slider,
            &models.enabled,
            &models.shading,
            &models.gradient_angle,
            &models.gradient_stops,
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
