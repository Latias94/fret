use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Color, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, Length, PressableState};
use fret_ui::elements;
use fret_ui::{Theme, ThemeConfig};

use super::visual::{resolve_selectable_palette, selectable_row_element};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
}

fn first_text(root: &AnyElement) -> Option<&AnyElement> {
    match &root.kind {
        ElementKind::Text(_) => Some(root),
        _ => root.children.iter().find_map(first_text),
    }
}

mod palette;
mod row_text;
