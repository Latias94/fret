#[path = "paint_overlay_wire_hint/draw.rs"]
mod draw;
#[path = "paint_overlay_wire_hint/layout.rs"]
mod layout;
mod message;
mod style;

use super::*;

struct WireHintPaintLayout {
    pad: f32,
    max_w: f32,
    offset_x: f32,
    offset_y: f32,
    text_style: fret_core::TextStyle,
    constraints: TextConstraints,
}

pub(super) fn paint_wire_drag_hint<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    wire_drag: &WireDrag,
    zoom: f32,
) {
    let invalid_hover =
        canvas.interaction.hover_port.is_some() && !canvas.interaction.hover_port_valid;
    let Some(text) = message::hint_text(canvas, wire_drag, invalid_hover) else {
        return;
    };
    let layout = layout::resolve_wire_hint_layout(canvas, cx.scale_factor, zoom);
    let border_color = style::hint_border_color(canvas, invalid_hover);
    draw::paint_wire_drag_hint_content(canvas, cx, wire_drag, zoom, text, border_color, &layout);
}
