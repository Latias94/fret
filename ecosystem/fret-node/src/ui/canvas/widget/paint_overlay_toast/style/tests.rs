use super::*;

#[test]
fn toast_border_color_matches_diagnostic_severity() {
    assert_eq!(
        toast_border_color(DiagnosticSeverity::Info),
        Color::from_srgb_hex_rgb(0x33_8c_f2)
    );
    assert_eq!(
        toast_border_color(DiagnosticSeverity::Warning),
        Color::from_srgb_hex_rgb(0xf2_bf_33)
    );
    assert_eq!(
        toast_border_color(DiagnosticSeverity::Error),
        Color::from_srgb_hex_rgb(0xe6_59_59)
    );
}
