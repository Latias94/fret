//! A vendored Radix Icons SVG icon pack for Fret demos/components.
//!
//! This crate registers a curated subset of Radix Icons SVGs into [`fret_icons::IconRegistry`].
//! Most users will use the higher-level install hooks exposed by the ecosystem `fret` crate.

use fret_icons::{IconId, IconRegistry};

#[cfg(feature = "semantic-ui")]
use fret_icons::ids;
use rust_embed::RustEmbed;
use std::{borrow::Cow, sync::Arc};

pub mod generated_ids;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/*.svg"]
struct Assets;

pub fn register_icons(reg: &mut IconRegistry) {
    register_vendor_icons(reg);

    #[cfg(feature = "semantic-ui")]
    register_ui_semantic_aliases(reg);
}

/// Register Radix vendor icon IDs (`radix.*`) into an [`IconRegistry`].
pub fn register_vendor_icons(reg: &mut IconRegistry) {
    register_curated(reg);
}

/// Register semantic `ui.*` aliases for this icon pack.
#[cfg(feature = "semantic-ui")]
pub fn register_ui_semantic_aliases(reg: &mut IconRegistry) {
    semantic_ui::register(reg);
}

#[cfg(feature = "app-integration")]
pub mod advanced;
#[cfg(feature = "app-integration")]
pub mod app;

fn register_curated(reg: &mut IconRegistry) {
    for line in include_str!("../icon-list.txt").lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let icon_name = line.strip_suffix(".svg").unwrap_or(line);
        register_vendor_icon(reg, icon_name);
    }
}

fn register_vendor_icon(reg: &mut IconRegistry, icon_name: &str) {
    let path = format!("icons/{icon_name}.svg");
    let Some(file) = Assets::get(&path) else {
        return;
    };

    let id = IconId::new(format!("radix.{icon_name}"));
    match file.data {
        Cow::Borrowed(b) => {
            let _ = reg.register_svg_bytes(id, Arc::from(b));
        }
        Cow::Owned(v) => {
            let _ = reg.register_svg_bytes(id, Arc::from(v));
        }
    }
}

#[cfg(feature = "semantic-ui")]
mod semantic_ui {
    use super::*;

    pub fn register(reg: &mut IconRegistry) {
        let _ = reg.alias_if_missing(
            ids::ui::ALERT_TRIANGLE,
            IconId::new("radix.exclamation-triangle"),
        );
        let _ = reg.alias_if_missing(ids::ui::ARROW_LEFT, IconId::new("radix.arrow-left"));
        let _ = reg.alias_if_missing(ids::ui::ARROW_RIGHT, IconId::new("radix.arrow-right"));
        let _ = reg.alias_if_missing(ids::ui::BOOK, IconId::new("radix.bookmark"));
        let _ = reg.alias_if_missing(ids::ui::CHECK, IconId::new("radix.check"));
        let _ = reg.alias_if_missing(ids::ui::CHEVRON_LEFT, IconId::new("radix.chevron-left"));
        let _ = reg.alias_if_missing(ids::ui::CHEVRON_DOWN, IconId::new("radix.chevron-down"));
        let _ = reg.alias_if_missing(ids::ui::CHEVRON_RIGHT, IconId::new("radix.chevron-right"));
        let _ = reg.alias_if_missing(ids::ui::CHEVRON_UP, IconId::new("radix.chevron-up"));
        // Radix Icons does not ship a direct equivalent of Lucide's `chevrons-up-down`.
        // Prefer a reasonable fallback so semantic UI ids stay usable across packs.
        let _ = reg.alias_if_missing(ids::ui::CHEVRONS_UP_DOWN, IconId::new("radix.chevron-down"));
        let _ = reg.alias_if_missing(ids::ui::CLOSE, IconId::new("radix.cross-1"));
        let _ = reg.alias_if_missing(ids::ui::LOADER, IconId::new("radix.update-icon"));
        let _ = reg.alias_if_missing(ids::ui::SEARCH, IconId::new("radix.magnifying-glass"));
        let _ = reg.alias_if_missing(ids::ui::RESET, IconId::new("radix.reset"));
        let _ = reg.alias_if_missing(ids::ui::SETTINGS, IconId::new("radix.gear"));
        let _ = reg.alias_if_missing(ids::ui::PLAY, IconId::new("radix.play"));
        let _ = reg.alias_if_missing(ids::ui::EYE, IconId::new("radix.eye-open"));
        let _ = reg.alias_if_missing(ids::ui::EYE_OFF, IconId::new("radix.eye-closed"));
        let _ = reg.alias_if_missing(ids::ui::STATUS_FAILED, IconId::new("radix.cross-circled"));
        let _ = reg.alias_if_missing(ids::ui::STATUS_PENDING, IconId::new("radix.circle"));
        let _ = reg.alias_if_missing(ids::ui::STATUS_RUNNING, IconId::new("radix.dot-filled"));
        let _ = reg.alias_if_missing(
            ids::ui::STATUS_SUCCEEDED,
            IconId::new("radix.check-circled"),
        );
        let _ = reg.alias_if_missing(ids::ui::TOOL, IconId::new("radix.gear"));
    }
}

#[cfg(test)]
mod tests {
    const LIB_RS: &str = include_str!("lib.rs");
    const APP_RS: &str = include_str!("app.rs");
    const ADVANCED_RS: &str = include_str!("advanced.rs");

    fn public_surface() -> &'static str {
        LIB_RS.split("#[cfg(test)]").next().unwrap_or(LIB_RS)
    }

    #[test]
    fn app_integration_stays_under_explicit_app_module() {
        let public_surface = public_surface();
        assert!(public_surface.contains("pub mod app;"));
        assert!(public_surface.contains("pub mod advanced;"));
        assert!(APP_RS.contains("pub fn install(app: &mut fret_app::App)"));
        assert!(!APP_RS.contains("install_with_ui_services"));
        assert!(ADVANCED_RS.contains("pub fn install_with_ui_services("));
    }
}
