use super::*;

pub(super) fn paint_context_menu<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    menu: &ContextMenuState,
    zoom: f32,
) {
    let rect = context_menu_rect_at(&canvas.style, menu.origin, menu.items.len(), zoom);
    let border_w = Px(1.0 / zoom);
    let radius = Px(canvas.style.paint.context_menu_corner_radius / zoom);

    cx.scene.push(SceneOp::Quad {
        order: DrawOrder(50),
        rect,
        background: fret_core::Paint::Solid(canvas.style.paint.context_menu_background).into(),
        border: Edges::all(border_w),
        border_paint: fret_core::Paint::Solid(canvas.style.paint.context_menu_border).into(),
        corner_radii: Corners::all(radius),
    });

    let pad = canvas.style.paint.context_menu_padding / zoom;
    let item_h = canvas.style.paint.context_menu_item_height / zoom;
    let inner_x = rect.origin.x.0 + pad;
    let inner_y = rect.origin.y.0 + pad;
    let inner_w = (rect.size.width.0 - 2.0 * pad).max(0.0);

    let mut text_style = canvas.style.geometry.context_menu_text_style.clone();
    text_style.size = Px(text_style.size.0 / zoom);
    if let Some(lh) = text_style.line_height.as_mut() {
        lh.0 /= zoom;
    }

    let constraints = TextConstraints {
        max_width: Some(Px(inner_w)),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: effective_scale_factor(cx.scale_factor, zoom),
    };

    for (ix, item) in menu.items.iter().enumerate() {
        let item_rect = Rect::new(
            Point::new(Px(inner_x), Px(inner_y + ix as f32 * item_h)),
            Size::new(Px(inner_w), Px(item_h)),
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
                corner_radii: Corners::all(Px(4.0 / zoom)),
            });
        }

        let (blob, metrics) =
            canvas
                .paint_cache
                .text_blob(cx.services, item.label.clone(), &text_style, constraints);

        let text_x = item_rect.origin.x;
        let inner_y =
            item_rect.origin.y.0 + (item_rect.size.height.0 - metrics.size.height.0) * 0.5;
        let text_y = Px(inner_y + metrics.baseline.0);
        let color = if item.enabled {
            canvas.style.paint.context_menu_text
        } else {
            canvas.style.paint.context_menu_text_disabled
        };

        cx.scene.push(SceneOp::Text {
            order: DrawOrder(52),
            origin: Point::new(text_x, text_y),
            text: blob,
            paint: color.into(),
            outline: None,
            shadow: None,
        });
    }
}
