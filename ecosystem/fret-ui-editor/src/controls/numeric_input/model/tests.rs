use super::NumericInputOptions;
use super::editor_numeric_input_text_style;
use crate::primitives::EditorDensity;
use fret_core::{Px, TextStyle};
use fret_ui_kit::Size;

#[test]
fn numeric_input_defaults_to_small_editor_control_size() {
    assert_eq!(NumericInputOptions::default().size, Size::Small);
}

#[test]
fn numeric_input_text_style_uses_density_row_height_for_edit_line_box() {
    let style = editor_numeric_input_text_style(
        TextStyle {
            size: Px(12.0),
            line_height: Some(Px(16.0)),
            ..Default::default()
        },
        EditorDensity {
            row_height: Px(24.0),
            ..Default::default()
        },
    );

    assert_eq!(style.line_height, Some(Px(24.0)));
}
