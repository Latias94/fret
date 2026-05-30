use super::axis_drag_value_input_text_style;
use fret_core::{Px, TextStyle};

#[test]
fn axis_drag_value_input_text_style_uses_density_row_height_for_typing_line_box() {
    let style = axis_drag_value_input_text_style(
        TextStyle {
            size: Px(12.0),
            line_height: Some(Px(16.0)),
            ..Default::default()
        },
        Px(24.0),
    );

    assert_eq!(style.line_height, Some(Px(24.0)));
}
