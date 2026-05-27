//! Shared Material field chrome helpers.

use std::sync::Arc;

use fret_core::{Color, Corners, DrawOrder, Edges, Point, Px, Rect, Size};
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, CanvasProps, PositionStyle};
use fret_ui::elements::ElementContext;

pub(crate) fn material_field_active_indicator_layer<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    height: Px,
    color: Color,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let mut props = CanvasProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.inset.top = Some(Px(0.0)).into();
    props.layout.inset.right = Some(Px(0.0)).into();
    props.layout.inset.bottom = Some(Px(0.0)).into();
    props.layout.inset.left = Some(Px(0.0)).into();

    let mut indicator = cx.canvas(props, move |p| {
        if height.0 <= 0.0 || color.a <= 0.0 {
            return;
        }

        let bounds = p.bounds();
        let y = Px(bounds.origin.y.0 + bounds.size.height.0 - height.0);
        let rect = Rect::new(
            Point::new(bounds.origin.x, y),
            Size::new(bounds.size.width, height),
        );
        p.scene().push(fret_core::SceneOp::Quad {
            order: DrawOrder(0),
            rect,
            background: fret_core::Paint::Solid(color).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });
    });

    if let Some(test_id) = test_id {
        indicator = indicator.test_id(test_id);
    }

    indicator
}
