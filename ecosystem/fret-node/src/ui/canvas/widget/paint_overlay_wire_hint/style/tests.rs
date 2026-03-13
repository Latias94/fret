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
