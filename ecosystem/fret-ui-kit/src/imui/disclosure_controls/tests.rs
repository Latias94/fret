use super::*;

use std::sync::Arc;

use super::super::{CollapsingHeaderOptions, TreeNodeOptions, UiWriterImUiFacadeExt};
use fret_app::App;
use fret_authoring::UiWriter;
use fret_core::{AppWindowId, Color, Point, Px, Rect, SemanticsRole, Size, TextOverflow, TextWrap};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, ElementKind, Length, PressableProps, PressableState};
use fret_ui::elements;
use fret_ui::{ElementContext, Theme, ThemeConfig};

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

fn contains_text(root: &AnyElement, expected: &str) -> bool {
    match &root.kind {
        ElementKind::Text(props) if props.text.as_ref() == expected => true,
        _ => root
            .children
            .iter()
            .any(|child| contains_text(child, expected)),
    }
}

fn first_pressable(root: &AnyElement) -> Option<&PressableProps> {
    match &root.kind {
        ElementKind::Pressable(props) => Some(props),
        _ => root.children.iter().find_map(first_pressable),
    }
}

fn first_text<'a>(root: &'a AnyElement, expected: &str) -> Option<&'a AnyElement> {
    match &root.kind {
        ElementKind::Text(props) if props.text.as_ref() == expected => Some(root),
        _ => root
            .children
            .iter()
            .find_map(|child| first_text(child, expected)),
    }
}

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

mod entry;
mod tree;
mod visual;
