use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Axis, Point, Px, Rect};
use fret_ui::element::{CrossAlign, ElementKind, Length, MainAlign, SpacingLength};
use fret_ui::elements;

use super::{centered_row_props, control_text, fill_row_props, fill_stack_props, fill_text};

fn test_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(320.0), Px(160.0)),
    )
}

mod layout;
mod text_roles;
