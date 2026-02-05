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
pub use button::{Button, ButtonStyle, ButtonVariant};
pub use card::{Card, CardStyle, CardVariant};
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
pub use icon_button::{IconButton, IconButtonSize, IconButtonStyle, IconButtonVariant};
pub use input_chip::{InputChip, InputChipStyle};
pub use list::{List, ListItem};
pub use menu::{Menu, MenuEntry, MenuItem, MenuStyle};
pub use modal_navigation_drawer::ModalNavigationDrawer;
pub use navigation_bar::{NavigationBar, NavigationBarItem};
pub use navigation_drawer::{NavigationDrawer, NavigationDrawerItem, NavigationDrawerVariant};
pub use navigation_rail::{NavigationRail, NavigationRailItem};
pub use progress_indicator::{CircularProgressIndicator, LinearProgressIndicator};
pub use radio::{Radio, RadioGroup, RadioGroupItem, RadioGroupOrientation, RadioStyle};
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
pub use tooltip::{PlainTooltip, TooltipAlign, TooltipProvider, TooltipSide};
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use fret_app::App;
    use fret_ui::Theme;

    use crate::tokens::v30::{ColorSchemeOptions, TypographyOptions, theme_config_with_colors};

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
            include_str!("tokens/tooltip.rs"),
            include_str!("tokens/slider.rs"),
            include_str!("tokens/top_app_bar.rs"),
        ];

        for src in sources {
            assert_material_only_tokens(src);
        }
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
                token_resolves(&theme, key),
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
}
