use super::*;

pub(super) fn paint_toast<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    toast: &ToastState,
    zoom: f32,
    viewport_origin_x: f32,
    viewport_origin_y: f32,
    viewport_h: f32,
) {
    let margin = 12.0 / zoom;
    let pad = 10.0 / zoom;
    let max_w = 420.0 / zoom;

    let mut text_style = canvas.style.geometry.context_menu_text_style.clone();
    text_style.size = Px(text_style.size.0 / zoom);
    if let Some(lh) = text_style.line_height.as_mut() {
        lh.0 /= zoom;
    }

    let constraints = TextConstraints {
        max_width: Some(Px(max_w - 2.0 * pad)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: effective_scale_factor(cx.scale_factor, zoom),
    };

    let (blob, metrics) =
        canvas
            .paint_cache
            .text_blob(cx.services, toast.message.clone(), &text_style, constraints);

    let box_w = (metrics.size.width.0 + 2.0 * pad).clamp(120.0 / zoom, max_w);
    let box_h = metrics.size.height.0 + 2.0 * pad;

    let x = viewport_origin_x + margin;
    let y = viewport_origin_y + viewport_h - box_h - margin;
    let rect = Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(box_w), Px(box_h)));

    let border_color = match toast.severity {
        DiagnosticSeverity::Info => Color::from_srgb_hex_rgb(0x33_8c_f2),
        DiagnosticSeverity::Warning => Color::from_srgb_hex_rgb(0xf2_bf_33),
        DiagnosticSeverity::Error => Color::from_srgb_hex_rgb(0xe6_59_59),
    };

    cx.scene.push(SceneOp::Quad {
        order: DrawOrder(70),
        rect,
        background: fret_core::Paint::Solid(canvas.style.paint.context_menu_background).into(),
        border: Edges::all(Px(1.0 / zoom)),
        border_paint: fret_core::Paint::Solid(border_color).into(),
        corner_radii: Corners::all(Px(6.0 / zoom)),
    });

    let text_x = Px(rect.origin.x.0 + pad);
    let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
    cx.scene.push(SceneOp::Text {
        order: DrawOrder(71),
        origin: Point::new(text_x, text_y),
        text: blob,
        paint: canvas.style.paint.context_menu_text.into(),
        outline: None,
        shadow: None,
    });
}
