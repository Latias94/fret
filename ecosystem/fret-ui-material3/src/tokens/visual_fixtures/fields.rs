use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::blend_over;
use crate::select::SelectVariant;
use crate::text_field::TextFieldVariant;
use crate::tokens::date_picker::DatePickerTokenVariant;
use crate::tokens::{
    autocomplete, date_picker, search_bar, search_view, select, text_field, time_input, time_picker,
};

use super::super::visual_fixture_model::Case;
use super::assertions::*;
use super::input::pressable_interaction;
use super::token_lookup::*;
use super::typography_helpers::{content_text_style, control_text_style};

pub(super) fn run_text_field_case(case: &Case, theme: &Theme) {
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

pub(super) fn run_select_case(case: &Case, theme: &Theme) {
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

pub(super) fn run_autocomplete_case(case: &Case, theme: &Theme) {
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

pub(super) fn run_search_bar_case(case: &Case, theme: &Theme) {
    let hovered = case.input.hovered;
    let pressed = case.input.interaction.as_deref() == Some("pressed");

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_search_bar_color(theme, hovered, pressed, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_search_bar_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_search_bar_number(theme, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_search_bar_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_search_bar_text_style(theme, &assertion.role),
                control_text_style(theme, require_token(assertion, "source_token")),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_search_view_case(case: &Case, theme: &Theme) {
    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_search_view_color(theme, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_search_view_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_search_view_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_search_view_text_style(theme, &assertion.role),
                control_text_style(theme, require_token(assertion, "source_token")),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_date_picker_case(case: &Case, theme: &Theme) {
    let variant = date_picker_variant(&case.input.variant, &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_date_picker_color(theme, variant, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_date_picker_metric(theme, variant, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_date_picker_number(theme, variant, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_date_picker_corners(theme, variant, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_date_picker_text_style(theme, variant, &assertion.role),
                control_text_style(theme, require_token(assertion, "source_token")),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_time_picker_case(case: &Case, theme: &Theme) {
    let selected = case.input.selected;
    let focused = case.input.focused;
    let hovered = case.input.hovered;
    let error = case.input.error;
    let interaction = pressable_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_time_picker_color(
                    theme,
                    selected,
                    focused,
                    hovered,
                    error,
                    interaction,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_time_picker_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_time_picker_number(theme, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_time_picker_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_time_picker_text_style(theme, &assertion.role),
                control_text_style(theme, require_token(assertion, "source_token")),
            ),
            "text_style_content_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_time_picker_text_style(theme, &assertion.role),
                content_text_style(theme, require_token(assertion, "source_token")),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

// Field-family token outcome helpers live with the cases that exercise them.
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

fn actual_search_bar_color(theme: &Theme, hovered: bool, pressed: bool, role: &str) -> Color {
    match role {
        "container_color" => search_bar::container_color(theme),
        "leading_icon_color" => search_bar::leading_icon_color(theme),
        "trailing_icon_color" => search_bar::trailing_icon_color(theme),
        "input_text_color" => search_bar::input_text_color(theme),
        "supporting_text_color" => search_bar::supporting_text_color(theme, hovered, pressed),
        "hover_state_layer_color" => search_bar::hover_state_layer_color(theme),
        "pressed_state_layer_color" => search_bar::pressed_state_layer_color(theme),
        other => panic!("unsupported search bar color role {other}"),
    }
}

fn actual_search_bar_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "container_height" => search_bar::container_height(theme),
        "container_min_width" => search_bar::container_min_width(theme),
        "container_max_width" => search_bar::container_max_width(theme),
        "container_elevation" => search_bar::container_elevation(theme),
        other => panic!("unsupported search bar metric role {other}"),
    }
}

fn actual_search_bar_number(theme: &Theme, role: &str) -> f32 {
    match role {
        "hover_state_layer_opacity" => search_bar::hover_state_layer_opacity(theme),
        "pressed_state_layer_opacity" => search_bar::pressed_state_layer_opacity(theme),
        other => panic!("unsupported search bar number role {other}"),
    }
}

fn actual_search_bar_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => search_bar::container_shape(theme),
        other => panic!("unsupported search bar corners role {other}"),
    }
}

fn actual_search_bar_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "input_text_style" => search_bar::input_text_style(theme),
        other => panic!("unsupported search bar text style role {other}"),
    }
}

fn actual_search_view_color(theme: &Theme, role: &str) -> Color {
    match role {
        "container_color" => search_view::container_color(theme),
        "divider_color" => search_view::divider_color(theme),
        "header_leading_icon_color" => search_view::header_leading_icon_color(theme),
        "header_trailing_icon_color" => search_view::header_trailing_icon_color(theme),
        "header_input_text_color" => search_view::header_input_text_color(theme),
        "header_supporting_text_color" => search_view::header_supporting_text_color(theme),
        other => panic!("unsupported search view color role {other}"),
    }
}

fn actual_search_view_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "container_elevation" => search_view::container_elevation(theme),
        "docked_header_container_height" => search_view::docked_header_container_height(theme),
        "full_screen_header_container_height" => {
            search_view::full_screen_header_container_height(theme)
        }
        other => panic!("unsupported search view metric role {other}"),
    }
}

fn actual_search_view_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "docked_container_shape" => search_view::docked_container_shape(theme),
        "full_screen_container_shape" => search_view::full_screen_container_shape(theme),
        other => panic!("unsupported search view corners role {other}"),
    }
}

fn actual_search_view_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "header_input_text_style" => search_view::header_input_text_style(theme),
        other => panic!("unsupported search view text style role {other}"),
    }
}

fn actual_date_picker_color(theme: &Theme, variant: DatePickerTokenVariant, role: &str) -> Color {
    match role {
        "container_color" => date_picker::container_color(theme, variant),
        "weekdays_label_text_color" => date_picker::weekdays_label_text_color(theme, variant),
        "header_headline_color" => date_picker::header_headline_color(theme),
        "month_nav_title_color" => date_picker::month_nav_title_color(theme),
        "date_today_outline_color" => date_picker::date_today_outline_color(theme, variant),
        "date_unselected_label_text_color" => {
            date_picker::date_unselected_label_text_color(theme, variant)
        }
        "date_selected_container_color" => {
            date_picker::date_selected_container_color(theme, variant)
        }
        "date_selected_label_text_color" => {
            date_picker::date_selected_label_text_color(theme, variant)
        }
        other => panic!("unsupported date picker color role {other}"),
    }
}

fn actual_date_picker_metric(theme: &Theme, variant: DatePickerTokenVariant, role: &str) -> Px {
    match role {
        "container_width" => date_picker::container_width(theme, variant),
        "container_height" => date_picker::container_height(theme, variant),
        "container_elevation" => date_picker::container_elevation(theme, variant),
        "date_cell_width" => date_picker::date_cell_width(theme, variant),
        "date_cell_height" => date_picker::date_cell_height(theme, variant),
        "date_today_outline_width" => date_picker::date_today_outline_width(theme, variant),
        other => panic!("unsupported date picker metric role {other}"),
    }
}

fn actual_date_picker_number(theme: &Theme, variant: DatePickerTokenVariant, role: &str) -> f32 {
    match role {
        "date_outside_month_opacity" => date_picker::date_outside_month_opacity(theme, variant),
        other => panic!("unsupported date picker number role {other}"),
    }
}

fn actual_date_picker_corners(
    theme: &Theme,
    variant: DatePickerTokenVariant,
    role: &str,
) -> Corners {
    match role {
        "container_shape" => date_picker::container_shape(theme, variant),
        "date_cell_shape" => date_picker::date_cell_shape(theme, variant),
        other => panic!("unsupported date picker corners role {other}"),
    }
}

fn actual_date_picker_text_style(
    theme: &Theme,
    variant: DatePickerTokenVariant,
    role: &str,
) -> TextStyle {
    match role {
        "weekdays_label_text_style" => date_picker::weekdays_label_text_style(theme, variant),
        "header_headline_style" => date_picker::header_headline_style(theme),
        "month_nav_title_text_style" => date_picker::month_nav_title_text_style(theme)
            .expect("date picker month nav title style should resolve"),
        "date_label_text_style" => date_picker::date_label_text_style(theme, variant),
        other => panic!("unsupported date picker text style role {other}"),
    }
}

fn actual_time_picker_color(
    theme: &Theme,
    selected: bool,
    focused: bool,
    hovered: bool,
    error: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "container_color" => time_picker::container_color(theme),
        "headline_color" => time_picker::headline_color(theme),
        "clock_dial_background" => time_picker::clock_dial_background(theme),
        "clock_dial_label_text_color" => time_picker::clock_dial_label_text_color(theme, selected),
        "clock_dial_handle_color" => time_picker::clock_dial_handle_color(theme),
        "clock_dial_selector_center_color" => time_picker::clock_dial_selector_center_color(theme),
        "clock_dial_selector_track_color" => time_picker::clock_dial_selector_track_color(theme),
        "time_selector_container_color" => {
            time_picker::time_selector_container_color(theme, selected)
        }
        "time_selector_label_color" => {
            time_picker::time_selector_label_color(theme, selected, interaction)
        }
        "time_selector_separator_color" => time_picker::time_selector_separator_color(theme),
        "time_selector_state_layer_color" => time_picker::time_selector_state_layer_color(
            theme,
            selected,
            interaction.expect("time selector state layer requires interaction"),
        ),
        "period_selector_outline_color" => time_picker::period_selector_outline_color(theme),
        "period_selector_selected_container_color" => {
            time_picker::period_selector_selected_container_color(theme)
        }
        "period_selector_label_color" => {
            time_picker::period_selector_label_color(theme, selected, interaction)
        }
        "period_selector_state_layer_color" => time_picker::period_selector_state_layer_color(
            theme,
            selected,
            interaction.expect("period selector state layer requires interaction"),
        ),
        "time_input_field_container_color" => {
            time_input::time_input_field_container_color(theme, focused, error)
        }
        "time_input_field_focus_outline_color" => {
            time_input::time_input_field_focus_outline_color(theme, error)
        }
        "time_input_field_label_color" => {
            time_input::time_input_field_label_color(theme, focused, hovered, error)
        }
        "time_input_field_state_layer_color" => {
            time_input::time_input_field_state_layer_color(theme)
        }
        "time_input_field_separator_color" => time_input::time_input_field_separator_color(theme),
        "time_input_field_supporting_text_color" => {
            time_input::time_input_field_supporting_text_color(theme, error)
        }
        "time_input_period_selector_outline_color" => {
            time_input::period_selector_outline_color(theme)
        }
        "time_input_period_selector_selected_container_color" => {
            time_input::period_selector_selected_container_color(theme)
        }
        "time_input_period_selector_label_color" => {
            time_input::period_selector_label_color(theme, selected, interaction)
        }
        "time_input_period_selector_state_layer_color" => {
            time_input::period_selector_state_layer_color(
                theme,
                selected,
                interaction.expect("time input period selector state layer requires interaction"),
            )
        }
        other => panic!("unsupported time picker color role {other}"),
    }
}

fn actual_time_picker_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "container_elevation" => time_picker::container_elevation(theme),
        "clock_dial_size" => time_picker::clock_dial_size(theme),
        "clock_dial_handle_size" => time_picker::clock_dial_handle_size(theme),
        "clock_dial_selector_center_size" => time_picker::clock_dial_selector_center_size(theme),
        "clock_dial_selector_track_width" => time_picker::clock_dial_selector_track_width(theme),
        "time_selector_container_width" => time_picker::time_selector_container_width(theme),
        "time_selector_container_height" => time_picker::time_selector_container_height(theme),
        "display_separator_width" => time_picker::display_separator_width(theme),
        "period_selector_container_width" => time_picker::period_selector_container_width(theme),
        "period_selector_container_height" => time_picker::period_selector_container_height(theme),
        "period_selector_outline_width" => time_picker::period_selector_outline_width(theme),
        "time_input_field_container_width" => time_input::time_input_field_container_width(theme),
        "time_input_field_container_height" => time_input::time_input_field_container_height(theme),
        "time_input_field_focus_outline_width" => {
            time_input::time_input_field_focus_outline_width(theme)
        }
        "time_input_period_selector_container_width" => {
            time_input::period_selector_container_width(theme)
        }
        "time_input_period_selector_container_height" => {
            time_input::period_selector_container_height(theme)
        }
        "time_input_period_selector_outline_width" => {
            time_input::period_selector_outline_width(theme)
        }
        other => panic!("unsupported time picker metric role {other}"),
    }
}

fn actual_time_picker_number(
    theme: &Theme,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> f32 {
    match role {
        "time_selector_state_layer_opacity" => time_picker::time_selector_state_layer_opacity(
            theme,
            interaction.expect("time selector state layer opacity requires interaction"),
        ),
        "period_selector_state_layer_opacity" => time_picker::period_selector_state_layer_opacity(
            theme,
            interaction.expect("period selector state layer opacity requires interaction"),
        ),
        "time_input_field_state_layer_opacity" => {
            time_input::time_input_field_state_layer_opacity(theme)
        }
        "time_input_period_selector_state_layer_opacity" => {
            time_input::period_selector_state_layer_opacity(
                theme,
                interaction
                    .expect("time input period selector state layer opacity requires interaction"),
            )
        }
        other => panic!("unsupported time picker number role {other}"),
    }
}

fn actual_time_picker_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => time_picker::container_shape(theme),
        "clock_dial_shape" => time_picker::clock_dial_shape(theme),
        "clock_dial_handle_shape" => time_picker::clock_dial_handle_shape(theme),
        "clock_dial_selector_center_shape" => time_picker::clock_dial_selector_center_shape(theme),
        "time_selector_shape" => time_picker::time_selector_shape(theme),
        "period_selector_shape" => time_picker::period_selector_shape(theme),
        "time_input_field_container_shape" => time_input::time_input_field_container_shape(theme),
        "time_input_period_selector_shape" => time_input::period_selector_shape(theme),
        other => panic!("unsupported time picker corners role {other}"),
    }
}

fn actual_time_picker_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "headline_style" => time_picker::headline_style(theme),
        "clock_dial_label_text_style" => time_picker::clock_dial_label_text_style(theme),
        "time_selector_label_text_style" => time_picker::time_selector_label_text_style(theme),
        "time_selector_separator_style" => time_picker::time_selector_separator_style(theme),
        "period_selector_label_text_style" => time_picker::period_selector_label_text_style(theme),
        "time_input_field_label_text_style" => time_input::time_input_field_label_text_style(theme),
        "time_input_field_separator_style" => time_input::time_input_field_separator_style(theme),
        "time_input_field_supporting_text_style" => {
            time_input::time_input_field_supporting_text_style(theme)
        }
        "time_input_period_selector_label_text_style" => {
            time_input::period_selector_label_text_style(theme)
        }
        other => panic!("unsupported time picker text style role {other}"),
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

fn date_picker_variant(value: &str, case_id: &str) -> DatePickerTokenVariant {
    match value {
        "docked" => DatePickerTokenVariant::Docked,
        "modal" => DatePickerTokenVariant::Modal,
        other => panic!("{case_id}: unsupported date picker variant {other}"),
    }
}
