//! Material Design 3 (and Expressive) component surface for Fret.
//!
//! This crate targets **visual + interaction outcome alignment** with the Material 3 design
//! system, while keeping `crates/fret-ui` focused on mechanisms (not Material-specific policy).

#![forbid(unsafe_code)]

pub mod autocomplete;
pub mod badge;
pub mod bottom_sheet;
pub mod button;
pub mod card;
pub mod carousel_item;
pub mod checkbox;
pub mod chip;
pub mod chip_set;
pub mod date_picker;
pub mod dialog;
pub mod divider;
pub mod dropdown_menu;
pub mod exposed_dropdown;
pub mod fab;
pub mod filter_chip;
mod foundation;
pub mod icon_button;
pub mod input_chip;
pub mod interaction;
pub mod list;
pub mod menu;
pub mod modal_navigation_drawer;
pub mod motion;
pub mod navigation_bar;
pub mod navigation_drawer;
pub mod navigation_rail;
pub mod progress_indicator;
pub mod radio;
pub mod search_bar;
pub mod search_view;
pub mod segmented_button;
pub mod select;
pub mod slider;
pub mod snackbar;
pub mod suggestion_chip;
pub mod switch;
pub mod tabs;
pub mod text_field;
pub mod theme;
pub mod time_picker;
pub mod tokens;
pub mod tooltip;
pub mod top_app_bar;

pub use autocomplete::{
    Autocomplete, AutocompleteItem, AutocompleteSelectCx, AutocompleteSelectMethod,
    AutocompleteVariant, OnAutocompleteSelect,
};
pub use badge::{Badge, BadgePlacement, BadgeValue};
pub use bottom_sheet::{DockedBottomSheet, DockedBottomSheetVariant, ModalBottomSheet};
pub use button::{Button, ButtonSize, ButtonStyle, ButtonVariant};
pub use card::{Card, CardStyle, CardVariant};
pub use carousel_item::{CarouselItem, CarouselItemStyle, CarouselItemVariant};
pub use checkbox::{Checkbox, CheckboxStyle};
pub use chip::{AssistChip, AssistChipStyle, AssistChipVariant};
pub use chip_set::{ChipSet, ChipSetItem};
pub use context::{MaterialDesignVariant, with_material_design_variant};
pub use date_picker::{DatePickerDialog, DatePickerVariant, DockedDatePicker};
pub use dialog::{Dialog, DialogAction, DialogStyle};
pub use divider::Divider;
pub use dropdown_menu::{DropdownMenu, DropdownMenuAlign, DropdownMenuSide};
pub use exposed_dropdown::ExposedDropdown;
pub use fab::{Fab, FabSize, FabStyle, FabVariant};
pub use filter_chip::{FilterChip, FilterChipStyle, FilterChipVariant};
pub use icon_button::{
    IconButton, IconButtonSize, IconButtonStyle, IconButtonVariant, IconToggleButton,
    IconToggleButtonShapes,
};
pub use input_chip::{InputChip, InputChipStyle};
pub use list::{List, ListItem};
pub use menu::{Menu, MenuEntry, MenuItem, MenuStyle};
pub use modal_navigation_drawer::ModalNavigationDrawer;
pub use navigation_bar::{NavigationBar, NavigationBarItem};
pub use navigation_drawer::{NavigationDrawer, NavigationDrawerItem, NavigationDrawerVariant};
pub use navigation_rail::{NavigationRail, NavigationRailItem};
pub use progress_indicator::{CircularProgressIndicator, LinearProgressIndicator};
pub use radio::{Radio, RadioGroup, RadioGroupItem, RadioGroupOrientation, RadioStyle};
pub use search_bar::SearchBar;
pub use search_view::SearchView;
pub use segmented_button::{SegmentedButtonItem, SegmentedButtonSet};
pub use select::{Select, SelectItem, SelectStyle, SelectVariant};
pub use slider::{RangeSlider, Slider, SliderStyle};
pub use snackbar::{Snackbar, SnackbarController, SnackbarDuration, SnackbarHost};
pub use suggestion_chip::{SuggestionChip, SuggestionChipStyle, SuggestionChipVariant};
pub use switch::{Switch, SwitchStyle};
pub use tabs::{TabItem, Tabs, TabsStyle};
pub use text_field::{TextField, TextFieldStyle, TextFieldVariant};
pub use time_picker::{
    DockedTimePicker, TimePickerDialog, TimePickerDisplayMode, TimePickerVariant,
};
pub use tooltip::{PlainTooltip, RichTooltip, TooltipAlign, TooltipProvider, TooltipSide};
pub use top_app_bar::{TopAppBar, TopAppBarAction, TopAppBarScrollBehavior, TopAppBarVariant};

pub mod context {
    //! Tree-local overrides for Material 3 rendering.
    //!
    //! This is a small, ecosystem-owned analogue to Compose's composition locals (content color,
    //! ripple configuration, motion scheme, etc.).

    pub use crate::foundation::context::{
        MaterialContentColor, MaterialDesignVariant, MaterialDesignVariantOverride,
        MaterialMotionScheme, MaterialMotionSchemeOverride, MaterialRippleConfiguration,
        inherited_content_color, inherited_content_color_policy, inherited_design_variant_override,
        inherited_motion_scheme_override, inherited_ripple_configuration, resolved_design_variant,
        resolved_motion_scheme, theme_default_design_variant, with_default_material_content_color,
        with_default_material_design_variant, with_default_material_motion_scheme,
        with_material_content_color, with_material_content_color_policy,
        with_material_design_variant, with_material_design_variant_override,
        with_material_motion_scheme, with_material_motion_scheme_override,
        with_material_ripple_configuration,
    };
}

#[doc(hidden)]
pub mod __testing {
    use fret_core::TextStyle;
    use fret_ui::Theme;

    pub fn search_bar_input_text_style(theme: &Theme) -> TextStyle {
        crate::tokens::search_bar::input_text_style(theme)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use fret_app::App;
    use fret_core::{TextLineHeightPolicy, TextVerticalPlacement};
    use fret_ui::Theme;

    use crate::tokens::v30::{ColorSchemeOptions, TypographyOptions, theme_config_with_colors};

    const README: &str = include_str!("../README.md");

    fn normalize_ws(source: &str) -> String {
        source.split_whitespace().collect()
    }

    fn assert_material_only_tokens(source: &str) {
        let forbidden_literals = [
            "color_required(\"card\")",
            "color_required(\"foreground\")",
            "color_required(\"muted-foreground\")",
            "color_required(\"border\")",
            "color_required(\"background\")",
            "color_required(\"color.accent\")",
            "color_required(\"color.text.primary\")",
            "color_required(\"color.text.disabled\")",
            "color_required(\"color.border\")",
            "color_by_key(\"card\")",
            "color_by_key(\"foreground\")",
            "color_by_key(\"muted-foreground\")",
            "color_by_key(\"border\")",
            "color_by_key(\"background\")",
            "color_by_key(\"color.accent\")",
            "color_by_key(\"color.text.primary\")",
            "color_by_key(\"color.text.disabled\")",
            "color_by_key(\"color.border\")",
        ];

        for lit in forbidden_literals {
            assert!(
                !source.contains(lit),
                "forbidden non-Material theme token reference: {lit}"
            );
        }
    }

    #[test]
    fn material3_component_sources_do_not_fallback_to_non_material_tokens() {
        let sources = [
            include_str!("button.rs"),
            include_str!("badge.rs"),
            include_str!("bottom_sheet.rs"),
            include_str!("card.rs"),
            include_str!("carousel_item.rs"),
            include_str!("checkbox.rs"),
            include_str!("chip.rs"),
            include_str!("date_picker.rs"),
            include_str!("time_picker.rs"),
            include_str!("divider.rs"),
            include_str!("dialog.rs"),
            include_str!("dropdown_menu.rs"),
            include_str!("fab.rs"),
            include_str!("icon_button.rs"),
            include_str!("list.rs"),
            include_str!("menu.rs"),
            include_str!("modal_navigation_drawer.rs"),
            include_str!("navigation_bar.rs"),
            include_str!("navigation_drawer.rs"),
            include_str!("navigation_rail.rs"),
            include_str!("progress_indicator.rs"),
            include_str!("radio.rs"),
            include_str!("search_bar.rs"),
            include_str!("search_view.rs"),
            include_str!("segmented_button.rs"),
            include_str!("slider.rs"),
            include_str!("select.rs"),
            include_str!("snackbar.rs"),
            include_str!("switch.rs"),
            include_str!("tabs.rs"),
            include_str!("text_field.rs"),
            include_str!("top_app_bar.rs"),
            include_str!("tooltip.rs"),
            include_str!("foundation/indication.rs"),
            include_str!("foundation/focus_ring.rs"),
            include_str!("foundation/geometry.rs"),
            include_str!("foundation/interaction.rs"),
            include_str!("foundation/tokens.rs"),
            include_str!("tokens/icon_button.rs"),
            include_str!("tokens/badge.rs"),
            include_str!("tokens/button.rs"),
            include_str!("tokens/card.rs"),
            include_str!("tokens/checkbox.rs"),
            include_str!("tokens/chip.rs"),
            include_str!("tokens/date_picker.rs"),
            include_str!("tokens/time_picker.rs"),
            include_str!("tokens/progress_indicator.rs"),
            include_str!("tokens/divider.rs"),
            include_str!("tokens/fab.rs"),
            include_str!("tokens/switch.rs"),
            include_str!("tokens/radio.rs"),
            include_str!("tokens/dialog.rs"),
            include_str!("tokens/snackbar.rs"),
            include_str!("tokens/tabs.rs"),
            include_str!("tokens/menu.rs"),
            include_str!("tokens/text_field.rs"),
            include_str!("tokens/list.rs"),
            include_str!("tokens/dropdown_menu.rs"),
            include_str!("tokens/segmented_button.rs"),
            include_str!("tokens/sheet_bottom.rs"),
            include_str!("tokens/search_bar.rs"),
            include_str!("tokens/search_view.rs"),
            include_str!("tokens/tooltip.rs"),
            include_str!("tokens/slider.rs"),
            include_str!("tokens/top_app_bar.rs"),
        ];

        for src in sources {
            assert_material_only_tokens(src);
        }
    }

    #[test]
    fn default_facing_material3_pressables_expose_action_first_aliases() {
        let action_sources = [
            include_str!("button.rs"),
            include_str!("fab.rs"),
            include_str!("card.rs"),
            include_str!("dialog.rs"),
            include_str!("icon_button.rs"),
            include_str!("checkbox.rs"),
            include_str!("switch.rs"),
            include_str!("radio.rs"),
            include_str!("chip.rs"),
            include_str!("suggestion_chip.rs"),
            include_str!("filter_chip.rs"),
            include_str!("input_chip.rs"),
            include_str!("top_app_bar.rs"),
        ];

        for src in action_sources {
            assert!(
                src.contains("pub fn action("),
                "expected default-facing Material3 widget to expose `action(...)`"
            );
        }
    }

    #[test]
    fn material3_chip_trailing_pressables_expose_action_first_aliases() {
        let trailing_action_sources = [
            include_str!("filter_chip.rs"),
            include_str!("input_chip.rs"),
        ];

        for src in trailing_action_sources {
            assert!(
                src.contains("pub fn trailing_action("),
                "expected trailing chip pressable to expose `trailing_action(...)`"
            );
        }
    }

    #[test]
    fn readme_keeps_icon_provider_installation_explicit_for_material3() {
        assert!(README.contains("semantic `IconId` / `ui.*` ids"));
        assert!(README.contains("This crate does not install a default icon provider for you."));
        assert!(README.contains("`fret_icons_lucide::app::install`"));
        assert!(README.contains("`fret_icons_radix::app::install`"));
        assert!(README.contains("use fret_icons::ids;"));
        assert!(README.contains("fret_icons_lucide::app::install(app);"));
        assert!(README.contains("fret_ui_material3::Button::new(\"Search\")"));
        assert!(README.contains(".leading_icon(ids::ui::SEARCH);"));
    }

    #[test]
    fn material3_control_typography_tokens_use_stable_line_boxes() {
        let cfg =
            theme_config_with_colors(TypographyOptions::default(), ColorSchemeOptions::default());
        let mut app = App::default();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));
        let theme = Theme::global(&app).clone();

        let control_styles = [
            crate::tokens::search_bar::input_text_style(&theme),
            crate::tokens::search_view::header_input_text_style(&theme),
            crate::tokens::slider::value_indicator_label_style(&theme),
            crate::tokens::time_input::time_input_field_label_text_style(&theme),
            crate::tokens::time_input::time_input_field_separator_style(&theme),
            crate::tokens::time_input::period_selector_label_text_style(&theme),
            crate::tokens::time_picker::headline_style(&theme),
            crate::tokens::time_picker::clock_dial_label_text_style(&theme),
            crate::tokens::time_picker::time_selector_label_text_style(&theme),
            crate::tokens::time_picker::time_selector_separator_style(&theme),
            crate::tokens::time_picker::period_selector_label_text_style(&theme),
            crate::tokens::date_picker::weekdays_label_text_style(
                &theme,
                crate::tokens::date_picker::DatePickerTokenVariant::Docked,
            ),
            crate::tokens::date_picker::date_label_text_style(
                &theme,
                crate::tokens::date_picker::DatePickerTokenVariant::Docked,
            ),
            crate::tokens::date_picker::header_headline_style(&theme),
        ];

        for style in control_styles {
            assert_eq!(
                style.line_height_policy,
                TextLineHeightPolicy::FixedFromStyle
            );
            assert_eq!(
                style.vertical_placement,
                TextVerticalPlacement::BoundsAsLineBox
            );
        }

        let supporting = crate::tokens::time_input::time_input_field_supporting_text_style(&theme);
        assert_eq!(
            supporting.line_height_policy,
            TextLineHeightPolicy::ExpandToFit
        );
        assert_eq!(
            supporting.vertical_placement,
            TextVerticalPlacement::CenterMetricsBox
        );
    }

    fn extract_md_literal_keys(source: &str) -> HashSet<&str> {
        let mut out = HashSet::new();
        let mut cursor: usize = 0;
        while let Some(idx) = source[cursor..].find("\"md.") {
            let start = cursor + idx + 1;
            let rest = &source[start..];
            let Some(end) = rest.find('"') else {
                break;
            };
            let key = &source[start..start + end];
            cursor = start + end + 1;
            if key.contains('{') || key.contains('}') || key.contains(' ') || key.contains('\n') {
                continue;
            }
            // Skip namespace/prefix strings like `md.comp.button` / `md.comp.checkbox.selected`
            // that are used to build other keys.
            // - `md.sys.*` tokens can be as short as `md.sys.color.primary` (3 dots).
            // - `md.comp.*` tokens are always deeper (at least 4 dots).
            let dot_count = key.matches('.').count();
            if key.starts_with("md.comp.") {
                if dot_count < 4 {
                    continue;
                }
            } else if dot_count < 3 {
                continue;
            }
            out.insert(key);
        }
        out
    }

    fn token_resolves(theme: &Theme, key: &str) -> bool {
        theme.color_by_key(key).is_some()
            || theme.metric_by_key(key).is_some()
            || theme.number_by_key(key).is_some()
            || theme.duration_ms_by_key(key).is_some()
            || theme.easing_by_key(key).is_some()
            || theme.corners_by_key(key).is_some()
            || theme.text_style_by_key(key).is_some()
    }

    #[test]
    fn material3_literal_md_tokens_resolve_in_v30_theme() {
        let cfg =
            theme_config_with_colors(TypographyOptions::default(), ColorSchemeOptions::default());

        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));
        let theme = Theme::global(&app);

        let sources = [
            include_str!("button.rs"),
            include_str!("card.rs"),
            include_str!("checkbox.rs"),
            include_str!("chip.rs"),
            include_str!("date_picker.rs"),
            include_str!("dialog.rs"),
            include_str!("dropdown_menu.rs"),
            include_str!("icon_button.rs"),
            include_str!("list.rs"),
            include_str!("menu.rs"),
            include_str!("modal_navigation_drawer.rs"),
            include_str!("navigation_bar.rs"),
            include_str!("navigation_drawer.rs"),
            include_str!("navigation_rail.rs"),
            include_str!("slider.rs"),
            include_str!("radio.rs"),
            include_str!("search_bar.rs"),
            include_str!("search_view.rs"),
            include_str!("select.rs"),
            include_str!("snackbar.rs"),
            include_str!("switch.rs"),
            include_str!("tabs.rs"),
            include_str!("text_field.rs"),
            include_str!("tooltip.rs"),
            include_str!("foundation/indication.rs"),
            include_str!("foundation/focus_ring.rs"),
            include_str!("foundation/geometry.rs"),
            include_str!("foundation/interaction.rs"),
            include_str!("foundation/tokens.rs"),
            include_str!("tokens/icon_button.rs"),
            include_str!("tokens/button.rs"),
            include_str!("tokens/card.rs"),
            include_str!("tokens/checkbox.rs"),
            include_str!("tokens/chip.rs"),
            include_str!("tokens/date_picker.rs"),
            include_str!("tokens/switch.rs"),
            include_str!("tokens/radio.rs"),
            include_str!("tokens/dialog.rs"),
            include_str!("tokens/snackbar.rs"),
            include_str!("tokens/tabs.rs"),
            include_str!("tokens/menu.rs"),
            include_str!("tokens/text_field.rs"),
            include_str!("tokens/list.rs"),
            include_str!("tokens/dropdown_menu.rs"),
            include_str!("tokens/select.rs"),
            include_str!("tokens/search_bar.rs"),
            include_str!("tokens/search_view.rs"),
            include_str!("tokens/tooltip.rs"),
            include_str!("tokens/slider.rs"),
        ];

        let mut keys: Vec<&str> = sources
            .iter()
            .flat_map(|src| extract_md_literal_keys(src))
            .collect();
        keys.sort_unstable();
        keys.dedup();

        for key in keys {
            assert!(
                token_resolves(theme, key),
                "md token not found in v30 theme config: {key}"
            );
        }
    }

    fn assert_minimum_touch_target_policy(file: &str, src: &str) {
        assert!(
            src.contains("enforce_minimum_interactive_size"),
            "{file}: missing minimum touch target policy enforcement"
        );

        let pointer_region_start = src
            .find("PointerRegionProps::default()")
            .unwrap_or_else(|| panic!("{file}: missing PointerRegionProps usage"));

        let window_end = (pointer_region_start + 800).min(src.len());
        let window = &src[pointer_region_start..window_end];

        assert!(
            window.contains("props.layout.size.width = Length::Fill"),
            "{file}: missing PointerRegion fill width"
        );
        assert!(
            window.contains("props.layout.size.height = Length::Fill"),
            "{file}: missing PointerRegion fill height"
        );
    }

    #[test]
    fn material3_components_apply_minimum_touch_target_policy() {
        let sources = [
            ("tabs.rs", include_str!("tabs.rs")),
            ("navigation_bar.rs", include_str!("navigation_bar.rs")),
            ("navigation_rail.rs", include_str!("navigation_rail.rs")),
            ("navigation_drawer.rs", include_str!("navigation_drawer.rs")),
            ("menu.rs", include_str!("menu.rs")),
            ("list.rs", include_str!("list.rs")),
        ];

        for (file, src) in sources {
            assert_minimum_touch_target_policy(file, src);
        }
    }

    #[test]
    fn material3_overlay_roots_offer_uncontrolled_copyable_paths_while_keeping_controlled_new() {
        let sources = [
            ("dropdown_menu.rs", include_str!("dropdown_menu.rs")),
            ("dialog.rs", include_str!("dialog.rs")),
            ("bottom_sheet.rs", include_str!("bottom_sheet.rs")),
            (
                "modal_navigation_drawer.rs",
                include_str!("modal_navigation_drawer.rs"),
            ),
        ];

        let required_markers = [
            "pub fn new(open: Model<bool>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {",
            "pub fn open_model(&self) -> Model<bool> {",
        ];

        for (file, src) in sources {
            let normalized = normalize_ws(src);

            for marker in required_markers {
                let marker = normalize_ws(marker);
                assert!(
                    normalized.contains(&marker),
                    "{file} should keep `new(open)` as the explicit controlled seam and expose `new_controllable(...)`, `uncontrolled(cx)`, and `open_model()` for copyable teaching surfaces"
                );
            }
        }
    }

    #[test]
    fn material3_search_view_offers_uncontrolled_copyable_path_while_keeping_controlled_new() {
        let normalized = normalize_ws(include_str!("search_view.rs"));
        let required_markers = [
            "pub fn new(open: Model<bool>, query: Model<String>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, query: Option<Model<String>>, default_query: impl Into<String>, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {",
            "pub fn open_model(&self) -> Model<bool> {",
            "pub fn query_model(&self) -> Model<String> {",
        ];

        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "search_view.rs should keep `new(open, query)` as the explicit controlled seam and expose `new_controllable(...)`, `uncontrolled(cx)`, `open_model()`, and `query_model()` for copyable teaching surfaces"
            );
        }
    }

    #[test]
    fn material3_autocomplete_offers_uncontrolled_copyable_query_path_while_keeping_controlled_new()
    {
        let normalized = normalize_ws(include_str!("autocomplete.rs"));
        let required_markers = [
            "pub fn new(query: Model<String>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, query: Option<Model<String>>, default_query: impl Into<String>, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {",
            "pub fn query_model(&self) -> Model<String> {",
        ];

        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "autocomplete.rs should keep `new(query)` as the explicit controlled seam and expose `new_controllable(...)`, `uncontrolled(cx)`, and `query_model()` for copyable teaching surfaces"
            );
        }
    }

    #[test]
    fn material3_exposed_dropdown_offers_uncontrolled_copyable_selected_and_query_paths() {
        let normalized = normalize_ws(include_str!("exposed_dropdown.rs"));
        let required_markers = [
            "pub fn new(selected_value: Model<Option<Arc<str>>>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, selected_value: Option<Model<Option<Arc<str>>>>, default_selected_value: Option<Arc<str>>, query: Option<Model<String>>, default_query: impl Into<String>, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {",
            "pub fn selected_value_model(&self) -> Model<Option<Arc<str>>> {",
            "pub fn query_model(&self) -> Model<String> {",
            "pub fn query(mut self, query: Model<String>) -> Self {",
        ];

        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "exposed_dropdown.rs should keep `new(selected_value)` + `.query(...)` as the explicit controlled seams and expose `new_controllable(...)`, `uncontrolled(cx)`, `selected_value_model()`, and `query_model()` for copyable teaching surfaces"
            );
        }
    }

    #[test]
    fn material3_select_offers_uncontrolled_copyable_value_path_while_keeping_controlled_new() {
        let normalized = normalize_ws(include_str!("select.rs"));
        let required_markers = [
            "pub fn new(selected_value: Model<Option<Arc<str>>>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, selected_value: Option<Model<Option<Arc<str>>>>, default_selected_value: Option<Arc<str>>, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {",
            "pub fn value_model(&self) -> Model<Option<Arc<str>>> {",
        ];

        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "select.rs should keep `new(selected_value)` as the explicit controlled seam and expose `new_controllable(...)`, `uncontrolled(cx)`, and `value_model()` for copyable teaching surfaces"
            );
        }
    }

    #[test]
    fn material3_selection_roots_offer_uncontrolled_copyable_value_paths_while_keeping_controlled_new()
     {
        let sources = [
            ("tabs.rs", include_str!("tabs.rs")),
            ("navigation_bar.rs", include_str!("navigation_bar.rs")),
            ("navigation_rail.rs", include_str!("navigation_rail.rs")),
            ("navigation_drawer.rs", include_str!("navigation_drawer.rs")),
            ("list.rs", include_str!("list.rs")),
        ];

        let required_markers = [
            "pub fn new(model: Model<Arc<str>>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, value: Option<Model<Arc<str>>>, default_value: impl Into<Arc<str>>, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>( cx: &mut ElementContext<'_, H>, default_value: impl Into<Arc<str>>, ) -> Self {",
            "pub fn value_model(&self) -> Model<Arc<str>> {",
        ];

        for (file, src) in sources {
            let normalized = normalize_ws(src);

            for marker in required_markers {
                let marker = normalize_ws(marker);
                assert!(
                    normalized.contains(&marker),
                    "{file} should keep `new(model)` as the explicit controlled seam and expose `new_controllable(...)`, `uncontrolled(cx, default)`, and `value_model()` for copyable teaching surfaces"
                );
            }
        }
    }

    #[test]
    fn material3_text_field_offers_uncontrolled_copyable_value_path_while_keeping_controlled_new() {
        let normalized = normalize_ws(include_str!("text_field.rs"));
        let required_markers = [
            "pub fn new(model: Model<String>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, value: Option<Model<String>>, default_value: impl Into<String>, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {",
            "pub fn value_model(&self) -> Model<String> {",
        ];

        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "text_field.rs should keep `new(model)` as the explicit controlled seam and expose `new_controllable(...)`, `uncontrolled(cx)`, and `value_model()` for copyable teaching surfaces"
            );
        }
    }

    #[test]
    fn material3_checkbox_offers_uncontrolled_copyable_checked_paths() {
        let normalized = normalize_ws(include_str!("checkbox.rs"));
        let required_markers = [
            "pub fn new(checked: Model<bool>) -> Self {",
            "pub fn new_optional(checked: Model<Option<bool>>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, checked: Option<Model<bool>>, default_checked: bool, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>, default_checked: bool) -> Self {",
            "pub fn checked_model(&self) -> Model<bool> {",
            "pub fn new_optional_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, checked: Option<Model<Option<bool>>>, default_checked: Option<bool>, ) -> Self {",
            "pub fn uncontrolled_optional<H: UiHost>( cx: &mut ElementContext<'_, H>, default_checked: Option<bool>, ) -> Self {",
            "pub fn optional_checked_model(&self) -> Model<Option<bool>> {",
        ];

        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "checkbox.rs should keep `new(checked)` / `new_optional(checked)` as explicit controlled seams and expose controllable/uncontrolled helpers plus model accessors for copyable teaching surfaces"
            );
        }
    }

    #[test]
    fn material3_switch_offers_uncontrolled_copyable_selected_path_while_keeping_controlled_new() {
        let normalized = normalize_ws(include_str!("switch.rs"));
        let required_markers = [
            "pub fn new(selected: Model<bool>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, selected: Option<Model<bool>>, default_selected: bool, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>, default_selected: bool) -> Self {",
            "pub fn selected_model(&self) -> Model<bool> {",
        ];

        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "switch.rs should keep `new(selected)` as the explicit controlled seam and expose `new_controllable(...)`, `uncontrolled(cx, default)`, and `selected_model()` for copyable teaching surfaces"
            );
        }
    }

    #[test]
    fn material3_radio_offers_uncontrolled_copyable_group_and_standalone_paths() {
        let normalized = normalize_ws(include_str!("radio.rs"));
        let required_markers = [
            "pub fn new(model: Model<Option<Arc<str>>>) -> Self {",
            "pub fn new_controllable<H: UiHost, T: Into<Arc<str>>>( cx: &mut ElementContext<'_, H>, value: Option<Model<Option<Arc<str>>>>, default_value: Option<T>, ) -> Self {",
            "pub fn uncontrolled<H: UiHost, T: Into<Arc<str>>>( cx: &mut ElementContext<'_, H>, default_value: Option<T>, ) -> Self {",
            "pub fn value_model(&self) -> Model<Option<Arc<str>>> {",
            "pub fn new(selected: Model<bool>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, selected: Option<Model<bool>>, default_selected: bool, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>, default_selected: bool) -> Self {",
            "pub fn selected_model(&self) -> Model<bool> {",
        ];

        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "radio.rs should keep controlled `RadioGroup::new(model)` / `Radio::new(selected)` seams and expose controllable/uncontrolled helpers plus model accessors for copyable teaching surfaces"
            );
        }
    }

    #[test]
    fn material3_slider_offers_uncontrolled_copyable_value_paths() {
        let normalized = normalize_ws(include_str!("slider.rs"));
        let required_markers = [
            "pub fn new(value: Model<f32>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, value: Option<Model<f32>>, default_value: f32, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>, default_value: f32) -> Self {",
            "pub fn value_model(&self) -> Model<f32> {",
            "pub fn new(values: Model<[f32; 2]>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, values: Option<Model<[f32; 2]>>, default_values: [f32; 2], ) -> Self {",
            "pub fn uncontrolled<H: UiHost>( cx: &mut ElementContext<'_, H>, default_values: [f32; 2], ) -> Self {",
            "pub fn values_model(&self) -> Model<[f32; 2]> {",
        ];

        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "slider.rs should keep controlled `Slider::new(value)` / `RangeSlider::new(values)` seams and expose controllable/uncontrolled helpers plus model accessors for copyable teaching surfaces"
            );
        }
    }

    #[test]
    fn material3_date_picker_dialog_offers_uncontrolled_copyable_open_month_and_selected_paths() {
        let normalized = normalize_ws(include_str!("date_picker.rs"));
        let required_markers = [
            "pub fn new( open: Model<bool>, month: Model<CalendarMonth>, selected: Model<Option<Date>>, ) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, month: Option<Model<CalendarMonth>>, default_month: CalendarMonth, selected: Option<Model<Option<Date>>>, default_selected: Option<Date>, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {",
            "pub fn open_model(&self) -> Model<bool> {",
            "pub fn month_model(&self) -> Model<CalendarMonth> {",
            "pub fn selected_model(&self) -> Model<Option<Date>> {",
        ];

        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "date_picker.rs should keep `new(open, month, selected)` as the explicit controlled seam and expose `new_controllable(...)`, `uncontrolled(cx)`, `open_model()`, `month_model()`, and `selected_model()` for copyable teaching surfaces"
            );
        }
    }

    #[test]
    fn material3_time_picker_dialog_offers_uncontrolled_copyable_open_and_selected_paths() {
        let normalized = normalize_ws(include_str!("time_picker.rs"));
        let required_markers = [
            "pub fn new(open: Model<bool>, selected: Model<Time>) -> Self {",
            "pub fn new_controllable<H: UiHost>( cx: &mut ElementContext<'_, H>, open: Option<Model<bool>>, default_open: bool, selected: Option<Model<Time>>, default_selected: Time, ) -> Self {",
            "pub fn uncontrolled<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Self {",
            "pub fn open_model(&self) -> Model<bool> {",
            "pub fn selected_model(&self) -> Model<Time> {",
        ];

        for marker in required_markers {
            let marker = normalize_ws(marker);
            assert!(
                normalized.contains(&marker),
                "time_picker.rs should keep `new(open, selected)` as the explicit controlled seam and expose `new_controllable(...)`, `uncontrolled(cx)`, `open_model()`, and `selected_model()` for copyable teaching surfaces"
            );
        }
    }
}
