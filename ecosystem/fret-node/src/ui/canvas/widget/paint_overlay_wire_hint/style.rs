use super::super::*;

fn diagnostic_hint_border_color(
    hover_port_convertible: bool,
    severity: Option<DiagnosticSeverity>,
) -> Color {
    if hover_port_convertible {
        Color::from_srgb_hex_rgb(0xf2_bf_33)
    } else {
        match severity.unwrap_or(DiagnosticSeverity::Error) {
            DiagnosticSeverity::Info => Color::from_srgb_hex_rgb(0x33_8c_f2),
            DiagnosticSeverity::Warning => Color::from_srgb_hex_rgb(0xf2_bf_33),
            DiagnosticSeverity::Error => Color::from_srgb_hex_rgb(0xe6_59_59),
        }
    }
}

fn resolved_hint_border_color(
    base_border_color: Color,
    invalid_hover: bool,
    hover_port_convertible: bool,
    severity: Option<DiagnosticSeverity>,
) -> Color {
    if invalid_hover {
        diagnostic_hint_border_color(hover_port_convertible, severity)
    } else {
        base_border_color
    }
}

pub(in super::super) fn hint_border_color<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    invalid_hover: bool,
) -> Color {
    resolved_hint_border_color(
        canvas.style.paint.context_menu_border,
        invalid_hover,
        canvas.interaction.hover_port_convertible,
        canvas
            .interaction
            .hover_port_diagnostic
            .as_ref()
            .map(|(severity, _)| *severity),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_hint_border_color_uses_context_border_for_valid_hover() {
        let base = Color::from_srgb_hex_rgb(0x12_34_56);
        assert_eq!(
            resolved_hint_border_color(base, false, false, Some(DiagnosticSeverity::Warning)),
            base
        );
    }

    #[test]
    fn diagnostic_hint_border_color_prefers_convertible_warning_color() {
        assert_eq!(
            diagnostic_hint_border_color(true, Some(DiagnosticSeverity::Error)),
            Color::from_srgb_hex_rgb(0xf2_bf_33)
        );
        assert_eq!(
            diagnostic_hint_border_color(false, Some(DiagnosticSeverity::Info)),
            Color::from_srgb_hex_rgb(0x33_8c_f2)
        );
        assert_eq!(
            diagnostic_hint_border_color(false, None),
            Color::from_srgb_hex_rgb(0xe6_59_59)
        );
    }
}
