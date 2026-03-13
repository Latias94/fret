use fret_core::TextStyle;

use super::super::*;

pub(in super::super) fn toast_text_style<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    zoom: f32,
) -> TextStyle {
    let mut text_style = canvas.style.geometry.context_menu_text_style.clone();
    text_style.size = Px(text_style.size.0 / zoom);
    if let Some(line_height) = text_style.line_height.as_mut() {
        line_height.0 /= zoom;
    }
    text_style
}

pub(in super::super) fn toast_border_color(severity: DiagnosticSeverity) -> Color {
    match severity {
        DiagnosticSeverity::Info => Color::from_srgb_hex_rgb(0x33_8c_f2),
        DiagnosticSeverity::Warning => Color::from_srgb_hex_rgb(0xf2_bf_33),
        DiagnosticSeverity::Error => Color::from_srgb_hex_rgb(0xe6_59_59),
    }
}

#[cfg(test)]
mod tests;
