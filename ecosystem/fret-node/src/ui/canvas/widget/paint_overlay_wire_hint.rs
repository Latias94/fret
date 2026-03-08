use super::*;

pub(super) fn paint_wire_drag_hint<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    wire_drag: &WireDrag,
    zoom: f32,
) {
    let invalid_hover =
        canvas.interaction.hover_port.is_some() && !canvas.interaction.hover_port_valid;
    let Some(text) = hint_text(canvas, wire_drag, invalid_hover) else {
        return;
    };

    let mut text_style = canvas.style.geometry.context_menu_text_style.clone();
    text_style.size = Px(text_style.size.0 / zoom);
    if let Some(lh) = text_style.line_height.as_mut() {
        lh.0 /= zoom;
    }

    let pad = 8.0 / zoom;
    let max_w = 220.0 / zoom;
    let constraints = TextConstraints {
        max_width: Some(Px(max_w - 2.0 * pad)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: effective_scale_factor(cx.scale_factor, zoom),
    };

    let (blob, metrics) = canvas
        .paint_cache
        .text_blob(cx.services, text, &text_style, constraints);

    let box_w = (metrics.size.width.0 + 2.0 * pad).clamp(72.0 / zoom, max_w);
    let box_h = metrics.size.height.0 + 2.0 * pad;

    let offset_x = 14.0 / zoom;
    let offset_y = 12.0 / zoom;
    let rect = Rect::new(
        Point::new(
            Px(wire_drag.pos.x.0 + offset_x),
            Px(wire_drag.pos.y.0 + offset_y),
        ),
        Size::new(Px(box_w), Px(box_h)),
    );

    let border_color = hint_border_color(canvas, invalid_hover);

    cx.scene.push(SceneOp::Quad {
        order: DrawOrder(69),
        rect,
        background: fret_core::Paint::Solid(canvas.style.paint.context_menu_background).into(),
        border: Edges::all(Px(1.0 / zoom)),
        border_paint: fret_core::Paint::Solid(border_color).into(),
        corner_radii: Corners::all(Px(6.0 / zoom)),
    });

    let text_x = Px(rect.origin.x.0 + pad);
    let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
    cx.scene.push(SceneOp::Text {
        order: DrawOrder(70),
        origin: Point::new(text_x, text_y),
        text: blob,
        paint: canvas.style.paint.context_menu_text.into(),
        outline: None,
        shadow: None,
    });
}

fn hint_text<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    wire_drag: &WireDrag,
    invalid_hover: bool,
) -> Option<Arc<str>> {
    if invalid_hover {
        return Some(
            canvas
                .interaction
                .hover_port_diagnostic
                .as_ref()
                .map(|(_sev, msg)| msg.clone())
                .unwrap_or_else(|| Arc::<str>::from("Invalid connection")),
        );
    }

    match &wire_drag.kind {
        WireDragKind::New { bundle, .. } if bundle.len() > 1 => {
            Some(Arc::<str>::from(format!("Bundle: {}", bundle.len())))
        }
        WireDragKind::ReconnectMany { edges } if edges.len() > 1 => {
            Some(Arc::<str>::from(format!("Yank: {}", edges.len())))
        }
        _ => None,
    }
}

fn hint_border_color<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    invalid_hover: bool,
) -> Color {
    if invalid_hover {
        if canvas.interaction.hover_port_convertible {
            Color::from_srgb_hex_rgb(0xf2_bf_33)
        } else {
            match canvas
                .interaction
                .hover_port_diagnostic
                .as_ref()
                .map(|(sev, _)| *sev)
                .unwrap_or(DiagnosticSeverity::Error)
            {
                DiagnosticSeverity::Info => Color::from_srgb_hex_rgb(0x33_8c_f2),
                DiagnosticSeverity::Warning => Color::from_srgb_hex_rgb(0xf2_bf_33),
                DiagnosticSeverity::Error => Color::from_srgb_hex_rgb(0xe6_59_59),
            }
        }
    } else {
        canvas.style.paint.context_menu_border
    }
}
