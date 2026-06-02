use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::{self, TextIntent};

use crate::button::{ButtonSize, ButtonVariant};
use crate::card::CardVariant;
use crate::fab::{FabSize, FabVariant};
use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::blend_over;
use crate::icon_button::{IconButtonSize, IconButtonVariant};
use crate::tokens::{
    badge, button, card, carousel_item, checkbox, chip, dialog, divider, dropdown_menu, fab,
    filter_chip, icon_button as icon_button_tokens, input_chip, list, menu, progress_indicator,
    radio, segmented_button, sheet_bottom, slider, snackbar, suggestion_chip, switch, tooltip,
    typography as token_typography,
};

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

fn actual_button_text_style(theme: &Theme, size: ButtonSize, role: &str) -> TextStyle {
    match role {
        "label_text_style" => button::label_text_style(theme, size),
        other => panic!("unsupported button text style role {other}"),
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

fn actual_segmented_button_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "label_text_style" => segmented_button::label_text_style(theme),
        other => panic!("unsupported segmented button text style role {other}"),
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
