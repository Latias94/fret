use super::*;

use fret_app::App;
use fret_authoring::UiWriter;
use fret_core::{AppWindowId, Px, Rect, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, Length};
use fret_ui::{ElementContext, UiHost, elements};

const BOOL_MODEL_RS_SOURCE: &str = include_str!("../bool_model.rs");
const FLOAT_MODEL_RS_SOURCE: &str = include_str!("../float_model.rs");
const OPTIONAL_TEXT_MODEL_RS_SOURCE: &str = include_str!("../optional_text_model.rs");
const TEXT_MODEL_RS_SOURCE: &str = include_str!("../text_model.rs");
const BOOLEAN_MODEL_SURFACE_RS_SOURCE: &str = include_str!("model_surface/boolean.rs");
const INPUT_TEXT_MODEL_SURFACE_RS_SOURCE: &str = include_str!("model_surface/text/input.rs");
const VALUE_COMBO_MODEL_SURFACE_RS_SOURCE: &str = include_str!("model_surface/value_combo.rs");
const CHECKBOX_WRAPPER_RS_SOURCE: &str = include_str!("boolean_wrappers/checkbox.rs");
const COMBO_WRAPPER_RS_SOURCE: &str = include_str!("value_models/combo_model.rs");
const INPUT_WRAPPER_RS_SOURCE: &str = include_str!("text_models/input.rs");
const SLIDER_WRAPPER_RS_SOURCE: &str = include_str!("value_models/slider.rs");
const SWITCH_WRAPPER_RS_SOURCE: &str = include_str!("boolean_wrappers/switch.rs");

struct TestWriter<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    out: &'cx mut Vec<AnyElement>,
}

impl<'cx, 'a, H: UiHost> UiWriter<H> for TestWriter<'cx, 'a, H> {
    fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
        f(self.cx)
    }

    fn add(&mut self, element: AnyElement) {
        self.out.push(element);
    }
}

#[test]
fn model_facade_accepts_narrow_imui_model_bridges() {
    assert!(BOOL_MODEL_RS_SOURCE.contains("pub trait IntoImUiBoolModel"));
    assert!(BOOL_MODEL_RS_SOURCE.contains("impl IntoImUiBoolModel for Model<bool>"));
    assert!(BOOL_MODEL_RS_SOURCE.contains("impl IntoImUiBoolModel for &Model<bool>"));
    assert!(BOOL_MODEL_RS_SOURCE.contains("impl IntoImUiBoolModel for &mut Model<bool>"));
    assert!(TEXT_MODEL_RS_SOURCE.contains("pub trait IntoImUiTextModel"));
    assert!(TEXT_MODEL_RS_SOURCE.contains("impl IntoImUiTextModel for &mut Model<String>"));
    assert!(FLOAT_MODEL_RS_SOURCE.contains("pub trait IntoImUiFloatModel"));
    assert!(FLOAT_MODEL_RS_SOURCE.contains("impl IntoImUiFloatModel for &mut Model<f32>"));
    assert!(OPTIONAL_TEXT_MODEL_RS_SOURCE.contains("pub trait IntoImUiOptionalTextModel"));

    for source in [
        BOOLEAN_MODEL_SURFACE_RS_SOURCE,
        CHECKBOX_WRAPPER_RS_SOURCE,
        SWITCH_WRAPPER_RS_SOURCE,
    ] {
        assert!(
            source.contains("model: impl crate::imui::IntoImUiBoolModel"),
            "IMUI boolean public facade should accept the narrow model bridge"
        );
        assert!(
            !source.contains("model: &fret_runtime::Model<bool>"),
            "IMUI boolean public facade should not require raw Model<bool> references"
        );
    }

    for (source, bridge, raw_model) in [
        (
            INPUT_TEXT_MODEL_SURFACE_RS_SOURCE,
            "model: impl crate::imui::IntoImUiTextModel",
            "model: &fret_runtime::Model<String>",
        ),
        (
            INPUT_WRAPPER_RS_SOURCE,
            "model: impl crate::imui::IntoImUiTextModel",
            "model: &fret_runtime::Model<String>",
        ),
        (
            VALUE_COMBO_MODEL_SURFACE_RS_SOURCE,
            "model: impl crate::imui::IntoImUiFloatModel",
            "model: &fret_runtime::Model<f32>",
        ),
        (
            SLIDER_WRAPPER_RS_SOURCE,
            "model: impl crate::imui::IntoImUiFloatModel",
            "model: &fret_runtime::Model<f32>",
        ),
        (
            VALUE_COMBO_MODEL_SURFACE_RS_SOURCE,
            "model: impl crate::imui::IntoImUiOptionalTextModel",
            "model: &fret_runtime::Model<Option<Arc<str>>>",
        ),
        (
            COMBO_WRAPPER_RS_SOURCE,
            "model: impl crate::imui::IntoImUiOptionalTextModel",
            "model: &fret_runtime::Model<Option<Arc<str>>>",
        ),
    ] {
        assert!(
            source.contains(bridge),
            "IMUI public facade should accept the narrow model bridge `{bridge}`"
        );
        assert!(
            !source.contains(raw_model),
            "IMUI public facade should not require raw model reference `{raw_model}`"
        );
    }
}

mod text;
mod wrapped;
