use super::*;

use fret_app::App;
use fret_authoring::UiWriter;
use fret_core::{AppWindowId, Px, Rect, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, Length};
use fret_ui::{ElementContext, UiHost, elements};

const BOOL_MODEL_RS_SOURCE: &str = include_str!("../bool_model.rs");
const BOOLEAN_MODEL_SURFACE_RS_SOURCE: &str = include_str!("model_surface/boolean.rs");
const CHECKBOX_WRAPPER_RS_SOURCE: &str = include_str!("boolean_wrappers/checkbox.rs");
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
fn boolean_model_facade_accepts_narrow_imui_model_bridge() {
    assert!(BOOL_MODEL_RS_SOURCE.contains("pub trait IntoImUiBoolModel"));
    assert!(BOOL_MODEL_RS_SOURCE.contains("impl IntoImUiBoolModel for Model<bool>"));
    assert!(BOOL_MODEL_RS_SOURCE.contains("impl IntoImUiBoolModel for &Model<bool>"));
    assert!(BOOL_MODEL_RS_SOURCE.contains("impl IntoImUiBoolModel for &mut Model<bool>"));

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
}

mod text;
mod wrapped;
