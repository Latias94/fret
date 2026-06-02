use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::{self, TextIntent};

use crate::button::ButtonVariant;
use crate::card::CardVariant;
use crate::fab::{FabSize, FabVariant};
use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::blend_over;
use crate::icon_button::{IconButtonSize, IconButtonVariant};
use crate::navigation_drawer::NavigationDrawerVariant;
use crate::select::SelectVariant;
use crate::text_field::TextFieldVariant;
use crate::tokens::date_picker::DatePickerTokenVariant;
use crate::tokens::{
    autocomplete, badge, button, card, carousel_item, checkbox, chip, date_picker, dialog, divider,
    dropdown_menu, fab, filter_chip, icon_button as icon_button_tokens, input_chip, list, menu,
    navigation_bar, navigation_drawer, navigation_rail, progress_indicator, radio, search_bar,
    search_view, segmented_button, select, sheet_bottom, slider, snackbar, suggestion_chip, switch,
    tabs, text_field, time_input, time_picker, tooltip, top_app_bar,
    typography as token_typography,
};
use crate::top_app_bar::TopAppBarVariant;

use super::button::ButtonInteraction;
use super::visual_fixture_model::{
    Assertion, Case, Component, SchemeModeFixture, load_suite, theme_for,
};

mod fields;
mod navigation;
mod overlays;
mod selection;
mod surfaces;

#[test]
fn material3_token_visual_fixtures_match_expected_token_outcomes() {
    let suite = load_suite();
    assert_eq!(suite.schema_version, 1);

    for case in &suite.cases {
        let theme = theme_for(&case.scheme);
        match case.component {
            Component::Autocomplete => fields::run_autocomplete_case(case, &theme),
            Component::Badge => surfaces::run_badge_case(case, &theme),
            Component::BottomSheet => overlays::run_bottom_sheet_case(case, &theme),
            Component::Button => surfaces::run_button_case(case, &theme),
            Component::Card => surfaces::run_card_case(case, &theme),
            Component::CarouselItem => surfaces::run_carousel_item_case(case, &theme),
            Component::Checkbox => selection::run_checkbox_case(case, &theme),
            Component::Chip => selection::run_chip_case(case, &theme),
            Component::DatePicker => fields::run_date_picker_case(case, &theme),
            Component::Dialog => overlays::run_dialog_case(case, &theme),
            Component::Divider => surfaces::run_divider_case(case, &theme),
            Component::DropdownMenu => overlays::run_dropdown_menu_case(case, &theme),
            Component::ExposedDropdown => fields::run_autocomplete_case(case, &theme),
            Component::Fab => surfaces::run_fab_case(case, &theme),
            Component::FilterChip => selection::run_filter_chip_case(case, &theme),
            Component::IconButton => selection::run_icon_button_case(case, &theme),
            Component::InputChip => selection::run_input_chip_case(case, &theme),
            Component::List => surfaces::run_list_case(case, &theme),
            Component::Menu => overlays::run_menu_case(case, &theme),
            Component::ModalNavigationDrawer => {
                navigation::run_navigation_drawer_case(case, &theme)
            }
            Component::NavigationBar => navigation::run_navigation_bar_case(case, &theme),
            Component::NavigationDrawer => navigation::run_navigation_drawer_case(case, &theme),
            Component::NavigationRail => navigation::run_navigation_rail_case(case, &theme),
            Component::ProgressIndicator => surfaces::run_progress_indicator_case(case, &theme),
            Component::Radio => selection::run_radio_case(case, &theme),
            Component::SearchBar => fields::run_search_bar_case(case, &theme),
            Component::SearchView => fields::run_search_view_case(case, &theme),
            Component::SegmentedButton => selection::run_segmented_button_case(case, &theme),
            Component::Select => fields::run_select_case(case, &theme),
            Component::Slider => selection::run_slider_case(case, &theme),
            Component::Snackbar => overlays::run_snackbar_case(case, &theme),
            Component::SuggestionChip => selection::run_suggestion_chip_case(case, &theme),
            Component::Switch => selection::run_switch_case(case, &theme),
            Component::Tabs => navigation::run_tabs_case(case, &theme),
            Component::TextField => fields::run_text_field_case(case, &theme),
            Component::TimePicker => fields::run_time_picker_case(case, &theme),
            Component::Tooltip => overlays::run_tooltip_case(case, &theme),
            Component::TopAppBar => navigation::run_top_app_bar_case(case, &theme),
        }
    }
}

#[test]
fn material3_token_visual_fixtures_cover_light_and_dark_scheme_modes() {
    let suite = load_suite();

    assert!(
        suite
            .cases
            .iter()
            .any(|case| matches!(case.scheme.mode, SchemeModeFixture::Light)),
        "visual fixture matrix must cover at least one light scheme case"
    );
    assert!(
        suite
            .cases
            .iter()
            .any(|case| matches!(case.scheme.mode, SchemeModeFixture::Dark)),
        "visual fixture matrix must cover at least one dark scheme case"
    );
}

fn actual_button_optional_color(
    theme: &Theme,
    variant: ButtonVariant,
    enabled: bool,
    interaction: Option<ButtonInteraction>,
    label: Color,
    role: &str,
) -> Option<Color> {
    match role {
        "container_color" => button::container_background(theme, variant, enabled, label),
        _ => Some(actual_button_color(
            theme,
            variant,
            enabled,
            interaction,
            label,
            role,
        )),
    }
}

fn actual_button_color(
    theme: &Theme,
    variant: ButtonVariant,
    enabled: bool,
    interaction: Option<ButtonInteraction>,
    label: Color,
    role: &str,
) -> Color {
    match role {
        "label_color" => label,
        "container_color" => button::container_background(theme, variant, enabled, label)
            .expect("expected button container color"),
        "icon_color" => button::icon_color(theme, variant, enabled, label, interaction),
        "state_layer_color" => button::state_layer_color(theme, variant, label, interaction),
        "shadow_color" => button::container_shadow_color(theme, variant),
        other => panic!("unsupported button color role {other}"),
    }
}

fn actual_button_metric(
    theme: &Theme,
    variant: ButtonVariant,
    enabled: bool,
    interaction: Option<ButtonInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_elevation" => button::container_elevation(theme, variant, enabled, interaction),
        other => panic!("unsupported button metric role {other}"),
    }
}

fn actual_button_number(
    theme: &Theme,
    variant: ButtonVariant,
    interaction: Option<ButtonInteraction>,
    role: &str,
) -> f32 {
    match (role, interaction) {
        ("state_layer_opacity", Some(interaction)) => {
            button::state_layer_opacity(theme, variant, interaction)
        }
        ("state_layer_opacity", None) => panic!("state_layer_opacity requires interaction"),
        _ => panic!("unsupported button number role {role}"),
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

fn actual_checkbox_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: checkbox::CheckboxInteraction,
    role: &str,
) -> Color {
    let chrome = checkbox::chrome(theme, selected, enabled, interaction);
    match role {
        "container_color" => chrome
            .container_bg
            .expect("checkbox container color requires selected state"),
        "outline_color" => chrome
            .outline_color
            .expect("checkbox outline color requires unselected state"),
        "icon_color" => chrome.icon_color,
        "state_layer_color" => checkbox::state_layer_color(theme, selected, interaction),
        other => panic!("unsupported checkbox color role {other}"),
    }
}

fn actual_checkbox_metric(theme: &Theme, role: &str) -> Px {
    let size = checkbox::size_tokens(theme);
    match role {
        "container_size" => size.container,
        "icon_size" => size.icon,
        "state_layer_size" => size.state_layer,
        "container_shape" => size.container_corner,
        other => panic!("unsupported checkbox metric role {other}"),
    }
}

fn actual_checkbox_number(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: checkbox::CheckboxInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => {
            checkbox::state_layer_target_opacity(theme, selected, enabled, interaction)
        }
        "pressed_state_layer_opacity" => checkbox::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported checkbox number role {other}"),
    }
}

fn actual_radio_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: radio::RadioInteraction,
    role: &str,
) -> Color {
    match role {
        "icon_color" => radio::icon_color(theme, selected, enabled, interaction),
        "state_layer_color" => radio::state_layer_color(theme, selected, interaction),
        other => panic!("unsupported radio color role {other}"),
    }
}

fn actual_radio_metric(theme: &Theme, role: &str) -> Px {
    let size = radio::size_tokens(theme);
    match role {
        "icon_size" => size.icon,
        "state_layer_size" => size.state_layer,
        other => panic!("unsupported radio metric role {other}"),
    }
}

fn actual_radio_number(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: radio::RadioInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => {
            radio::state_layer_target_opacity(theme, selected, enabled, interaction)
        }
        "pressed_state_layer_opacity" => radio::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported radio number role {other}"),
    }
}

fn actual_switch_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: switch::SwitchInteraction,
    role: &str,
) -> Color {
    let chrome = switch::chrome(theme, selected, enabled, interaction);
    match role {
        "track_color" => chrome.track_color,
        "outline_color" => chrome
            .outline_color
            .expect("switch outline color requires unselected state"),
        "handle_color" => chrome.handle_color,
        "icon_color" => switch::icon_color(theme, selected, enabled, interaction),
        "state_layer_color" => switch::state_layer_color(theme, selected, interaction),
        other => panic!("unsupported switch color role {other}"),
    }
}

fn actual_switch_metric(theme: &Theme, selected: bool, role: &str) -> Px {
    match role {
        "icon_size" => switch::icon_size(theme, selected),
        other => panic!("unsupported switch metric role {other}"),
    }
}

fn actual_switch_number(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: switch::SwitchInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => {
            switch::state_layer_target_opacity(theme, selected, enabled, interaction)
        }
        "pressed_state_layer_opacity" => switch::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported switch number role {other}"),
    }
}

fn actual_switch_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "track_shape" => switch::track_shape(theme),
        "handle_shape" => switch::handle_shape(theme),
        "state_layer_shape" => switch::state_layer_shape(theme),
        other => panic!("unsupported switch corners role {other}"),
    }
}

fn actual_slider_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: slider::SliderInteraction,
    role: &str,
) -> Color {
    match role {
        "state_layer_color" => slider::state_layer_color(theme, interaction),
        "value_indicator_container_color" => slider::value_indicator_container_color(theme),
        "value_indicator_label_color" => slider::value_indicator_label_color(theme),
        "tick_mark_color" => slider::tick_mark_color(theme, enabled, selected),
        "stop_indicator_color" => slider::stop_indicator_color(theme, enabled, selected),
        "active_track_color" => slider::active_track_color(theme, enabled, interaction),
        "inactive_track_color" => slider::inactive_track_color(theme, enabled, interaction),
        "handle_color" => slider::handle_color(theme, enabled, interaction),
        other => panic!("unsupported slider color role {other}"),
    }
}

fn actual_slider_metric(
    theme: &Theme,
    enabled: bool,
    interaction: slider::SliderInteraction,
    role: &str,
) -> Px {
    match role {
        "state_layer_size" => slider::state_layer_size(theme),
        "value_indicator_bottom_space" => slider::value_indicator_bottom_space(theme),
        "tick_mark_size" => slider::tick_mark_size(theme),
        "stop_indicator_size" => slider::stop_indicator_size(theme),
        "stop_indicator_trailing_space" => slider::stop_indicator_trailing_space(theme),
        "active_track_height" => slider::active_track_height(theme),
        "inactive_track_height" => slider::inactive_track_height(theme),
        "handle_height" => slider::handle_height(theme),
        "handle_width" => slider::handle_width(theme, enabled, interaction),
        other => panic!("unsupported slider metric role {other}"),
    }
}

fn actual_slider_number(
    theme: &Theme,
    enabled: bool,
    selected: bool,
    interaction: slider::SliderInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => slider::state_layer_target_opacity(theme, enabled, interaction),
        "pressed_state_layer_opacity" => slider::pressed_state_layer_opacity(theme),
        "tick_mark_opacity" => slider::tick_mark_opacity(theme, enabled, selected),
        other => panic!("unsupported slider number role {other}"),
    }
}

fn actual_slider_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "tick_mark_shape" => slider::tick_mark_shape(theme),
        "stop_indicator_shape" => slider::stop_indicator_shape(theme),
        "track_shape" => slider::track_shape(theme),
        "handle_shape" => slider::handle_shape(theme),
        other => panic!("unsupported slider corners role {other}"),
    }
}

fn actual_slider_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "value_indicator_label_style" => slider::value_indicator_label_style(theme),
        other => panic!("unsupported slider text style role {other}"),
    }
}

fn actual_segmented_button_optional_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<segmented_button::SegmentedButtonInteraction>,
    role: &str,
) -> Option<Color> {
    match role {
        "container_color" => segmented_button::container_background(theme, selected),
        _ => Some(actual_segmented_button_color(
            theme,
            selected,
            enabled,
            interaction,
            role,
        )),
    }
}

fn actual_segmented_button_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<segmented_button::SegmentedButtonInteraction>,
    role: &str,
) -> Color {
    match role {
        "container_color" => segmented_button::container_background(theme, selected)
            .expect("segmented button container color requires selected state"),
        "outline_color" => segmented_button::outline_color(theme, enabled),
        "label_color" => segmented_button::label_color(theme, selected, enabled, interaction),
        "icon_color" => segmented_button::icon_color(theme, selected, enabled, interaction),
        "state_layer_color" => segmented_button::state_layer_color(
            theme,
            selected,
            interaction.expect("segmented button state layer color requires interaction"),
        ),
        other => panic!("unsupported segmented button color role {other}"),
    }
}

fn actual_segmented_button_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "container_height" => segmented_button::container_height(theme),
        "outline_width" => segmented_button::outline_width(theme),
        "shape" => segmented_button::shape_radius(theme),
        "icon_size" => segmented_button::icon_size(theme),
        other => panic!("unsupported segmented button metric role {other}"),
    }
}

fn actual_segmented_button_number(
    theme: &Theme,
    interaction: Option<segmented_button::SegmentedButtonInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => segmented_button::state_layer_opacity(
            theme,
            interaction.expect("segmented button state layer opacity requires interaction"),
        ),
        "pressed_state_layer_opacity" => segmented_button::pressed_state_layer_opacity(theme),
        other => panic!("unsupported segmented button number role {other}"),
    }
}

fn actual_icon_button_color(
    theme: &Theme,
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
    enabled: bool,
    interaction: Option<icon_button_tokens::IconButtonInteraction>,
    icon: Color,
    role: &str,
) -> Color {
    match role {
        "icon_color" => {
            icon_button_tokens::icon_color(theme, variant, toggle, selected, enabled, interaction)
        }
        "state_layer_color" => icon_button_tokens::state_layer_color(
            theme,
            variant,
            toggle,
            selected,
            enabled,
            interaction,
        ),
        "container_color" => icon_button_tokens::container_background(
            theme, variant, toggle, selected, enabled, icon,
        )
        .expect("icon button container color requires filled, tonal, or selected outlined variant"),
        "outline_color" => icon_button_tokens::outlined_outline_color(theme, enabled),
        other => panic!("unsupported icon button color role {other}"),
    }
}

fn actual_icon_button_metric(theme: &Theme, role: &str) -> Px {
    let size = icon_button_tokens::size_tokens(theme, IconButtonSize::Small);
    match role {
        "container_size" => size.container,
        "leading_space" => size.pad_left,
        "trailing_space" => size.pad_right,
        "icon_size" => size.icon_size,
        "outline_width" => size.outline_width,
        "container_shape" => Px(icon_button_tokens::container_shape_radius(theme)),
        "selected_container_shape" => {
            Px(icon_button_tokens::selected_container_shape_radius(theme))
        }
        "pressed_container_shape" => Px(icon_button_tokens::pressed_container_shape_radius(theme)),
        other => panic!("unsupported icon button metric role {other}"),
    }
}

fn actual_icon_button_number(
    theme: &Theme,
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
    interaction: Option<icon_button_tokens::IconButtonInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => icon_button_tokens::state_layer_opacity(
            theme,
            variant,
            toggle,
            selected,
            interaction.expect("icon button state layer opacity requires interaction"),
        ),
        "pressed_state_layer_opacity" => {
            icon_button_tokens::pressed_state_layer_opacity(theme, variant, toggle, selected)
        }
        other => panic!("unsupported icon button number role {other}"),
    }
}

fn actual_chip_color(
    theme: &Theme,
    variant: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "label_color" => chip::label_color(theme, enabled, interaction),
        "leading_icon_color" => chip::leading_icon_color(theme, enabled, interaction),
        "state_layer_color" => chip::state_layer_color(theme, interaction),
        "container_color" if variant == "elevated" => {
            chip::elevated_container_background(theme, enabled)
        }
        "shadow_color" if variant == "elevated" => chip::elevated_container_shadow_color(theme),
        "outline_color" if variant == "flat" => {
            chip::flat_outline(theme, enabled, interaction)
                .expect("assist chip flat outline should exist")
                .color
        }
        other => panic!("unsupported assist chip color role {other}"),
    }
}

fn actual_chip_metric(
    theme: &Theme,
    variant: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_height" => chip::container_height(theme),
        "leading_icon_size" => chip::leading_icon_size(theme),
        "container_elevation" if variant == "elevated" => {
            chip::elevated_container_elevation(theme, enabled, interaction)
        }
        "outline_width" if variant == "flat" => {
            chip::flat_outline(theme, enabled, interaction)
                .expect("assist chip flat outline should exist")
                .width
        }
        other => panic!("unsupported assist chip metric role {other}"),
    }
}

fn actual_chip_number(theme: &Theme, interaction: Option<PressableInteraction>, role: &str) -> f32 {
    match role {
        "state_layer_opacity" => chip::state_layer_opacity(theme, interaction),
        "pressed_state_layer_opacity" => chip::pressed_state_layer_opacity(theme),
        other => panic!("unsupported assist chip number role {other}"),
    }
}

fn actual_chip_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => chip::container_shape(theme),
        other => panic!("unsupported assist chip corners role {other}"),
    }
}

fn actual_chip_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "label_text_style" => chip::label_text_style(theme),
        other => panic!("unsupported assist chip text style role {other}"),
    }
}

fn actual_filter_chip_color(
    theme: &Theme,
    variant: &str,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "label_color" => filter_chip::label_color(theme, selected, enabled, interaction),
        "leading_icon_color" => {
            filter_chip::leading_icon_color(theme, selected, enabled, interaction)
        }
        "trailing_icon_color" => {
            filter_chip::trailing_icon_color(theme, selected, enabled, interaction)
        }
        "state_layer_color" => filter_chip::state_layer_color(theme, selected, interaction),
        "container_color" if variant == "elevated" => {
            filter_chip::elevated_container_background(theme, selected, enabled)
        }
        "container_color" if variant == "flat" && selected => {
            filter_chip::flat_selected_container_background(theme, enabled)
        }
        "shadow_color" if variant == "elevated" => {
            filter_chip::elevated_container_shadow_color(theme)
        }
        "outline_color" if variant == "flat" && !selected => {
            filter_chip::flat_unselected_outline(theme, enabled, interaction).color
        }
        other => panic!("unsupported filter chip color role {other}"),
    }
}

fn actual_filter_chip_metric(
    theme: &Theme,
    variant: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_height" => filter_chip::container_height(theme),
        "leading_icon_size" => filter_chip::leading_icon_size(theme),
        "trailing_icon_size" => filter_chip::trailing_icon_size(theme),
        "container_elevation" if variant == "elevated" => {
            filter_chip::elevated_container_elevation(theme, enabled, interaction)
        }
        "outline_width" if variant == "flat" => {
            filter_chip::flat_unselected_outline(theme, enabled, interaction).width
        }
        other => panic!("unsupported filter chip metric role {other}"),
    }
}

fn actual_filter_chip_number(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => filter_chip::state_layer_opacity(theme, selected, interaction),
        "pressed_state_layer_opacity" => filter_chip::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported filter chip number role {other}"),
    }
}

fn actual_filter_chip_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => filter_chip::container_shape(theme),
        other => panic!("unsupported filter chip corners role {other}"),
    }
}

fn actual_filter_chip_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "label_text_style" => filter_chip::label_text_style(theme),
        other => panic!("unsupported filter chip text style role {other}"),
    }
}

fn actual_input_chip_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "label_color" => input_chip::label_color(theme, selected, enabled, interaction),
        "leading_icon_color" => {
            input_chip::leading_icon_color(theme, selected, enabled, interaction)
        }
        "trailing_icon_color" => {
            input_chip::trailing_icon_color(theme, selected, enabled, interaction)
        }
        "state_layer_color" => input_chip::state_layer_color(theme, selected, interaction),
        "container_color" if selected => input_chip::selected_container_background(theme, enabled),
        "outline_color" if !selected => {
            input_chip::unselected_outline(theme, enabled, interaction).color
        }
        other => panic!("unsupported input chip color role {other}"),
    }
}

fn actual_input_chip_metric(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_height" => input_chip::container_height(theme),
        "leading_icon_size" => input_chip::leading_icon_size(theme),
        "trailing_icon_size" => input_chip::trailing_icon_size(theme),
        "outline_width" => input_chip::unselected_outline(theme, enabled, interaction).width,
        other => panic!("unsupported input chip metric role {other}"),
    }
}

fn actual_input_chip_number(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => input_chip::state_layer_opacity(theme, selected, interaction),
        "pressed_state_layer_opacity" => input_chip::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported input chip number role {other}"),
    }
}

fn actual_input_chip_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => input_chip::container_shape(theme),
        other => panic!("unsupported input chip corners role {other}"),
    }
}

fn actual_input_chip_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "label_text_style" => input_chip::label_text_style(theme),
        other => panic!("unsupported input chip text style role {other}"),
    }
}

fn actual_suggestion_chip_color(
    theme: &Theme,
    variant: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "label_color" => suggestion_chip::label_color(theme, enabled, interaction),
        "leading_icon_color" => suggestion_chip::leading_icon_color(theme, enabled, interaction),
        "state_layer_color" => suggestion_chip::state_layer_color(theme, interaction),
        "container_color" if variant == "elevated" => {
            suggestion_chip::elevated_container_background(theme, enabled)
        }
        "shadow_color" if variant == "elevated" => {
            suggestion_chip::elevated_container_shadow_color(theme)
        }
        "outline_color" if variant == "flat" => {
            suggestion_chip::flat_outline(theme, enabled, interaction).color
        }
        other => panic!("unsupported suggestion chip color role {other}"),
    }
}

fn actual_suggestion_chip_metric(
    theme: &Theme,
    variant: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_height" => suggestion_chip::container_height(theme),
        "leading_icon_size" => suggestion_chip::leading_icon_size(theme),
        "container_elevation" if variant == "elevated" => {
            suggestion_chip::elevated_container_elevation(theme, enabled, interaction)
        }
        "outline_width" if variant == "flat" => {
            suggestion_chip::flat_outline(theme, enabled, interaction).width
        }
        other => panic!("unsupported suggestion chip metric role {other}"),
    }
}

fn actual_suggestion_chip_number(
    theme: &Theme,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => suggestion_chip::state_layer_opacity(theme, interaction),
        "pressed_state_layer_opacity" => suggestion_chip::pressed_state_layer_opacity(theme),
        other => panic!("unsupported suggestion chip number role {other}"),
    }
}

fn actual_suggestion_chip_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => suggestion_chip::container_shape(theme),
        other => panic!("unsupported suggestion chip corners role {other}"),
    }
}

fn actual_suggestion_chip_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "label_text_style" => suggestion_chip::label_text_style(theme),
        other => panic!("unsupported suggestion chip text style role {other}"),
    }
}

fn actual_tabs_color(
    theme: &Theme,
    active: bool,
    interaction: tabs::TabInteraction,
    role: &str,
) -> Color {
    let kind = tabs::NavigationTabKind::Primary;
    match role {
        "container_color" => tabs::container_background_for(theme, kind),
        "active_indicator_color" => tabs::active_indicator_color(theme),
        "label_color" => tabs::label_color_for(theme, kind, active, interaction),
        "state_layer_color" => tabs::state_layer_color_for(theme, kind, active, interaction),
        other => panic!("unsupported tabs color role {other}"),
    }
}

fn actual_tabs_metric(theme: &Theme, variant: &str, role: &str) -> Px {
    let kind = tabs::NavigationTabKind::Primary;
    match role {
        "container_height" => tabs::container_height_for(theme, kind),
        "active_indicator_height" => tabs::active_indicator_height(theme),
        "active_indicator_min_width" => tabs::active_indicator_min_width(theme),
        "scrollable_edge_padding" if variant == "scrollable" => {
            tabs::scrollable_edge_padding_for(theme, kind)
        }
        "scrollable_min_tab_width" if variant == "scrollable" => {
            tabs::scrollable_min_tab_width_for(theme, kind)
        }
        other => panic!("unsupported tabs metric role {other}"),
    }
}

fn actual_tabs_number(
    theme: &Theme,
    active: bool,
    interaction: tabs::TabInteraction,
    role: &str,
) -> f32 {
    let kind = tabs::NavigationTabKind::Primary;
    match role {
        "state_layer_opacity" => tabs::state_layer_opacity_for(theme, kind, active, interaction),
        "pressed_state_layer_opacity" => tabs::pressed_state_layer_opacity_for(theme, kind, active),
        other => panic!("unsupported tabs number role {other}"),
    }
}

fn actual_tabs_corners(theme: &Theme, role: &str) -> Corners {
    let kind = tabs::NavigationTabKind::Primary;
    match role {
        "active_indicator_shape" => tabs::active_indicator_shape_for(theme, kind),
        other => panic!("unsupported tabs corners role {other}"),
    }
}

fn actual_tabs_text_style(theme: &Theme, role: &str) -> TextStyle {
    let kind = tabs::NavigationTabKind::Primary;
    match role {
        "label_text_style" => tabs::label_text_style_for(theme, kind),
        other => panic!("unsupported tabs text style role {other}"),
    }
}

fn actual_navigation_bar_color(
    theme: &Theme,
    active: bool,
    interaction: navigation_bar::NavigationBarItemInteraction,
    role: &str,
) -> Color {
    match role {
        "container_color" => navigation_bar::container_background(theme),
        "container_shadow_color" => navigation_bar::container_shadow_color(theme),
        "active_indicator_color" => navigation_bar::active_indicator_color(theme),
        "label_color" => navigation_bar::label_color(theme, active, interaction),
        "icon_color" => navigation_bar::icon_color(theme, active, interaction),
        "state_layer_color" => navigation_bar::state_layer_color(theme, active, interaction),
        other => panic!("unsupported navigation bar color role {other}"),
    }
}

fn actual_navigation_bar_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "container_height" => navigation_bar::container_height(theme),
        "container_elevation" => navigation_bar::container_elevation(theme),
        "active_indicator_width" => navigation_bar::active_indicator_width(theme),
        "active_indicator_height" => navigation_bar::active_indicator_height(theme),
        "active_indicator_top_offset" => navigation_bar::active_indicator_top_offset(theme),
        "icon_size" => navigation_bar::icon_size(theme),
        "item_gap" => navigation_bar::item_gap(theme),
        other => panic!("unsupported navigation bar metric role {other}"),
    }
}

fn actual_navigation_bar_number(
    theme: &Theme,
    interaction: navigation_bar::NavigationBarItemInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => navigation_bar::state_layer_opacity(theme, interaction),
        "pressed_state_layer_opacity" => navigation_bar::pressed_state_layer_opacity(theme),
        other => panic!("unsupported navigation bar number role {other}"),
    }
}

fn actual_navigation_bar_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => navigation_bar::container_shape(theme),
        "active_indicator_shape" => navigation_bar::active_indicator_shape(theme),
        other => panic!("unsupported navigation bar corners role {other}"),
    }
}

fn actual_navigation_bar_text_style(theme: &Theme, active: bool, role: &str) -> TextStyle {
    match role {
        "label_text_style" => navigation_bar::label_text_style(theme, active),
        other => panic!("unsupported navigation bar text style role {other}"),
    }
}

fn actual_navigation_rail_color(
    theme: &Theme,
    active: bool,
    interaction: navigation_rail::NavigationRailItemInteraction,
    role: &str,
) -> Color {
    match role {
        "container_color" => navigation_rail::container_background(theme),
        "active_indicator_color" => navigation_rail::active_indicator_color(theme),
        "label_color" => navigation_rail::label_color(theme, active, interaction),
        "icon_color" => navigation_rail::icon_color(theme, active, interaction),
        "state_layer_color" => navigation_rail::state_layer_color(theme, active, interaction),
        other => panic!("unsupported navigation rail color role {other}"),
    }
}

fn actual_navigation_rail_metric(theme: &Theme, variant: &str, role: &str) -> Px {
    let has_label = variant != "no_label";
    match role {
        "container_width" => navigation_rail::container_width(theme),
        "item_width" => navigation_rail::item_width(theme),
        "item_height" => navigation_rail::item_height(theme),
        "vertical_padding" => navigation_rail::vertical_padding(theme),
        "active_indicator_width" => navigation_rail::active_indicator_width(theme),
        "active_indicator_height" => navigation_rail::active_indicator_height(theme, has_label),
        "icon_size" => navigation_rail::icon_size(theme),
        other => panic!("unsupported navigation rail metric role {other}"),
    }
}

fn actual_navigation_rail_number(
    theme: &Theme,
    interaction: navigation_rail::NavigationRailItemInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => navigation_rail::state_layer_opacity(theme, interaction),
        "pressed_state_layer_opacity" => navigation_rail::pressed_state_layer_opacity(theme),
        other => panic!("unsupported navigation rail number role {other}"),
    }
}

fn actual_navigation_rail_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => navigation_rail::container_shape(theme),
        "active_indicator_shape" => navigation_rail::active_indicator_shape(theme),
        other => panic!("unsupported navigation rail corners role {other}"),
    }
}

fn actual_navigation_rail_text_style(theme: &Theme, active: bool, role: &str) -> TextStyle {
    match role {
        "label_text_style" => navigation_rail::label_text_style(theme, active),
        other => panic!("unsupported navigation rail text style role {other}"),
    }
}

fn actual_navigation_drawer_color(
    theme: &Theme,
    variant: NavigationDrawerVariant,
    active: bool,
    interaction: navigation_drawer::NavigationDrawerItemInteraction,
    role: &str,
) -> Color {
    match role {
        "container_color" => navigation_drawer::container_background(theme, variant),
        "active_indicator_color" => navigation_drawer::active_indicator_color(theme),
        "label_color" => navigation_drawer::label_color(theme, active, interaction),
        "icon_color" => navigation_drawer::icon_color(theme, active, interaction),
        "state_layer_color" => navigation_drawer::state_layer_color(theme, active, interaction),
        "large_badge_label_color" => navigation_drawer::large_badge_label_color(theme),
        "scrim_color" => navigation_drawer::scrim_color(theme),
        "scrim_color_alpha" => alpha_color(
            navigation_drawer::scrim_color(theme),
            navigation_drawer::scrim_opacity(theme),
        ),
        other => panic!("unsupported navigation drawer color role {other}"),
    }
}

fn actual_navigation_drawer_metric(
    theme: &Theme,
    variant: NavigationDrawerVariant,
    role: &str,
) -> Px {
    match role {
        "container_width" => navigation_drawer::container_width(theme),
        "container_elevation" => navigation_drawer::container_elevation(theme, variant),
        "item_horizontal_padding" => navigation_drawer::item_horizontal_padding(theme),
        "active_indicator_width" => navigation_drawer::active_indicator_width(theme),
        "active_indicator_height" => navigation_drawer::active_indicator_height(theme),
        "icon_size" => navigation_drawer::icon_size(theme),
        other => panic!("unsupported navigation drawer metric role {other}"),
    }
}

fn actual_navigation_drawer_number(
    theme: &Theme,
    interaction: navigation_drawer::NavigationDrawerItemInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => {
            navigation_drawer::state_layer_target_opacity(theme, true, interaction)
        }
        "pressed_state_layer_opacity" => navigation_drawer::pressed_state_layer_opacity(theme),
        "scrim_opacity" => navigation_drawer::scrim_opacity(theme),
        other => panic!("unsupported navigation drawer number role {other}"),
    }
}

fn actual_navigation_drawer_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => navigation_drawer::container_shape(theme),
        "active_indicator_shape" => navigation_drawer::active_indicator_shape(theme),
        other => panic!("unsupported navigation drawer corners role {other}"),
    }
}

fn actual_navigation_drawer_text_style(theme: &Theme, active: bool, role: &str) -> TextStyle {
    match role {
        "label_text_style" => navigation_drawer::label_text_style(theme, active),
        "large_badge_label_text_style" => navigation_drawer::large_badge_label_text_style(theme),
        other => panic!("unsupported navigation drawer text style role {other}"),
    }
}

fn actual_top_app_bar_color(
    theme: &Theme,
    variant: TopAppBarVariant,
    scrolled: bool,
    role: &str,
) -> Color {
    match role {
        "container_color" => top_app_bar::container_background(theme, variant, scrolled),
        "headline_color" => top_app_bar::headline_color(theme, variant),
        "leading_icon_color" => top_app_bar::leading_icon_color(theme, variant),
        "trailing_icon_color" => top_app_bar::trailing_icon_color(theme, variant),
        other => panic!("unsupported top app bar color role {other}"),
    }
}

fn actual_top_app_bar_metric(
    theme: &Theme,
    variant: TopAppBarVariant,
    scrolled: bool,
    role: &str,
) -> Px {
    match role {
        "container_height" => top_app_bar::container_height(theme, variant),
        "container_elevation" => top_app_bar::container_elevation(theme, variant, scrolled),
        other => panic!("unsupported top app bar metric role {other}"),
    }
}

fn actual_top_app_bar_corners(theme: &Theme, variant: TopAppBarVariant, role: &str) -> Corners {
    match role {
        "container_shape" => top_app_bar::container_shape(theme, variant),
        other => panic!("unsupported top app bar corners role {other}"),
    }
}

fn actual_top_app_bar_text_style(
    theme: &Theme,
    variant: TopAppBarVariant,
    role: &str,
) -> TextStyle {
    match role {
        "headline_text_style" => top_app_bar::headline_text_style(theme, variant),
        other => panic!("unsupported top app bar text style role {other}"),
    }
}

fn actual_menu_color(
    theme: &Theme,
    enabled: bool,
    interaction: menu::MenuItemInteraction,
    role: &str,
) -> Color {
    let (label, state_layer, _) = menu::item_outcomes(theme, enabled, interaction);
    match role {
        "container_color" => menu::container_background(theme),
        "container_shadow_color" => menu::container_shadow_color(theme),
        "divider_color" => menu::divider_color(theme),
        "item_label_color" | "label_color" => label,
        "item_icon_color" => menu::item_icon_color(theme, enabled, interaction),
        "item_supporting_text_color" => menu::item_supporting_text_color(theme, enabled),
        "item_trailing_text_color" => menu::item_trailing_text_color(theme, enabled),
        "section_label_color" => menu::section_label_color(theme),
        "state_layer_color" => state_layer,
        other => panic!("unsupported menu color role {other}"),
    }
}

fn actual_menu_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "item_height" => menu::list_item_height(theme),
        "item_two_line_height" => menu::list_item_two_line_height(theme),
        "item_min_width" => menu::item_min_width(theme),
        "item_max_width" => menu::item_max_width(theme),
        "container_vertical_padding" => menu::container_vertical_padding(theme),
        "item_horizontal_padding" => menu::item_horizontal_padding(theme),
        "item_slot_gap" => menu::item_slot_gap(theme),
        "item_icon_size" => menu::item_icon_size(theme),
        "section_label_height" => menu::section_label_height(theme),
        "container_elevation" => menu::container_elevation(theme),
        "divider_height" => menu::divider_height(theme),
        other => panic!("unsupported menu metric role {other}"),
    }
}

fn actual_dropdown_menu_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "divider_margin_total" => dropdown_menu::divider_margin_total(theme),
        "collision_padding" => dropdown_menu::collision_padding(theme).left,
        "max_height" => dropdown_menu::max_height(theme),
        other => actual_menu_metric(theme, other),
    }
}

fn actual_menu_number(
    theme: &Theme,
    enabled: bool,
    interaction: menu::MenuItemInteraction,
    role: &str,
) -> f32 {
    let (_, _, state_layer_opacity) = menu::item_outcomes(theme, enabled, interaction);
    match role {
        "state_layer_opacity" => state_layer_opacity,
        "pressed_state_layer_opacity" => menu::pressed_state_layer_opacity(theme),
        other => panic!("unsupported menu number role {other}"),
    }
}

fn actual_menu_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => menu::container_shape(theme),
        other => panic!("unsupported menu corners role {other}"),
    }
}

fn actual_menu_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "item_label_text_style" | "label_text_style" => menu::item_label_text_style(theme),
        "item_supporting_text_style" => menu::item_supporting_text_style(theme),
        "item_trailing_text_style" => menu::item_trailing_text_style(theme),
        "section_label_text_style" => menu::section_label_text_style(theme),
        other => panic!("unsupported menu text style role {other}"),
    }
}

fn actual_dialog_color(
    theme: &Theme,
    interaction: dialog::DialogActionInteraction,
    role: &str,
) -> Color {
    match role {
        "scrim_color" => dialog::scrim_color(theme),
        "scrim_color_alpha" => alpha_color(
            dialog::scrim_color(theme),
            dialog::scrim_opacity(theme, 0.32),
        ),
        "container_color" => dialog::container_background(theme),
        "container_shadow_color" => dialog::container_shadow_color(theme),
        "headline_color" => dialog::headline_color(theme),
        "supporting_text_color" => dialog::supporting_text_color(theme),
        "action_label_color" => dialog::action_label_color(theme, interaction),
        "action_state_layer_color" => dialog::action_state_layer_color(theme, interaction),
        other => panic!("unsupported dialog color role {other}"),
    }
}

fn actual_dialog_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "container_elevation" => dialog::container_elevation(theme),
        "action_height" => dialog::action_height(theme),
        "container_min_width" => dialog::container_min_width(theme),
        "container_max_width" => dialog::container_max_width(theme),
        other => panic!("unsupported dialog metric role {other}"),
    }
}

fn actual_dialog_number(
    theme: &Theme,
    interaction: dialog::DialogActionInteraction,
    role: &str,
) -> f32 {
    match role {
        "scrim_opacity" => dialog::scrim_opacity(theme, 0.32),
        "action_state_layer_opacity" => {
            dialog::action_state_layer_target_opacity(theme, interaction)
        }
        "action_pressed_state_layer_opacity" => dialog::action_pressed_state_layer_opacity(theme),
        other => panic!("unsupported dialog number role {other}"),
    }
}

fn actual_dialog_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => dialog::container_shape(theme),
        "action_corner_radii" => dialog::action_corner_radii(theme),
        other => panic!("unsupported dialog corners role {other}"),
    }
}

fn actual_dialog_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "headline_text_style" => dialog::headline_text_style(theme),
        "supporting_text_style" => dialog::supporting_text_style(theme),
        "action_label_text_style" => dialog::action_label_text_style(theme),
        other => panic!("unsupported dialog text style role {other}"),
    }
}

fn actual_bottom_sheet_color(theme: &Theme, role: &str) -> Color {
    match role {
        "scrim_color" => sheet_bottom::modal_scrim_color(theme),
        "scrim_color_alpha" => alpha_color(
            sheet_bottom::modal_scrim_color(theme),
            sheet_bottom::modal_scrim_opacity(theme, 0.32),
        ),
        "container_color" => sheet_bottom::docked_container_color(theme),
        "drag_handle_color" => sheet_bottom::docked_drag_handle_color(theme),
        "drag_handle_color_alpha" => alpha_color(
            sheet_bottom::docked_drag_handle_color(theme),
            sheet_bottom::docked_drag_handle_opacity(theme),
        ),
        "focus_indicator_color" => sheet_bottom::focus_indicator_color(theme),
        other => panic!("unsupported bottom sheet color role {other}"),
    }
}

fn actual_bottom_sheet_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "modal_container_elevation" => sheet_bottom::docked_modal_elevation(theme),
        "standard_container_elevation" => sheet_bottom::docked_standard_elevation(theme),
        "drag_handle_width" => sheet_bottom::docked_drag_handle_width(theme),
        "drag_handle_height" => sheet_bottom::docked_drag_handle_height(theme),
        "focus_indicator_thickness" => sheet_bottom::focus_indicator_thickness(theme),
        "focus_indicator_outline_offset" => sheet_bottom::focus_indicator_outline_offset(theme),
        other => panic!("unsupported bottom sheet metric role {other}"),
    }
}

fn actual_bottom_sheet_number(theme: &Theme, role: &str) -> f32 {
    match role {
        "scrim_opacity" => sheet_bottom::modal_scrim_opacity(theme, 0.32),
        "drag_handle_opacity" => sheet_bottom::docked_drag_handle_opacity(theme),
        other => panic!("unsupported bottom sheet number role {other}"),
    }
}

fn actual_bottom_sheet_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => sheet_bottom::docked_container_shape(theme),
        other => panic!("unsupported bottom sheet corners role {other}"),
    }
}

fn actual_tooltip_color(theme: &Theme, variant: &str, role: &str) -> Color {
    match (variant, role) {
        ("plain", "container_color") => tooltip::plain_container_background(theme),
        ("plain", "supporting_text_color") => tooltip::plain_supporting_text_color(theme),
        ("rich", "container_color") => tooltip::rich_container_background(theme),
        ("rich", "container_shadow_color") => tooltip::rich_container_shadow_color(theme),
        ("rich", "subhead_color") => tooltip::rich_subhead_color(theme),
        ("rich", "supporting_text_color") => tooltip::rich_supporting_text_color(theme),
        (_, "shadow_color") => tooltip::shadow_color(theme),
        _ => panic!("unsupported tooltip color role {variant}:{role}"),
    }
}

fn actual_tooltip_metric(theme: &Theme, variant: &str, role: &str) -> Px {
    match (variant, role) {
        ("plain", "container_max_width") => tooltip::plain_container_max_width(theme),
        ("rich", "container_max_width") => tooltip::rich_container_max_width(theme),
        ("rich", "container_elevation") => tooltip::rich_container_elevation(theme),
        (_, "container_min_width") => tooltip::container_min_width(theme),
        (_, "container_min_height") => tooltip::container_min_height(theme),
        _ => panic!("unsupported tooltip metric role {variant}:{role}"),
    }
}

fn actual_tooltip_corners(theme: &Theme, variant: &str, role: &str) -> Corners {
    match (variant, role) {
        ("plain", "container_shape") => tooltip::plain_container_shape(theme),
        ("rich", "container_shape") => tooltip::rich_container_shape(theme),
        _ => panic!("unsupported tooltip corners role {variant}:{role}"),
    }
}

fn actual_tooltip_text_style(theme: &Theme, variant: &str, role: &str) -> TextStyle {
    match (variant, role) {
        ("plain", "supporting_text_style") => tooltip::plain_supporting_text_style(theme),
        ("rich", "subhead_text_style") => tooltip::rich_subhead_text_style(theme),
        ("rich", "supporting_text_style") => tooltip::rich_supporting_text_style(theme),
        _ => panic!("unsupported tooltip text style role {variant}:{role}"),
    }
}

fn actual_snackbar_color(
    theme: &Theme,
    interaction: snackbar::SnackbarActionInteraction,
    role: &str,
) -> Color {
    match role {
        "container_color" => snackbar::container_background(theme),
        "container_shadow_color" => snackbar::container_shadow_color(theme),
        "supporting_text_color" => snackbar::supporting_text_color(theme),
        "action_label_color" => snackbar::action_label_color(theme, interaction),
        "action_state_layer_color" => snackbar::action_state_layer_color(theme, interaction),
        "icon_color" => snackbar::icon_color(theme, interaction),
        "icon_state_layer_color" => snackbar::icon_state_layer_color(theme, interaction),
        other => panic!("unsupported snackbar color role {other}"),
    }
}

fn actual_snackbar_metric(theme: &Theme, variant: &str, role: &str) -> Px {
    match (variant, role) {
        (_, "icon_size") => snackbar::icon_size(theme),
        (_, "container_elevation") => snackbar::container_elevation(theme),
        ("single_line", "container_height") => snackbar::single_line_min_height(theme)
            .unwrap_or_else(|| panic!("expected single line snackbar height")),
        ("two_line", "container_height") => snackbar::two_line_min_height(theme)
            .unwrap_or_else(|| panic!("expected two line snackbar height")),
        _ => panic!("unsupported snackbar metric role {variant}:{role}"),
    }
}

fn actual_snackbar_number(
    theme: &Theme,
    interaction: snackbar::SnackbarActionInteraction,
    role: &str,
) -> f32 {
    match role {
        "action_state_layer_opacity" => snackbar::action_state_layer_opacity(theme, interaction),
        "icon_state_layer_opacity" => snackbar::icon_state_layer_opacity(theme, interaction),
        other => panic!("unsupported snackbar number role {other}"),
    }
}

fn actual_snackbar_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => snackbar::container_shape(theme),
        other => panic!("unsupported snackbar corners role {other}"),
    }
}

fn actual_snackbar_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "supporting_text_style" => snackbar::supporting_text_style(theme),
        "action_label_text_style" => snackbar::action_label_text_style(theme),
        other => panic!("unsupported snackbar text style role {other}"),
    }
}

fn actual_card_color(
    theme: &Theme,
    variant: CardVariant,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "container_color" => card::container_background(theme, variant, enabled),
        "container_shadow_color" => card::container_shadow_color(theme, variant),
        "outline_color" => {
            card::outline(theme, variant, enabled, interaction)
                .unwrap_or_else(|| panic!("expected card outline"))
                .color
        }
        "state_layer_color" => card::state_layer_color(theme, variant, interaction),
        other => panic!("unsupported card color role {other}"),
    }
}

fn actual_card_metric(
    theme: &Theme,
    variant: CardVariant,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_elevation" => card::container_elevation(theme, variant, enabled, interaction),
        "outline_width" => {
            card::outline(theme, variant, enabled, interaction)
                .unwrap_or_else(|| panic!("expected card outline"))
                .width
        }
        other => panic!("unsupported card metric role {other}"),
    }
}

fn actual_card_number(
    theme: &Theme,
    variant: CardVariant,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => card::state_layer_opacity(theme, variant, interaction),
        "pressed_state_layer_opacity" => card::pressed_state_layer_opacity(theme, variant),
        other => panic!("unsupported card number role {other}"),
    }
}

fn actual_card_corners(theme: &Theme, variant: CardVariant, role: &str) -> Corners {
    match role {
        "container_shape" => card::container_shape(theme, variant),
        other => panic!("unsupported card corners role {other}"),
    }
}

fn actual_badge_color(theme: &Theme, role: &str) -> Color {
    match role {
        "dot_color" => badge::dot_color(theme),
        "large_color" => badge::large_color(theme),
        "large_label_color" => badge::large_label_color(theme),
        other => panic!("unsupported badge color role {other}"),
    }
}

fn actual_badge_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "dot_size" => badge::dot_size(theme),
        "large_size" => badge::large_size(theme),
        other => panic!("unsupported badge metric role {other}"),
    }
}

fn actual_badge_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "dot_shape" => badge::shape(theme),
        "large_shape" => badge::large_shape(theme),
        other => panic!("unsupported badge corners role {other}"),
    }
}

fn actual_badge_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "large_label_text_style" => badge::large_label_text_style(theme),
        other => panic!("unsupported badge text style role {other}"),
    }
}

fn actual_fab_color(
    theme: &Theme,
    extended: bool,
    variant: FabVariant,
    enabled: bool,
    interaction: Option<fab::FabInteraction>,
    role: &str,
) -> Color {
    match role {
        "container_color" => fab::container_background(theme, extended, variant, enabled, false),
        "container_shadow_color" => fab::container_shadow_color(theme, extended, variant),
        "icon_color" => fab::icon_color(theme, extended, variant, enabled, interaction),
        "label_color" => fab::label_color(theme, variant, enabled, interaction),
        "state_layer_color" => fab::state_layer_color(
            theme,
            extended,
            variant,
            interaction.unwrap_or(fab::FabInteraction::Pressed),
        ),
        other => panic!("unsupported fab color role {other}"),
    }
}

fn actual_fab_metric(
    theme: &Theme,
    extended: bool,
    size: FabSize,
    variant: FabVariant,
    enabled: bool,
    interaction: Option<fab::FabInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_size" => fab::container_size(theme, size),
        "icon_size" => fab::icon_size(theme, size),
        "extended_container_height" => fab::extended_container_height(theme, size),
        "extended_min_width" => fab::extended_min_width(theme, size),
        "extended_icon_size" => fab::extended_icon_size(theme, size),
        "extended_leading_space" => fab::extended_leading_space(theme, size, true),
        "extended_trailing_space" => fab::extended_trailing_space(theme, size),
        "extended_icon_label_space" => fab::extended_icon_label_space(theme, size),
        "container_elevation" => {
            fab::container_elevation(theme, extended, variant, enabled, false, interaction)
        }
        other => panic!("unsupported fab metric role {other}"),
    }
}

fn actual_fab_number(
    theme: &Theme,
    extended: bool,
    variant: FabVariant,
    interaction: Option<fab::FabInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => fab::state_layer_opacity(
            theme,
            extended,
            variant,
            interaction.unwrap_or(fab::FabInteraction::Pressed),
        ),
        "pressed_state_layer_opacity" => {
            fab::pressed_state_layer_opacity_for_variant(theme, extended, variant)
        }
        other => panic!("unsupported fab number role {other}"),
    }
}

fn actual_fab_corners(theme: &Theme, extended: bool, size: FabSize, role: &str) -> Corners {
    match (extended, role) {
        (false, "container_shape") => fab::container_shape(theme, size),
        (true, "container_shape") => fab::extended_container_shape(theme, size),
        _ => panic!("unsupported fab corners role {role}"),
    }
}

fn actual_fab_text_style(
    theme: &Theme,
    size: FabSize,
    variant: FabVariant,
    role: &str,
) -> TextStyle {
    match role {
        "label_text_style" => fab::extended_label_text_style(theme, size, variant),
        other => panic!("unsupported fab text style role {other}"),
    }
}

fn actual_list_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: list::ListItemInteraction,
    role: &str,
) -> Color {
    let (label, icon, state_layer, _) = list::item_outcomes(theme, selected, enabled, interaction);
    match role {
        "selected_container_color" => list::selected_container_background(theme, enabled),
        "label_color" => label,
        "leading_icon_color" | "icon_color" => icon,
        "state_layer_color" => state_layer,
        "supporting_text_color" => list::supporting_text_color(theme, enabled, selected),
        "overline_text_color" => list::overline_text_color(theme, enabled, selected),
        "trailing_supporting_text_color" => {
            list::trailing_supporting_text_color(theme, enabled, selected)
        }
        other => panic!("unsupported list color role {other}"),
    }
}

fn actual_list_metric(theme: &Theme, variant: &str, expressive: bool, role: &str) -> Px {
    match role {
        "container_height" => match variant {
            "one_line" => list::one_line_container_height(theme),
            "two_line" => list::two_line_container_height(theme),
            "three_line" => list::three_line_container_height(theme),
            other => panic!("unsupported list variant {other}"),
        },
        "item_between_space" => list::item_between_space(theme),
        "item_leading_space" => list::item_leading_space(theme),
        "item_trailing_space" => list::item_trailing_space(theme),
        "item_top_space" => list::item_top_space(theme),
        "item_bottom_space" => list::item_bottom_space(theme),
        "leading_icon_size" => list::leading_icon_size_with_variant(theme, expressive),
        "trailing_icon_size" => list::trailing_icon_size_with_variant(theme, expressive),
        other => panic!("unsupported list metric role {other}"),
    }
}

fn actual_list_number(
    theme: &Theme,
    selected: bool,
    interaction: list::ListItemInteraction,
    role: &str,
) -> f32 {
    let (_, _, _, state_layer_opacity) = list::item_outcomes(theme, selected, true, interaction);
    match role {
        "state_layer_opacity" => state_layer_opacity,
        "pressed_state_layer_opacity" => list::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported list number role {other}"),
    }
}

fn actual_list_corners(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: list::ListItemInteraction,
    expressive: bool,
    role: &str,
) -> Corners {
    match role {
        "container_shape" => list::item_container_shape_for_interaction(
            theme,
            selected,
            enabled,
            interaction,
            expressive,
        ),
        other => panic!("unsupported list corners role {other}"),
    }
}

fn actual_list_text_style(theme: &Theme, selected: bool, role: &str) -> TextStyle {
    match role {
        "label_text_style" => list::label_text_style(theme, selected),
        "supporting_text_style" => list::supporting_text_style(theme, selected)
            .map(|style| typography::with_intent(style, TextIntent::Control))
            .unwrap_or_default(),
        "overline_text_style" => list::overline_text_style(theme, selected)
            .map(|style| typography::with_intent(style, TextIntent::Control))
            .unwrap_or_default(),
        "trailing_supporting_text_style" => list::trailing_supporting_text_style(theme, selected)
            .map(|style| typography::with_intent(style, TextIntent::Control))
            .unwrap_or_default(),
        other => panic!("unsupported list text style role {other}"),
    }
}

fn actual_progress_indicator_color(theme: &Theme, role: &str) -> Color {
    match role {
        "track_color" => progress_indicator::track_color(theme),
        "active_color" => progress_indicator::active_color(theme),
        "four_color_1" => progress_indicator::four_color_palette(theme)[0],
        "four_color_2" => progress_indicator::four_color_palette(theme)[1],
        "four_color_3" => progress_indicator::four_color_palette(theme)[2],
        "four_color_4" => progress_indicator::four_color_palette(theme)[3],
        other => panic!("unsupported progress indicator color role {other}"),
    }
}

fn actual_progress_indicator_metric(theme: &Theme, variant: &str, role: &str) -> Px {
    match (variant, role) {
        ("linear", "height") => progress_indicator::linear_height(theme),
        ("linear", "track_thickness") => progress_indicator::linear_track_thickness(theme),
        ("linear", "active_thickness") => progress_indicator::linear_active_thickness(theme),
        ("circular", "size") => progress_indicator::circular_size(theme),
        ("circular", "track_thickness") => progress_indicator::circular_track_thickness(theme),
        ("circular", "active_thickness") => progress_indicator::circular_active_thickness(theme),
        _ => panic!("unsupported progress indicator metric role {variant}:{role}"),
    }
}

fn actual_progress_indicator_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "track_shape" => progress_indicator::track_shape(theme),
        "active_shape" => progress_indicator::active_shape(theme),
        other => panic!("unsupported progress indicator corners role {other}"),
    }
}

fn actual_divider_color(theme: &Theme, role: &str) -> Color {
    match role {
        "color" | "divider_color" => divider::color(theme),
        other => panic!("unsupported divider color role {other}"),
    }
}

fn actual_divider_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "thickness" => divider::thickness(theme),
        other => panic!("unsupported divider metric role {other}"),
    }
}

fn actual_carousel_item_color(
    theme: &Theme,
    with_outline: bool,
    disabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "container_color" => carousel_item::container_background(theme, disabled),
        "container_shadow_color" => carousel_item::container_shadow_color(theme),
        "state_layer_color" => carousel_item::state_layer_color(theme, interaction),
        "outline_color" => {
            carousel_item::outline(theme, with_outline, disabled, interaction)
                .unwrap_or_else(|| panic!("expected carousel item outline"))
                .color
        }
        other => panic!("unsupported carousel item color role {other}"),
    }
}

fn actual_carousel_item_metric(
    theme: &Theme,
    with_outline: bool,
    disabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_elevation" => carousel_item::container_elevation(theme, disabled, interaction),
        "outline_width" => {
            carousel_item::outline(theme, with_outline, disabled, interaction)
                .unwrap_or_else(|| panic!("expected carousel item outline"))
                .width
        }
        other => panic!("unsupported carousel item metric role {other}"),
    }
}

fn actual_carousel_item_number(
    theme: &Theme,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => carousel_item::state_layer_opacity(theme, interaction),
        "pressed_state_layer_opacity" => carousel_item::pressed_state_layer_opacity(theme),
        "disabled_opacity" => carousel_item::disabled_opacity(theme),
        other => panic!("unsupported carousel item number role {other}"),
    }
}

fn actual_carousel_item_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => carousel_item::container_shape(theme),
        other => panic!("unsupported carousel item corners role {other}"),
    }
}

fn enabled_input(case: &Case) -> bool {
    case.input.enabled.unwrap_or(!case.input.disabled)
}

fn button_variant(value: &str, case_id: &str) -> ButtonVariant {
    match value {
        "filled" => ButtonVariant::Filled,
        "tonal" => ButtonVariant::Tonal,
        "elevated" => ButtonVariant::Elevated,
        "outlined" => ButtonVariant::Outlined,
        "text" => ButtonVariant::Text,
        other => panic!("{case_id}: unsupported button variant {other}"),
    }
}

fn button_interaction(value: Option<&str>, case_id: &str) -> Option<ButtonInteraction> {
    match value.unwrap_or("none") {
        "none" => None,
        "hovered" => Some(ButtonInteraction::Hovered),
        "focused" => Some(ButtonInteraction::Focused),
        "pressed" => Some(ButtonInteraction::Pressed),
        other => panic!("{case_id}: unsupported button interaction {other}"),
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

fn pressable_interaction(value: Option<&str>, case_id: &str) -> Option<PressableInteraction> {
    match value.unwrap_or("none") {
        "none" => None,
        "hovered" => Some(PressableInteraction::Hovered),
        "focused" => Some(PressableInteraction::Focused),
        "pressed" => Some(PressableInteraction::Pressed),
        other => panic!("{case_id}: unsupported pressable interaction {other}"),
    }
}

fn checkbox_interaction(value: Option<&str>, case_id: &str) -> checkbox::CheckboxInteraction {
    match value.unwrap_or("none") {
        "none" => checkbox::CheckboxInteraction::None,
        "hovered" => checkbox::CheckboxInteraction::Hovered,
        "focused" => checkbox::CheckboxInteraction::Focused,
        "pressed" => checkbox::CheckboxInteraction::Pressed,
        other => panic!("{case_id}: unsupported checkbox interaction {other}"),
    }
}

fn radio_interaction(value: Option<&str>, case_id: &str) -> radio::RadioInteraction {
    match value.unwrap_or("none") {
        "none" => radio::RadioInteraction::None,
        "hovered" => radio::RadioInteraction::Hovered,
        "focused" => radio::RadioInteraction::Focused,
        "pressed" => radio::RadioInteraction::Pressed,
        other => panic!("{case_id}: unsupported radio interaction {other}"),
    }
}

fn switch_interaction(value: Option<&str>, case_id: &str) -> switch::SwitchInteraction {
    match value.unwrap_or("none") {
        "none" => switch::SwitchInteraction::None,
        "hovered" => switch::SwitchInteraction::Hovered,
        "focused" => switch::SwitchInteraction::Focused,
        "pressed" => switch::SwitchInteraction::Pressed,
        other => panic!("{case_id}: unsupported switch interaction {other}"),
    }
}

fn slider_interaction(value: Option<&str>, case_id: &str) -> slider::SliderInteraction {
    match value.unwrap_or("none") {
        "none" => slider::SliderInteraction::None,
        "hovered" => slider::SliderInteraction::Hovered,
        "focused" => slider::SliderInteraction::Focused,
        "pressed" => slider::SliderInteraction::Pressed,
        other => panic!("{case_id}: unsupported slider interaction {other}"),
    }
}

fn segmented_button_interaction(
    value: Option<&str>,
    case_id: &str,
) -> Option<segmented_button::SegmentedButtonInteraction> {
    match value.unwrap_or("none") {
        "none" => None,
        "hovered" => Some(segmented_button::SegmentedButtonInteraction::Hovered),
        "focused" => Some(segmented_button::SegmentedButtonInteraction::Focused),
        "pressed" => Some(segmented_button::SegmentedButtonInteraction::Pressed),
        other => panic!("{case_id}: unsupported segmented button interaction {other}"),
    }
}

fn icon_button_interaction(
    value: Option<&str>,
    case_id: &str,
) -> Option<icon_button_tokens::IconButtonInteraction> {
    match value.unwrap_or("none") {
        "none" => None,
        "hovered" => Some(icon_button_tokens::IconButtonInteraction::Hovered),
        "focused" => Some(icon_button_tokens::IconButtonInteraction::Focused),
        "pressed" => Some(icon_button_tokens::IconButtonInteraction::Pressed),
        other => panic!("{case_id}: unsupported icon button interaction {other}"),
    }
}

fn icon_button_variant(value: &str, case_id: &str) -> IconButtonVariant {
    match value {
        "standard" => IconButtonVariant::Standard,
        "filled" => IconButtonVariant::Filled,
        "tonal" => IconButtonVariant::Tonal,
        "outlined" => IconButtonVariant::Outlined,
        other => panic!("{case_id}: unsupported icon button variant {other}"),
    }
}

fn tab_interaction(value: Option<&str>, case_id: &str) -> tabs::TabInteraction {
    match value.unwrap_or("none") {
        "none" => tabs::TabInteraction::Default,
        "hovered" => tabs::TabInteraction::Hovered,
        "focused" => tabs::TabInteraction::Focused,
        "pressed" => tabs::TabInteraction::Pressed,
        other => panic!("{case_id}: unsupported tabs interaction {other}"),
    }
}

fn navigation_bar_interaction(
    value: Option<&str>,
    case_id: &str,
) -> navigation_bar::NavigationBarItemInteraction {
    match value.unwrap_or("none") {
        "none" => navigation_bar::NavigationBarItemInteraction::Default,
        "hovered" => navigation_bar::NavigationBarItemInteraction::Hovered,
        "focused" => navigation_bar::NavigationBarItemInteraction::Focused,
        "pressed" => navigation_bar::NavigationBarItemInteraction::Pressed,
        other => panic!("{case_id}: unsupported navigation bar interaction {other}"),
    }
}

fn navigation_rail_interaction(
    value: Option<&str>,
    case_id: &str,
) -> navigation_rail::NavigationRailItemInteraction {
    match value.unwrap_or("none") {
        "none" => navigation_rail::NavigationRailItemInteraction::Default,
        "hovered" => navigation_rail::NavigationRailItemInteraction::Hovered,
        "focused" => navigation_rail::NavigationRailItemInteraction::Focused,
        "pressed" => navigation_rail::NavigationRailItemInteraction::Pressed,
        other => panic!("{case_id}: unsupported navigation rail interaction {other}"),
    }
}

fn navigation_drawer_interaction(
    value: Option<&str>,
    case_id: &str,
) -> navigation_drawer::NavigationDrawerItemInteraction {
    match value.unwrap_or("none") {
        "none" => navigation_drawer::NavigationDrawerItemInteraction::Default,
        "hovered" => navigation_drawer::NavigationDrawerItemInteraction::Hovered,
        "focused" => navigation_drawer::NavigationDrawerItemInteraction::Focused,
        "pressed" => navigation_drawer::NavigationDrawerItemInteraction::Pressed,
        other => panic!("{case_id}: unsupported navigation drawer interaction {other}"),
    }
}

fn navigation_drawer_variant(value: &str, case_id: &str) -> NavigationDrawerVariant {
    match value {
        "standard" => NavigationDrawerVariant::Standard,
        "modal_navigation_drawer" => NavigationDrawerVariant::Modal,
        "modal" | "modal_content" => NavigationDrawerVariant::Modal,
        other => panic!("{case_id}: unsupported navigation drawer variant {other}"),
    }
}

fn top_app_bar_variant(value: &str, case_id: &str) -> TopAppBarVariant {
    match value {
        "small" => TopAppBarVariant::Small,
        "small_centered" | "center_aligned" => TopAppBarVariant::SmallCentered,
        "medium" => TopAppBarVariant::Medium,
        "large" => TopAppBarVariant::Large,
        other => panic!("{case_id}: unsupported top app bar variant {other}"),
    }
}

fn menu_interaction(value: Option<&str>, case_id: &str) -> menu::MenuItemInteraction {
    match value.unwrap_or("none") {
        "none" => menu::MenuItemInteraction::Default,
        "hovered" => menu::MenuItemInteraction::Hovered,
        "focused" => menu::MenuItemInteraction::Focused,
        "pressed" => menu::MenuItemInteraction::Pressed,
        other => panic!("{case_id}: unsupported menu interaction {other}"),
    }
}

fn dialog_action_interaction(
    value: Option<&str>,
    case_id: &str,
) -> dialog::DialogActionInteraction {
    match value.unwrap_or("none") {
        "none" => dialog::DialogActionInteraction::Default,
        "hovered" => dialog::DialogActionInteraction::Hovered,
        "focused" => dialog::DialogActionInteraction::Focused,
        "pressed" => dialog::DialogActionInteraction::Pressed,
        other => panic!("{case_id}: unsupported dialog action interaction {other}"),
    }
}

fn snackbar_interaction(value: Option<&str>, case_id: &str) -> snackbar::SnackbarActionInteraction {
    match value.unwrap_or("none") {
        "none" => snackbar::SnackbarActionInteraction::Default,
        "hovered" => snackbar::SnackbarActionInteraction::Hovered,
        "focused" => snackbar::SnackbarActionInteraction::Focused,
        "pressed" => snackbar::SnackbarActionInteraction::Pressed,
        other => panic!("{case_id}: unsupported snackbar interaction {other}"),
    }
}

fn card_variant(value: &str, case_id: &str) -> CardVariant {
    match value {
        "filled" => CardVariant::Filled,
        "elevated" => CardVariant::Elevated,
        "outlined" => CardVariant::Outlined,
        other => panic!("{case_id}: unsupported card variant {other}"),
    }
}

fn fab_case_variant(value: &str, case_id: &str) -> (bool, FabVariant, FabSize) {
    let mut extended = false;
    let mut variant = FabVariant::Surface;
    let mut size = FabSize::Regular;

    for part in value.split('_') {
        match part {
            "extended" => extended = true,
            "surface" => variant = FabVariant::Surface,
            "primary" => variant = FabVariant::Primary,
            "secondary" => variant = FabVariant::Secondary,
            "tertiary" => variant = FabVariant::Tertiary,
            "regular" => size = FabSize::Regular,
            "small" => size = FabSize::Small,
            "medium" => size = FabSize::Medium,
            "large" => size = FabSize::Large,
            "" => {}
            other => panic!("{case_id}: unsupported fab variant part {other}"),
        }
    }

    (extended, variant, size)
}

fn fab_interaction(value: Option<&str>, case_id: &str) -> Option<fab::FabInteraction> {
    match value.unwrap_or("none") {
        "none" => None,
        "hovered" => Some(fab::FabInteraction::Hovered),
        "focused" => Some(fab::FabInteraction::Focused),
        "pressed" => Some(fab::FabInteraction::Pressed),
        other => panic!("{case_id}: unsupported fab interaction {other}"),
    }
}

fn list_interaction(value: Option<&str>, case_id: &str) -> list::ListItemInteraction {
    match value.unwrap_or("none") {
        "none" => list::ListItemInteraction::Default,
        "hovered" => list::ListItemInteraction::Hovered,
        "focused" => list::ListItemInteraction::Focused,
        "pressed" => list::ListItemInteraction::Pressed,
        other => panic!("{case_id}: unsupported list interaction {other}"),
    }
}

fn require_token<'a>(assertion: &'a Assertion, field: &str) -> &'a str {
    let token = match field {
        "token" => assertion.token.as_deref(),
        "source_token" => assertion.source_token.as_deref(),
        "color_token" => assertion.color_token.as_deref(),
        "opacity_token" => assertion.opacity_token.as_deref(),
        "base_color_token" => assertion.base_color_token.as_deref(),
        "overlay_color_token" => assertion.overlay_color_token.as_deref(),
        other => panic!("unsupported token field {other}"),
    };
    token.unwrap_or_else(|| panic!("{} missing {field}", assertion.role))
}

fn require_value(assertion: &Assertion) -> f32 {
    assertion
        .value
        .unwrap_or_else(|| panic!("{} missing value", assertion.role))
}

fn token_color(theme: &Theme, key: &str) -> Color {
    theme
        .color_by_key(key)
        .unwrap_or_else(|| panic!("expected color token {key}"))
}

fn token_metric(theme: &Theme, key: &str) -> Px {
    theme
        .metric_by_key(key)
        .unwrap_or_else(|| panic!("expected metric token {key}"))
}

fn token_number(theme: &Theme, key: &str) -> f32 {
    theme
        .number_by_key(key)
        .unwrap_or_else(|| panic!("expected number token {key}"))
}

fn token_corners(theme: &Theme, key: &str) -> Corners {
    theme
        .corners_by_key(key)
        .unwrap_or_else(|| panic!("expected corners token {key}"))
}

fn token_text_style(theme: &Theme, key: &str) -> TextStyle {
    theme
        .text_style_by_key(key)
        .unwrap_or_else(|| panic!("expected text style token {key}"))
}

fn color_with_alpha(theme: &Theme, color_token: &str, opacity_token: &str) -> Color {
    let mut color = token_color(theme, color_token);
    color.a = (color.a * token_number(theme, opacity_token)).clamp(0.0, 1.0);
    color
}

fn alpha_color(mut color: Color, opacity: f32) -> Color {
    color.a = (color.a * opacity).clamp(0.0, 1.0);
    color
}

fn control_text_style(theme: &Theme, key: &str) -> TextStyle {
    typography::with_intent(token_text_style(theme, key), TextIntent::Control)
}

fn control_text_style_with_weight(theme: &Theme, source_key: &str, weight_key: &str) -> TextStyle {
    text_style_with_weight(theme, source_key, weight_key, TextIntent::Control)
}

fn content_text_style_with_weight(theme: &Theme, source_key: &str, weight_key: &str) -> TextStyle {
    text_style_with_weight(theme, source_key, weight_key, TextIntent::Content)
}

fn text_style_with_weight(
    theme: &Theme,
    source_key: &str,
    weight_key: &str,
    intent: TextIntent,
) -> TextStyle {
    let _ = token_number(theme, weight_key);
    token_typography::text_style_with_weight(theme, None, source_key, Some(weight_key), intent)
}

fn text_intent_for_role(role: &str) -> TextIntent {
    if role.contains("action") || role.contains("label") {
        TextIntent::Control
    } else {
        TextIntent::Content
    }
}

fn content_text_style(theme: &Theme, key: &str) -> TextStyle {
    typography::with_intent(token_text_style(theme, key), TextIntent::Content)
}

fn assert_text_style_alias(theme: &Theme, case: &Case, assertion: &Assertion) {
    assert_eq!(
        token_text_style(theme, require_token(assertion, "token")),
        token_text_style(theme, require_token(assertion, "source_token")),
        "{}:{} text style alias mismatch",
        case.id,
        assertion.role
    );
}

fn assert_text_style_eq(case_id: &str, role: &str, actual: TextStyle, expected: TextStyle) {
    assert_eq!(
        actual, expected,
        "{case_id}:{role} text style mismatch: actual={actual:?} expected={expected:?}"
    );
}

fn assert_color_close(case_id: &str, role: &str, actual: Color, expected: Color) {
    assert!(
        close(actual.r, expected.r)
            && close(actual.g, expected.g)
            && close(actual.b, expected.b)
            && close(actual.a, expected.a),
        "{case_id}:{role} color mismatch: actual={actual:?} expected={expected:?}"
    );
}

fn assert_px_eq(case_id: &str, role: &str, actual: Px, expected: Px) {
    assert!(
        close(actual.0, expected.0),
        "{case_id}:{role} px mismatch: actual={actual:?} expected={expected:?}"
    );
}

fn assert_number_close(case_id: &str, role: &str, actual: f32, expected: f32) {
    assert!(
        close(actual, expected),
        "{case_id}:{role} number mismatch: actual={actual} expected={expected}"
    );
}

fn assert_corners_eq(case_id: &str, role: &str, actual: Corners, expected: Corners) {
    assert_px_eq(
        case_id,
        &format!("{role}.top_left"),
        actual.top_left,
        expected.top_left,
    );
    assert_px_eq(
        case_id,
        &format!("{role}.top_right"),
        actual.top_right,
        expected.top_right,
    );
    assert_px_eq(
        case_id,
        &format!("{role}.bottom_right"),
        actual.bottom_right,
        expected.bottom_right,
    );
    assert_px_eq(
        case_id,
        &format!("{role}.bottom_left"),
        actual.bottom_left,
        expected.bottom_left,
    );
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() <= 0.0001
}
