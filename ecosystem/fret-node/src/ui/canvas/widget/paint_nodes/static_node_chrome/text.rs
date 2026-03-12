use super::*;

fn node_text_constraints(scale_factor: f32, zoom: f32, max_w: f32) -> TextConstraints {
    TextConstraints {
        max_width: Some(Px(max_w.max(0.0))),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: effective_scale_factor(scale_factor, zoom),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_static_node_title<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    scene: &mut fret_core::Scene,
    services: &mut dyn fret_core::UiServices,
    scale_factor: f32,
    rect: Rect,
    title: &Arc<str>,
    title_text: Color,
    node_text_style: &TextStyle,
    zoom: f32,
    title_pad: f32,
    title_h: f32,
) {
    if title.is_empty() {
        return;
    }

    let constraints =
        node_text_constraints(scale_factor, zoom, rect.size.width.0 - 2.0 * title_pad);
    let (blob, metrics) =
        canvas
            .paint_cache
            .text_blob(services, title.clone(), node_text_style, constraints);

    let text_x = Px(rect.origin.x.0 + title_pad);
    let inner_y = rect.origin.y.0 + (title_h - metrics.size.height.0) * 0.5;
    let text_y = Px(inner_y + metrics.baseline.0);
    scene.push(SceneOp::Text {
        order: DrawOrder(4),
        origin: Point::new(text_x, text_y),
        text: blob,
        paint: title_text.into(),
        outline: None,
        shadow: None,
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_static_node_body<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    scene: &mut fret_core::Scene,
    services: &mut dyn fret_core::UiServices,
    scale_factor: f32,
    rect: Rect,
    body: Option<&Arc<str>>,
    pin_rows: usize,
    node_text_style: &TextStyle,
    zoom: f32,
    title_pad: f32,
) {
    let Some(body) = body else {
        return;
    };
    if body.is_empty() {
        return;
    }

    let pin_rows = pin_rows as f32;
    let body_top = rect.origin.y.0
        + (canvas.style.geometry.node_header_height
            + canvas.style.geometry.node_padding
            + pin_rows * canvas.style.geometry.pin_row_height
            + canvas.style.geometry.node_padding)
            / zoom;

    let constraints =
        node_text_constraints(scale_factor, zoom, rect.size.width.0 - 2.0 * title_pad);
    let (blob, metrics) =
        canvas
            .paint_cache
            .text_blob(services, body.clone(), node_text_style, constraints);

    let text_x = Px(rect.origin.x.0 + title_pad);
    let inner_y = body_top + metrics.baseline.0;
    scene.push(SceneOp::Text {
        order: DrawOrder(4),
        origin: Point::new(text_x, Px(inner_y)),
        text: blob,
        paint: canvas.style.paint.context_menu_text.into(),
        outline: None,
        shadow: None,
    });
}
