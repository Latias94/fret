use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::token_resolver::blend_over;
use crate::select::SelectVariant;
use crate::text_field::TextFieldVariant;
use crate::tokens::{autocomplete, select, text_field};

use super::super::super::visual_fixture_model::Case;
use super::super::assertions::*;
use super::super::token_lookup::*;
use super::super::typography_helpers::control_text_style;
pub(in super::super) fn run_text_field_case(case: &Case, theme: &Theme) {
    let variant = text_field_variant(&case.input.variant, &case.id);
    let hovered = case.input.hovered;
    let focused = case.input.focused;
    let disabled = case.input.disabled;
    let error = case.input.error;
    let style = text_field::text_input_style(theme, variant, focused, hovered, disabled, error);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_text_field_color(
                    theme,
                    variant,
                    hovered,
                    focused,
                    disabled,
                    error,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_text_field_color(
                    theme,
                    variant,
                    hovered,
                    focused,
                    disabled,
                    error,
                    &assertion.role,
                ),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "blend_over" => assert_color_close(
                &case.id,
                &assertion.role,
                style.background,
                blend_over(
                    token_color(theme, require_token(assertion, "base_color_token")),
                    token_color(theme, require_token(assertion, "overlay_color_token")),
                    token_number(theme, require_token(assertion, "opacity_token")),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_text_field_metric(theme, variant, &style, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "metric_literal" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_text_field_metric(theme, variant, &style, &assertion.role),
                Px(require_value(assertion)),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_text_field_number(theme, variant, error, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners" => assert_corners_eq(
                &case.id,
                &assertion.role,
                style.corner_radii,
                token_corners(theme, require_token(assertion, "token")),
            ),
            "text_style_alias" => assert_text_style_alias(theme, case, assertion),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(in super::super) fn run_select_case(case: &Case, theme: &Theme) {
    let variant = select_variant(&case.input.variant, &case.id);
    let hovered = case.input.hovered;
    let focused = case.input.focused;
    let disabled = case.input.disabled;
    let error = case.input.error;
    let item_enabled = case.input.enabled.unwrap_or(true);
    let selected = case.input.selected;

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_select_color(
                    theme,
                    variant,
                    hovered,
                    focused,
                    disabled,
                    error,
                    item_enabled,
                    selected,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_select_color(
                    theme,
                    variant,
                    hovered,
                    focused,
                    disabled,
                    error,
                    item_enabled,
                    selected,
                    &assertion.role,
                ),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "blend_over" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_select_color(
                    theme,
                    variant,
                    hovered,
                    focused,
                    disabled,
                    error,
                    item_enabled,
                    selected,
                    &assertion.role,
                ),
                blend_over(
                    token_color(theme, require_token(assertion, "base_color_token")),
                    token_color(theme, require_token(assertion, "overlay_color_token")),
                    token_number(theme, require_token(assertion, "opacity_token")),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_select_metric(
                    theme,
                    variant,
                    hovered,
                    focused,
                    disabled,
                    error,
                    &assertion.role,
                ),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_select_number(
                    theme,
                    variant,
                    hovered,
                    focused,
                    disabled,
                    error,
                    &assertion.role,
                ),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_select_corners(theme, variant, selected, &assertion.role),
                token_corners(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_select_corners(theme, variant, selected, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_select_text_style(theme, variant, &assertion.role),
                control_text_style(theme, require_token(assertion, "source_token")),
            ),
            "text_style_alias" => assert_text_style_alias(theme, case, assertion),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(in super::super) fn run_autocomplete_case(case: &Case, theme: &Theme) {
    let variant = text_field_variant(&case.input.variant, &case.id);
    let hovered = case.input.hovered;
    let focused = case.input.focused;
    let disabled = case.input.disabled;
    let error = case.input.error;
    let item_enabled = case.input.enabled.unwrap_or(true);
    let selected = case.input.selected;
    let style = autocomplete::text_input_style(theme, variant, focused, hovered, disabled, error);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_autocomplete_color(
                    theme,
                    variant,
                    hovered,
                    focused,
                    disabled,
                    error,
                    item_enabled,
                    selected,
                    &style,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_autocomplete_color(
                    theme,
                    variant,
                    hovered,
                    focused,
                    disabled,
                    error,
                    item_enabled,
                    selected,
                    &style,
                    &assertion.role,
                ),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "blend_over" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_autocomplete_color(
                    theme,
                    variant,
                    hovered,
                    focused,
                    disabled,
                    error,
                    item_enabled,
                    selected,
                    &style,
                    &assertion.role,
                ),
                blend_over(
                    token_color(theme, require_token(assertion, "base_color_token")),
                    token_color(theme, require_token(assertion, "overlay_color_token")),
                    token_number(theme, require_token(assertion, "opacity_token")),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_autocomplete_metric(theme, variant, &style, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_autocomplete_number(theme, variant, error, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_autocomplete_corners(theme, variant, selected, &style, &assertion.role),
                token_corners(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_autocomplete_corners(theme, variant, selected, &style, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_autocomplete_text_style(theme, variant, &assertion.role),
                control_text_style(theme, require_token(assertion, "source_token")),
            ),
            "text_style_alias" => assert_text_style_alias(theme, case, assertion),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

fn actual_text_field_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    focused: bool,
    disabled: bool,
    error: bool,
    role: &str,
) -> Color {
    let style = text_field::text_input_style(theme, variant, focused, hovered, disabled, error);
    match role {
        "text_input_style.background" => style.background,
        "text_input_style.border_color" => style.border_color,
        "text_input_style.text_color" => style.text_color,
        "text_input_style.caret_color" => style.caret_color,
        "label_color" => text_field::label_color(theme, variant, hovered, disabled, error, focused),
        "supporting_text_color" => {
            text_field::supporting_text_color(theme, variant, hovered, disabled, error, focused)
        }
        "hover_state_layer_color" => {
            text_field::hover_state_layer(theme, variant, error)
                .expect("expected filled text field hover state layer")
                .0
        }
        other => panic!("unsupported text field color role {other}"),
    }
}

fn actual_text_field_metric(
    theme: &Theme,
    variant: TextFieldVariant,
    style: &fret_ui::TextInputStyle,
    role: &str,
) -> Px {
    match role {
        "container_height" => text_field::container_height(theme, variant),
        "text_input_style.border_all" => {
            assert_eq!(style.border.top, style.border.right);
            assert_eq!(style.border.top, style.border.bottom);
            assert_eq!(style.border.top, style.border.left);
            style.border.top
        }
        "text_input_style.border_bottom" => style.border.bottom,
        other => panic!("unsupported text field metric role {other}"),
    }
}

fn actual_text_field_number(
    theme: &Theme,
    variant: TextFieldVariant,
    error: bool,
    role: &str,
) -> f32 {
    match role {
        "hover_state_layer_opacity" => {
            text_field::hover_state_layer(theme, variant, error)
                .expect("expected filled text field hover state layer")
                .1
        }
        other => panic!("unsupported text field number role {other}"),
    }
}

fn actual_select_color(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    focused: bool,
    disabled: bool,
    error: bool,
    item_enabled: bool,
    selected: bool,
    role: &str,
) -> Color {
    match role {
        "container_color" => select::container_background(theme, variant, disabled),
        "input_text_color" => {
            let (color, opacity) =
                select::input_text_color(theme, variant, hovered, disabled, error, focused);
            alpha_color(color, opacity)
        }
        "leading_icon_color" => {
            let (color, opacity) =
                select::leading_icon_color(theme, variant, hovered, disabled, error, focused);
            alpha_color(color, opacity)
        }
        "trailing_icon_color" => {
            let (color, opacity) =
                select::trailing_icon_color(theme, variant, hovered, disabled, error, focused);
            alpha_color(color, opacity)
        }
        "placeholder_color" => select::placeholder_color(theme, variant, disabled, error),
        "label_color" => select::label_color(theme, variant, hovered, disabled, error, focused),
        "supporting_text_color" => {
            select::supporting_text_color(theme, variant, hovered, disabled, error, focused)
        }
        "outline_color" => {
            let (_, color, opacity) =
                select::outline(theme, variant, hovered, disabled, error, focused)
                    .expect("expected outlined select outline");
            alpha_color(color, opacity)
        }
        "active_indicator_color" => {
            let (_, color, opacity) =
                select::active_indicator(theme, variant, hovered, disabled, error, focused)
                    .expect("expected filled select active indicator");
            alpha_color(color, opacity)
        }
        "hover_state_layer_color" => select::hover_state_layer(theme, variant, error).0,
        "menu_container_background" => select::menu_container_background(theme, variant),
        "menu_container_shadow_color" => select::menu_container_shadow_color(theme, variant),
        "menu_list_item_label_text_color" => {
            select::menu_list_item_label_text_color(theme, variant, item_enabled, selected)
        }
        "menu_list_item_leading_icon_color" => {
            select::menu_list_item_leading_icon_color(theme, variant, item_enabled, selected)
        }
        "menu_list_item_trailing_icon_color" => {
            select::menu_list_item_trailing_icon_color(theme, variant, item_enabled, selected)
        }
        "menu_list_item_selected_container_color" => {
            select::menu_list_item_selected_container_color(theme, variant)
        }
        other => panic!("unsupported select color role {other}"),
    }
}

fn actual_select_metric(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    focused: bool,
    disabled: bool,
    error: bool,
    role: &str,
) -> Px {
    match role {
        "container_height" => select::container_height(theme, variant),
        "outline_width" => {
            select::outline(theme, variant, hovered, disabled, error, focused)
                .expect("expected outlined select outline")
                .0
        }
        "active_indicator_height" => {
            select::active_indicator(theme, variant, hovered, disabled, error, focused)
                .expect("expected filled select active indicator")
                .0
        }
        "leading_icon_size" => select::leading_icon_size(theme, variant),
        "trailing_icon_size" => select::trailing_icon_size(theme, variant),
        "menu_container_elevation" => select::menu_container_elevation(theme, variant),
        "menu_list_item_height" => select::menu_list_item_height(theme, variant),
        "menu_list_item_leading_icon_size" => {
            select::menu_list_item_leading_icon_size(theme, variant)
        }
        "menu_list_item_trailing_icon_size" => {
            select::menu_list_item_trailing_icon_size(theme, variant)
        }
        "menu_selectable_item_outer_horizontal_padding" => {
            select::menu_selectable_item_outer_horizontal_padding(theme, variant)
        }
        "menu_selectable_item_outer_vertical_padding" => {
            select::menu_selectable_item_outer_vertical_padding(theme, variant, false)
        }
        "menu_selectable_item_with_secondary_outer_vertical_padding" => {
            select::menu_selectable_item_outer_vertical_padding(theme, variant, true)
        }
        "menu_list_item_content_horizontal_padding" => {
            select::menu_list_item_content_horizontal_padding(theme, variant)
        }
        "menu_list_item_icon_text_gap" => select::menu_list_item_icon_text_gap(theme, variant),
        other => panic!("unsupported select metric role {other}"),
    }
}

fn actual_select_number(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    focused: bool,
    disabled: bool,
    error: bool,
    role: &str,
) -> f32 {
    match role {
        "hover_state_layer_opacity" => select::hover_state_layer(theme, variant, error).1,
        "outline_opacity" => {
            select::outline(theme, variant, hovered, disabled, error, focused)
                .expect("expected outlined select outline")
                .2
        }
        "active_indicator_opacity" => {
            select::active_indicator(theme, variant, hovered, disabled, error, focused)
                .expect("expected filled select active indicator")
                .2
        }
        other => panic!("unsupported select number role {other}"),
    }
}

fn actual_select_corners(
    theme: &Theme,
    variant: SelectVariant,
    selected: bool,
    role: &str,
) -> Corners {
    match role {
        "container_corner" => select::container_corner(theme, variant),
        "menu_container_shape" => select::menu_container_shape(theme, variant),
        "menu_list_item_container_shape" => {
            select::menu_list_item_container_shape(theme, variant, selected)
        }
        other => panic!("unsupported select corners role {other}"),
    }
}

fn actual_select_text_style(theme: &Theme, variant: SelectVariant, role: &str) -> TextStyle {
    match role {
        "input_text_style" => {
            select::input_text_style(theme, variant).expect("expected select input text style")
        }
        "menu_list_item_label_text_style" => {
            select::menu_list_item_label_text_style(theme, variant)
                .expect("expected select menu list item label text style")
        }
        other => panic!("unsupported select text style role {other}"),
    }
}

fn actual_autocomplete_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    focused: bool,
    disabled: bool,
    error: bool,
    item_enabled: bool,
    selected: bool,
    style: &fret_ui::TextInputStyle,
    role: &str,
) -> Color {
    match role {
        "text_input_style.background" => style.background,
        "text_input_style.border_color" => style.border_color,
        "text_input_style.text_color" => style.text_color,
        "text_input_style.caret_color" => style.caret_color,
        "label_color" => {
            autocomplete::label_color(theme, variant, hovered, disabled, error, focused)
        }
        "supporting_text_color" => {
            autocomplete::supporting_text_color(theme, variant, hovered, disabled, error, focused)
        }
        "leading_icon_color" => {
            let (color, opacity) =
                autocomplete::leading_icon_color(theme, variant, hovered, disabled, error, focused);
            alpha_color(color, opacity)
        }
        "trailing_icon_color" => {
            let (color, opacity) = autocomplete::trailing_icon_color(
                theme, variant, hovered, disabled, error, focused,
            );
            alpha_color(color, opacity)
        }
        "hover_state_layer_color" => {
            autocomplete::hover_state_layer(theme, variant, error)
                .expect("expected autocomplete hover state layer")
                .0
        }
        "menu_container_background" => autocomplete::menu_container_background(theme, variant),
        "menu_container_shadow_color" => autocomplete::menu_container_shadow_color(theme, variant),
        "menu_list_item_label_text_color" => {
            autocomplete::menu_list_item_label_text_color(theme, variant, item_enabled, selected)
        }
        "menu_list_item_selected_container_color" => {
            autocomplete::menu_list_item_selected_container_color(theme, variant)
        }
        other => panic!("unsupported autocomplete color role {other}"),
    }
}

fn actual_autocomplete_metric(
    theme: &Theme,
    variant: TextFieldVariant,
    style: &fret_ui::TextInputStyle,
    role: &str,
) -> Px {
    match role {
        "container_height" => autocomplete::text_field_container_height(theme, variant),
        "text_input_style.border_all" => {
            assert_eq!(style.border.top, style.border.right);
            assert_eq!(style.border.top, style.border.bottom);
            assert_eq!(style.border.top, style.border.left);
            style.border.top
        }
        "text_input_style.border_bottom" => style.border.bottom,
        "leading_icon_size" => autocomplete::leading_icon_size(theme, variant),
        "trailing_icon_size" => autocomplete::trailing_icon_size(theme, variant),
        "menu_container_elevation" => autocomplete::menu_container_elevation(theme, variant),
        "menu_list_item_height" => autocomplete::menu_list_item_height(theme, variant),
        "menu_selectable_item_outer_horizontal_padding" => {
            autocomplete::menu_selectable_item_outer_horizontal_padding(theme, variant)
        }
        "menu_selectable_item_outer_vertical_padding" => {
            autocomplete::menu_selectable_item_outer_vertical_padding(theme, variant, false)
        }
        "menu_selectable_item_with_secondary_outer_vertical_padding" => {
            autocomplete::menu_selectable_item_outer_vertical_padding(theme, variant, true)
        }
        "menu_list_item_content_horizontal_padding" => {
            autocomplete::menu_list_item_content_horizontal_padding(theme, variant)
        }
        other => panic!("unsupported autocomplete metric role {other}"),
    }
}

fn actual_autocomplete_number(
    theme: &Theme,
    variant: TextFieldVariant,
    error: bool,
    role: &str,
) -> f32 {
    match role {
        "hover_state_layer_opacity" => {
            autocomplete::hover_state_layer(theme, variant, error)
                .expect("expected autocomplete hover state layer")
                .1
        }
        other => panic!("unsupported autocomplete number role {other}"),
    }
}

fn actual_autocomplete_corners(
    theme: &Theme,
    variant: TextFieldVariant,
    selected: bool,
    style: &fret_ui::TextInputStyle,
    role: &str,
) -> Corners {
    match role {
        "text_input_style.corner_radii" => style.corner_radii,
        "menu_container_shape" => autocomplete::menu_container_shape(theme, variant),
        "menu_list_item_container_shape" => {
            autocomplete::menu_list_item_container_shape(theme, variant, selected)
        }
        other => panic!("unsupported autocomplete corners role {other}"),
    }
}

fn actual_autocomplete_text_style(
    theme: &Theme,
    variant: TextFieldVariant,
    role: &str,
) -> TextStyle {
    match role {
        "menu_list_item_label_text_style" => {
            autocomplete::menu_list_item_label_text_style(theme, variant)
                .expect("expected autocomplete menu list item label text style")
        }
        other => panic!("unsupported autocomplete text style role {other}"),
    }
}

fn text_field_variant(value: &str, case_id: &str) -> TextFieldVariant {
    match value {
        "outlined" => TextFieldVariant::Outlined,
        "filled" => TextFieldVariant::Filled,
        other => panic!("{case_id}: unsupported text field variant {other}"),
    }
}

fn select_variant(value: &str, case_id: &str) -> SelectVariant {
    match value {
        "outlined" => SelectVariant::Outlined,
        "filled" => SelectVariant::Filled,
        other => panic!("{case_id}: unsupported select variant {other}"),
    }
}
