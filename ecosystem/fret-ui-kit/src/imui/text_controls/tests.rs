use super::*;

use std::sync::Arc;

use fret_app::App;
use fret_authoring::UiWriter;
use fret_core::{AppWindowId, Corners, Edges, Px, Rect};
use fret_ui::element::{AnyElement, ElementKind, Length};
use fret_ui::{ElementContext, UiHost};

use crate::imui::{InputTextOptions, TextAreaOptions};

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

fn first_text_input(root: &AnyElement) -> Option<&fret_ui::element::TextInputProps> {
    match &root.kind {
        ElementKind::TextInput(props) => Some(props),
        _ => root.children.iter().find_map(first_text_input),
    }
}

fn first_text_area(root: &AnyElement) -> Option<&fret_ui::element::TextAreaProps> {
    match &root.kind {
        ElementKind::TextArea(props) => Some(props),
        _ => root.children.iter().find_map(first_text_area),
    }
}

mod input_chrome;
mod textarea_chrome;
