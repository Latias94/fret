use std::sync::Arc;

use fret_app::App;
use fret_authoring::UiWriter;
use fret_core::{AppWindowId, Px, Rect, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, Length};
use fret_ui::{ElementContext, UiHost, elements};

use super::text::{tooltip_body_text, tooltip_text_with_options};
use crate::imui::{ResponseExt, TooltipOptions};

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

mod mount;
mod options;
mod text_role;
