use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::Theme;
use fret_ui::element::{AnyElement, ElementKind, Length};
use fret_ui::elements;

use super::bullet_text_element;
use crate::imui::BulletTextOptions;

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

mod text_role;
