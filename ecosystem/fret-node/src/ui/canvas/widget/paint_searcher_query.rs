use super::*;

pub(super) fn paint_searcher_query<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    searcher: &SearcherState,
    text_style: &fret_core::TextStyle,
    constraints: TextConstraints,
    inner_x: f32,
    inner_y: f32,
    inner_w: f32,
    item_h: f32,
    pad: f32,
    zoom: f32,
) -> f32 {
    let query_rect = Rect::new(
        Point::new(Px(inner_x), Px(inner_y)),
        Size::new(Px(inner_w), Px(item_h)),
    );
    cx.scene.push(SceneOp::Quad {
        order: DrawOrder(56),
        rect: query_rect,
        background: fret_core::Paint::Solid(self::query_background_color(canvas)).into(),
        border: Edges::all(Px(0.0)),
        border_paint: fret_core::Paint::TRANSPARENT.into(),
        corner_radii: Corners::all(Px(4.0 / zoom)),
    });

    let query_text = query_text(searcher);
    let (blob, metrics) =
        canvas
            .paint_cache
            .text_blob(cx.services, query_text, text_style, constraints);
    let text_y = Px(query_rect.origin.y.0
        + (query_rect.size.height.0 - metrics.size.height.0) * 0.5
        + metrics.baseline.0);
    let query_color = if searcher.query.is_empty() {
        canvas.style.paint.context_menu_text_disabled
    } else {
        canvas.style.paint.context_menu_text
    };
    cx.scene.push(SceneOp::Text {
        order: DrawOrder(57),
        origin: Point::new(query_rect.origin.x, text_y),
        text: blob,
        paint: (query_color).into(),
        outline: None,
        shadow: None,
    });

    inner_y + item_h + pad
}

fn query_text(searcher: &SearcherState) -> Arc<str> {
    if searcher.query.is_empty() {
        Arc::<str>::from("Search...")
    } else {
        Arc::<str>::from(format!("Search: {}", searcher.query))
    }
}

fn query_background_color<M: NodeGraphCanvasMiddleware>(canvas: &NodeGraphCanvasWith<M>) -> Color {
    canvas.style.paint.context_menu_hover_background
}
