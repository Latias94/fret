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
pub mod menu;
pub mod modal_navigation_drawer;
pub mod motion;
pub mod navigation_bar;
pub mod navigation_drawer;
pub mod navigation_rail;
pub mod radio;
pub mod switch;
pub mod tabs;
pub mod text_field;
pub mod theme;
pub mod tokens;

pub use button::{Button, ButtonVariant};
pub use checkbox::Checkbox;
pub use dialog::{Dialog, DialogAction};
pub use dropdown_menu::{DropdownMenu, DropdownMenuAlign, DropdownMenuSide};
pub use icon_button::{IconButton, IconButtonSize, IconButtonVariant};
pub use menu::{Menu, MenuEntry, MenuItem};
pub use modal_navigation_drawer::ModalNavigationDrawer;
pub use navigation_bar::{NavigationBar, NavigationBarItem};
pub use navigation_drawer::{NavigationDrawer, NavigationDrawerItem};
pub use navigation_rail::{NavigationRail, NavigationRailItem};
pub use radio::{Radio, RadioGroup, RadioGroupItem, RadioGroupOrientation};
pub use switch::Switch;
pub use tabs::{TabItem, Tabs};
pub use text_field::{TextField, TextFieldVariant};

#[cfg(test)]
mod tests {
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
            include_str!("menu.rs"),
            include_str!("modal_navigation_drawer.rs"),
            include_str!("navigation_bar.rs"),
            include_str!("navigation_drawer.rs"),
            include_str!("navigation_rail.rs"),
            include_str!("radio.rs"),
            include_str!("switch.rs"),
            include_str!("tabs.rs"),
            include_str!("text_field.rs"),
            include_str!("foundation/indication.rs"),
            include_str!("foundation/focus_ring.rs"),
            include_str!("foundation/geometry.rs"),
            include_str!("foundation/tokens.rs"),
        ];

        for src in sources {
            assert_material_only_tokens(src);
        }
    }
}
