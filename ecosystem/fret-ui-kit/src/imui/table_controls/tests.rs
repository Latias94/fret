use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, Length, ScrollAxis};
use fret_ui::elements;
use fret_ui::scroll::ScrollHandle;

use super::{BuiltTableCell, BuiltTableRow, header, render};
use crate::imui::{TableColumn, TableOptions, TableSortDirection};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    )
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

fn count_x_scrolls(root: &AnyElement) -> usize {
    let here = match &root.kind {
        ElementKind::Scroll(props) if props.axis == ScrollAxis::X => 1,
        _ => 0,
    };
    here + root.children.iter().map(count_x_scrolls).sum::<usize>()
}

mod header_text;
mod rendering;
