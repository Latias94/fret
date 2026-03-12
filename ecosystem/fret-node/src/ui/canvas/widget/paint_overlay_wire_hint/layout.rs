use super::WireHintPaintLayout;
use super::*;

pub(super) fn resolve_wire_hint_layout<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    scale_factor: f32,
    zoom: f32,
) -> WireHintPaintLayout {
    let mut text_style = canvas.style.geometry.context_menu_text_style.clone();
    text_style.size = Px(text_style.size.0 / zoom);
    if let Some(line_height) = text_style.line_height.as_mut() {
        line_height.0 /= zoom;
    }

    let pad = 8.0 / zoom;
    let max_w = 220.0 / zoom;
    let constraints = TextConstraints {
        max_width: Some(Px(max_w - 2.0 * pad)),
        wrap: TextWrap::Word,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        scale_factor: effective_scale_factor(scale_factor, zoom),
    };

    WireHintPaintLayout {
        pad,
        max_w,
        offset_x: 14.0 / zoom,
        offset_y: 12.0 / zoom,
        text_style,
        constraints,
    }
}
