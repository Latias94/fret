use super::*;

pub(super) fn paint_searcher_rows<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    searcher: &SearcherState,
    text_style: &fret_core::TextStyle,
    constraints: TextConstraints,
    inner_x: f32,
    list_y0: f32,
    inner_w: f32,
    item_h: f32,
    zoom: f32,
) {
    let start = searcher.scroll.min(searcher.rows.len());
    let end = (start + searcher_visible_rows(searcher)).min(searcher.rows.len());
    for (slot, row_ix) in (start..end).enumerate() {
        let row = &searcher.rows[row_ix];
        let item_rect = Rect::new(
            Point::new(Px(inner_x), Px(list_y0 + slot as f32 * item_h)),
            Size::new(Px(inner_w), Px(item_h)),
        );

        if should_highlight_row(searcher, row, row_ix) {
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(56),
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
                .text_blob(cx.services, row.label.clone(), text_style, constraints);
        let text_y = Px(item_rect.origin.y.0
            + (item_rect.size.height.0 - metrics.size.height.0) * 0.5
            + metrics.baseline.0);
        let color = if row.enabled {
            canvas.style.paint.context_menu_text
        } else {
            canvas.style.paint.context_menu_text_disabled
        };

        cx.scene.push(SceneOp::Text {
            order: DrawOrder(57),
            origin: Point::new(item_rect.origin.x, text_y),
            text: blob,
            paint: (color).into(),
            outline: None,
            shadow: None,
        });
    }
}

fn should_highlight_row(searcher: &SearcherState, row: &SearcherRow, row_ix: usize) -> bool {
    let is_active = searcher.active_row == row_ix;
    let is_hovered = searcher.hovered_row == Some(row_ix);
    (is_hovered || is_active) && super::searcher_rows::searcher_is_selectable_row(row)
}
