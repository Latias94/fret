use std::sync::Arc;

use fret_core::Color;
use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};
use fret_ui_editor::controls::EnumSelectItem;

pub(super) fn editor_material_shading_items() -> Arc<[EnumSelectItem]> {
    vec![
        EnumSelectItem::new("lit", "Lit"),
        EnumSelectItem::new("unlit", "Unlit"),
        EnumSelectItem::new("subsurface", "Subsurface"),
        EnumSelectItem::new("clearcoat", "Clearcoat"),
        EnumSelectItem::new("sheen", "Sheen"),
        EnumSelectItem::new("anisotropy", "Anisotropy"),
        EnumSelectItem::new("iridescence", "Iridescence"),
        EnumSelectItem::new("transmission", "Transmission"),
        EnumSelectItem::new("specular-gloss", "Specular gloss"),
        EnumSelectItem::new("matcap", "Matcap"),
        EnumSelectItem::new("toon", "Toon"),
        EnumSelectItem::new("cloth", "Cloth"),
    ]
    .into()
}

pub(super) fn named_demo_state<H: UiHost, S: Clone + 'static>(
    cx: &mut ElementContext<'_, H>,
    name: &'static str,
    init: impl FnOnce(&mut ElementContext<'_, H>) -> S,
) -> S {
    cx.named(name, |cx| {
        let slot = cx.slot_id();
        let existing = cx.state_for(slot, || None::<S>, |st| st.clone());
        match existing {
            Some(v) => v,
            None => {
                let v = init(cx);
                cx.state_for(
                    slot,
                    || None::<S>,
                    |st| {
                        if st.is_none() {
                            *st = Some(v.clone());
                        }
                        st.clone()
                            .expect("named_demo_state slot must contain a value after init")
                    },
                )
            }
        }
    })
}

pub(super) fn editor_demo_value_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.value", |cx| {
        cx.app.models_mut().insert(0.8_f64)
    })
}

pub(super) fn editor_demo_roughness_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.roughness", |cx| {
        cx.app.models_mut().insert(0.35_f64)
    })
}

pub(super) fn editor_demo_metallic_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.metallic", |cx| {
        cx.app.models_mut().insert(0.1_f64)
    })
}

#[derive(Clone)]
pub(super) struct GradientDemoStop {
    pub(super) id: fret_ui::ItemKey,
    pub(super) position: Model<f64>,
    pub(super) color: Model<Color>,
}

pub(super) fn editor_demo_gradient_angle_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<f64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.gradient_angle", |cx| {
        cx.app.models_mut().insert(45.0_f64)
    })
}

pub(super) fn editor_demo_gradient_stops_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Vec<GradientDemoStop>> {
    named_demo_state(cx, "imui_editor_proof_demo.model.gradient_stops", |cx| {
        let stop_0_pos = cx.app.models_mut().insert(0.0_f64);
        let stop_0_color = cx.app.models_mut().insert(Color {
            a: 1.0,
            ..Color::from_srgb_hex_rgb(0xf2_59_33)
        });
        let stop_1_pos = cx.app.models_mut().insert(1.0_f64);
        let stop_1_color = cx.app.models_mut().insert(Color {
            a: 1.0,
            ..Color::from_srgb_hex_rgb(0x33_73_f2)
        });
        cx.app.models_mut().insert(vec![
            GradientDemoStop {
                id: 1,
                position: stop_0_pos,
                color: stop_0_color,
            },
            GradientDemoStop {
                id: 2,
                position: stop_1_pos,
                color: stop_1_color,
            },
        ])
    })
}

pub(super) fn editor_demo_gradient_next_id_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<u64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.gradient_next_id", |cx| {
        cx.app.models_mut().insert(3_u64)
    })
}

pub(super) fn editor_demo_base_color_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Color> {
    named_demo_state(cx, "imui_editor_proof_demo.model.base_color", |cx| {
        cx.app.models_mut().insert(Color {
            r: 0.9,
            g: 0.2,
            b: 0.2,
            a: 1.0,
        })
    })
}

pub(super) fn editor_demo_position_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<f64>, Model<f64>, Model<f64>) {
    named_demo_state(cx, "imui_editor_proof_demo.model.position", |cx| {
        let x = cx.app.models_mut().insert(0.0_f64);
        let y = cx.app.models_mut().insert(1.0_f64);
        let z = cx.app.models_mut().insert(0.0_f64);
        (x, y, z)
    })
}

pub(super) fn editor_demo_rotation_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<f64>, Model<f64>, Model<f64>) {
    named_demo_state(cx, "imui_editor_proof_demo.model.rotation", |cx| {
        let x = cx.app.models_mut().insert(0.0_f64);
        let y = cx.app.models_mut().insert(0.0_f64);
        let z = cx.app.models_mut().insert(0.0_f64);
        (x, y, z)
    })
}

pub(super) fn editor_demo_scale_models<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> (Model<f64>, Model<f64>, Model<f64>) {
    named_demo_state(cx, "imui_editor_proof_demo.model.scale", |cx| {
        let x = cx.app.models_mut().insert(1.0_f64);
        let y = cx.app.models_mut().insert(1.0_f64);
        let z = cx.app.models_mut().insert(1.0_f64);
        (x, y, z)
    })
}

pub(super) fn editor_demo_alpha_clip_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<bool> {
    named_demo_state(cx, "imui_editor_proof_demo.model.alpha_clip", |cx| {
        cx.app.models_mut().insert(false)
    })
}

pub(super) fn editor_demo_cast_shadows_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<bool>> {
    named_demo_state(cx, "imui_editor_proof_demo.model.cast_shadows", |cx| {
        // Start in "mixed/indeterminate" to exercise tri-state checkbox rendering.
        cx.app.models_mut().insert(None::<bool>)
    })
}

pub(super) fn editor_demo_shading_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Arc<str>>> {
    named_demo_state(cx, "imui_editor_proof_demo.model.shading_model", |cx| {
        cx.app
            .models_mut()
            .insert(Some::<Arc<str>>(Arc::from("cloth")))
    })
}

pub(super) fn editor_demo_iterations_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<i32> {
    named_demo_state(cx, "imui_editor_proof_demo.model.iterations", |cx| {
        cx.app.models_mut().insert(16_i32)
    })
}

pub(super) fn editor_demo_exposure_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<f64> {
    named_demo_state(cx, "imui_editor_proof_demo.model.exposure", |cx| {
        cx.app.models_mut().insert(0.75_f64)
    })
}

pub(super) fn editor_demo_search_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.search", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

pub(super) fn editor_demo_search_assist_dismissed_query_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.search_assist_dismissed_query",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

pub(super) fn editor_demo_search_assist_active_item_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Arc<str>>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.search_assist_active_item",
        |cx| cx.app.models_mut().insert(None::<Arc<str>>),
    )
}

pub(super) fn editor_demo_name_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.name", |cx| {
        cx.app.models_mut().insert("Cube".to_string())
    })
}

pub(super) fn editor_demo_buffered_name_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.buffered_name", |cx| {
        cx.app.models_mut().insert("Buffered Cube".to_string())
    })
}

pub(super) fn editor_demo_inline_rename_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.inline_rename", |cx| {
        cx.app.models_mut().insert("Props_Root".to_string())
    })
}

pub(super) fn editor_demo_name_assist_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.name_assist", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

pub(super) fn editor_demo_name_assist_dismissed_query_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.name_assist_dismissed_query",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

pub(super) fn editor_demo_name_assist_active_item_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<Option<Arc<str>>> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.name_assist_active_item",
        |cx| cx.app.models_mut().insert(None::<Arc<str>>),
    )
}

pub(super) fn editor_demo_name_assist_accepted_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.name_assist_accepted",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

pub(super) fn editor_demo_password_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.password", |cx| {
        cx.app.models_mut().insert("secret42".to_string())
    })
}

pub(super) fn editor_demo_drag_value_outcome_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.drag_value_outcome",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

pub(super) fn editor_demo_password_outcome_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.password_outcome", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

pub(super) fn editor_demo_inline_rename_outcome_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(
        cx,
        "imui_editor_proof_demo.model.inline_rename_outcome",
        |cx| cx.app.models_mut().insert(String::new()),
    )
}

pub(super) fn editor_demo_notes_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.notes", |cx| {
        cx.app
            .models_mut()
            .insert("Multiline TextField (v1)\n- uses TextArea\n- clear affordance\n".to_string())
    })
}

pub(super) fn editor_demo_notes_outcome_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.notes_outcome", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

pub(super) fn editor_demo_position_outcome_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.position_outcome", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}

pub(super) fn editor_demo_transform_outcome_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<String> {
    named_demo_state(cx, "imui_editor_proof_demo.model.transform_outcome", |cx| {
        cx.app.models_mut().insert(String::new())
    })
}
