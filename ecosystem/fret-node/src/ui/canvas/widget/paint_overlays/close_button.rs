use crate::ui::canvas::widget::*;

pub(super) fn paint_close_button<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    zoom: f32,
) {
    if canvas.close_command.is_none() {
        return;
    }

    let rect = NodeGraphCanvasWith::<M>::close_button_rect(snapshot.pan, zoom);
    let hovered = canvas
        .interaction
        .last_pos
        .is_some_and(|position| NodeGraphCanvasWith::<M>::rect_contains(rect, position));

    let background = if hovered {
        canvas.style.paint.context_menu_hover_background
    } else {
        canvas.style.paint.context_menu_background
    };

    cx.scene.push(SceneOp::Quad {
        order: DrawOrder(60),
        rect,
        background: fret_core::Paint::Solid(background).into(),
        border: Edges::all(Px(1.0 / zoom)),
        border_paint: fret_core::Paint::Solid(canvas.style.paint.context_menu_border).into(),
        corner_radii: Corners::all(Px(6.0 / zoom)),
    });

    let mut text_style = canvas.style.geometry.context_menu_text_style.clone();
    text_style.size = Px(text_style.size.0 / zoom);
    if let Some(line_height) = text_style.line_height.as_mut() {
        line_height.0 /= zoom;
    }
    let pad = 10.0 / zoom;
    let constraints = TextConstraints {
        max_width: Some(Px((rect.size.width.0 - 2.0 * pad).max(0.0))),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: effective_scale_factor(cx.scale_factor, zoom),
    };
    let (blob, metrics) =
        canvas
            .paint_cache
            .text_blob(cx.services, "Close", &text_style, constraints);

    let text_x = Px(rect.origin.x.0 + pad);
    let inner_y = rect.origin.y.0 + (rect.size.height.0 - metrics.size.height.0) * 0.5;
    let text_y = Px(inner_y + metrics.baseline.0);
    cx.scene.push(SceneOp::Text {
        order: DrawOrder(61),
        origin: Point::new(text_x, text_y),
        text: blob,
        paint: canvas.style.paint.context_menu_text.into(),
        outline: None,
        shadow: None,
    });
}
