use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Point, Px, Rect, Size};
use fret_ui::element::{
    AnyElement, ContainerProps, ElementKind, LayoutStyle, Length, Overflow, VirtualListMeasureMode,
};

use super::row::{row_height_for_index, wrap_row};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    )
}

fn oversized_content(cx: &mut fret_ui::ElementContext<'_, App>) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.height = Length::Px(Px(80.0));
    cx.container(props, |_cx| Vec::new())
}

mod fixed_known;
mod measured;
