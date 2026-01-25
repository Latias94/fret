//! Material Design 3 (and Expressive) component surface for Fret.
//!
//! This crate targets **visual + interaction outcome alignment** with the Material 3 design
//! system, while keeping `crates/fret-ui` focused on mechanisms (not Material-specific policy).

#![forbid(unsafe_code)]

pub mod button;
pub mod checkbox;
pub mod dialog;
pub mod dropdown_menu;
mod foundation;
pub mod icon_button;
pub mod interaction;
pub mod list;
pub mod menu;
pub mod modal_navigation_drawer;
pub mod motion;
pub mod navigation_bar;
pub mod navigation_drawer;
pub mod navigation_rail;
pub mod radio;
pub mod snackbar;
pub mod switch;
pub mod tabs;
pub mod text_field;
pub mod theme;
pub mod tokens;
pub mod tooltip;

pub use button::{Button, ButtonVariant};
pub use checkbox::Checkbox;
pub use dialog::{Dialog, DialogAction};
pub use dropdown_menu::{DropdownMenu, DropdownMenuAlign, DropdownMenuSide};
pub use icon_button::{IconButton, IconButtonSize, IconButtonVariant};
pub use list::{List, ListItem};
pub use menu::{Menu, MenuEntry, MenuItem};
pub use modal_navigation_drawer::ModalNavigationDrawer;
pub use navigation_bar::{NavigationBar, NavigationBarItem};
pub use navigation_drawer::{NavigationDrawer, NavigationDrawerItem, NavigationDrawerVariant};
pub use navigation_rail::{NavigationRail, NavigationRailItem};
pub use radio::{Radio, RadioGroup, RadioGroupItem, RadioGroupOrientation};
pub use snackbar::{Snackbar, SnackbarController, SnackbarDuration, SnackbarHost};
pub use switch::Switch;
pub use tabs::{TabItem, Tabs};
pub use text_field::{TextField, TextFieldVariant};
pub use tooltip::{PlainTooltip, TooltipAlign, TooltipProvider, TooltipSide};

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
            include_str!("checkbox.rs"),
            include_str!("dialog.rs"),
            include_str!("dropdown_menu.rs"),
            include_str!("icon_button.rs"),
            include_str!("list.rs"),
            include_str!("menu.rs"),
            include_str!("modal_navigation_drawer.rs"),
            include_str!("navigation_bar.rs"),
            include_str!("navigation_drawer.rs"),
            include_str!("navigation_rail.rs"),
            include_str!("radio.rs"),
            include_str!("snackbar.rs"),
            include_str!("switch.rs"),
            include_str!("tabs.rs"),
            include_str!("text_field.rs"),
            include_str!("tooltip.rs"),
            include_str!("foundation/indication.rs"),
            include_str!("foundation/focus_ring.rs"),
            include_str!("foundation/geometry.rs"),
            include_str!("foundation/tokens.rs"),
            include_str!("tokens/icon_button.rs"),
            include_str!("tokens/button.rs"),
            include_str!("tokens/switch.rs"),
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
            include_str!("checkbox.rs"),
            include_str!("dialog.rs"),
            include_str!("dropdown_menu.rs"),
            include_str!("icon_button.rs"),
            include_str!("list.rs"),
            include_str!("menu.rs"),
            include_str!("modal_navigation_drawer.rs"),
            include_str!("navigation_bar.rs"),
            include_str!("navigation_drawer.rs"),
            include_str!("navigation_rail.rs"),
            include_str!("radio.rs"),
            include_str!("snackbar.rs"),
            include_str!("switch.rs"),
            include_str!("tabs.rs"),
            include_str!("text_field.rs"),
            include_str!("tooltip.rs"),
            include_str!("foundation/indication.rs"),
            include_str!("foundation/focus_ring.rs"),
            include_str!("foundation/geometry.rs"),
            include_str!("foundation/tokens.rs"),
            include_str!("tokens/icon_button.rs"),
            include_str!("tokens/button.rs"),
            include_str!("tokens/switch.rs"),
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
