use super::ContextMenuPaintLayout;
use super::*;

pub(super) fn paint_context_menu_items<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    menu: &ContextMenuState,
    zoom: f32,
    layout: &ContextMenuPaintLayout,
) {
    let constraints = TextConstraints {
        max_width: Some(layout.inner_width),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: effective_scale_factor(cx.scale_factor, zoom),
    };

    for (ix, item) in menu.items.iter().enumerate() {
        let item_rect = Rect::new(
            Point::new(
                layout.inner_origin.x,
                Px(layout.inner_origin.y.0 + ix as f32 * layout.item_height.0),
            ),
            Size::new(layout.inner_width, layout.item_height),
        );

        let is_active = menu.active_item == ix;
        let is_hovered = menu.hovered_item == Some(ix);
        if (is_hovered || is_active) && item.enabled {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(51),
                rect: item_rect,
                background: fret_core::Paint::Solid(
                    canvas.style.paint.context_menu_hover_background,
                )
                .into(),
                border: Edges::all(Px(0.0)),
                border_paint: fret_core::Paint::TRANSPARENT.into(),
                corner_radii: Corners::all(layout.hover_radius),
            });
        }

        let (blob, metrics) = canvas.paint_cache.text_blob(
            cx.services,
            item.label.clone(),
            &layout.text_style,
            constraints,
        );

        let text_y = Px(item_rect.origin.y.0
            + (item_rect.size.height.0 - metrics.size.height.0) * 0.5
            + metrics.baseline.0);
        let color = if item.enabled {
            canvas.style.paint.context_menu_text
        } else {
            canvas.style.paint.context_menu_text_disabled
        };

        cx.scene.push(SceneOp::Text {
            order: DrawOrder(52),
            origin: Point::new(item_rect.origin.x, text_y),
            text: blob,
            paint: color.into(),
            outline: None,
            shadow: None,
        });
    }
}
