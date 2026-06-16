use std::collections::BTreeSet;
use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, Color, Rect};
use fret_ui::Theme;
use fret_ui::element::{AnyElement, ElementKind, Length};

use super::drag_drop::apply_color_drop_payload;
use super::model::{
    ColorNumericInputMode, HsvColor, HueWheelDragTarget, color_from_rgb_preserving_alpha,
    color_numeric_input_modes, format_hex, hsv_from_color, hsv_numeric_text,
    hsv_to_color_preserving_alpha, hsv_to_rgb, hsv_with_hue_wheel_position,
    hsv_with_sv_from_local_position, hue_from_local_y, hue_wheel_geometry,
    hue_wheel_rotated_triangle, hue_wheel_sv_cursor_position, hue_wheel_target_from_local_position,
    parse_color_numeric_input, parse_hex, rgb_numeric_text, rgb_to_hsv,
};
use super::popup::copy::{ColorEditCopyFormat, color_copy_entries};
use super::popup::picker::alpha::{alpha_from_local_x, alpha_from_local_y, alpha_percent_text};
use super::popup::preview::{
    SIDE_PREVIEW_SWATCH_HEIGHT, SIDE_PREVIEW_SWATCH_WIDTH, checkerboard_cell_color,
    opaque_preview_color, preview_color_for_alpha_visibility, restore_reference_color,
};
use super::popup::tooltip::color_tooltip_lines;
use super::*;
use crate::primitives::EditorDensity;
use crate::primitives::style::EditorStyle;

mod affordances;
mod drag_drop;
mod numeric;
mod palette;
mod picker;
mod popup_policy;

fn assert_hsv_close(actual: HsvColor, hue: f32, saturation: f32, value: f32) {
    assert!(
        (actual.hue - hue).abs() < 0.002,
        "hue mismatch: actual {:?}, expected {hue}",
        actual
    );
    assert!(
        (actual.saturation - saturation).abs() < 0.002,
        "saturation mismatch: actual {:?}, expected {saturation}",
        actual
    );
    assert!(
        (actual.value - value).abs() < 0.002,
        "value mismatch: actual {:?}, expected {value}",
        actual
    );
}

#[test]
fn color_edit_uses_stable_editor_chrome_height() {
    let mut app = App::new();
    let window = AppWindowId::default();
    let model = app
        .models_mut()
        .insert(Color::from_srgb_hex_rgb(0x33_66_99));
    let element = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        Rect::default(),
        "color-edit-height",
        |cx| ColorEdit::new(model).into_element(cx),
    );

    let theme = Theme::global(&app);
    let density = EditorDensity::resolve(theme);
    let row_height = density.row_height;
    let expected_min_height = {
        let style = EditorStyle::resolve(theme);
        style
            .frame_chrome_small()
            .control_outer_height(style.density.row_height)
    };
    assert!(
        expected_min_height.0 > row_height.0,
        "ColorEdit should reserve full editor chrome height, not only the text line"
    );

    let ElementKind::Flex(root) = &element.kind else {
        panic!("color edit root should be a flex layout");
    };
    assert_eq!(
        root.layout.size.min_height,
        Some(Length::Px(expected_min_height))
    );
    assert_eq!(element.children.len(), 1);

    let row = &element.children[0];
    let ElementKind::Flex(_) = &row.kind else {
        panic!("color edit root child should be the swatch/input row");
    };
    assert_eq!(row.children.len(), 2);

    let swatch_layout = element_layout(&row.children[0], "swatch");
    assert_eq!(swatch_layout.size.height, Length::Px(density.hit_thickness));
    assert_eq!(swatch_layout.size.width, Length::Px(density.hit_thickness));

    assert!(
        descendant_has_min_height(&row.children[1], "input", expected_min_height),
        "color edit input branch should reserve the full editor chrome height"
    );
}

fn descendant_has_min_height(element: &AnyElement, label: &str, expected: fret_core::Px) -> bool {
    let layout = element_layout(element, label);
    if layout.size.min_height == Some(Length::Px(expected)) {
        return true;
    }

    element
        .children
        .iter()
        .any(|child| descendant_has_min_height(child, label, expected))
}

fn element_layout<'a>(element: &'a AnyElement, label: &str) -> &'a fret_ui::element::LayoutStyle {
    match &element.kind {
        ElementKind::Flex(props) => &props.layout,
        ElementKind::PointerRegion(props) => &props.layout,
        ElementKind::Pressable(props) => &props.layout,
        ElementKind::TextInput(props) => &props.layout,
        other => panic!("{label} should expose layout props, got {other:?}"),
    }
}
