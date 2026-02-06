//! A vendored Radix Icons SVG icon pack for Fret demos/components.

use fret_icons::{ids, IconId, IconRegistry};
use rust_embed::RustEmbed;
use std::{borrow::Cow, sync::Arc};

pub mod generated_ids;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/*.svg"]
struct Assets;

pub fn register_icons(reg: &mut IconRegistry) {
    register_curated(reg);

    #[cfg(feature = "semantic-ui")]
    semantic_ui::register(reg);
}

#[cfg(feature = "app-integration")]
mod app_integration;
#[cfg(feature = "app-integration")]
pub use app_integration::{install, install_app};

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
        Cow::Borrowed(b) => reg.register_svg_bytes(id, Arc::from(b)),
        Cow::Owned(v) => reg.register_svg_bytes(id, Arc::from(v)),
    }
}

#[cfg(feature = "semantic-ui")]
mod semantic_ui {
    use super::*;

    pub fn register(reg: &mut IconRegistry) {
        reg.alias(ids::ui::CHECK, IconId::new("radix.check"));
        reg.alias(ids::ui::CHEVRON_DOWN, IconId::new("radix.chevron-down"));
        reg.alias(ids::ui::CHEVRON_RIGHT, IconId::new("radix.chevron-right"));
        reg.alias(ids::ui::CHEVRON_UP, IconId::new("radix.chevron-up"));
        reg.alias(ids::ui::CLOSE, IconId::new("radix.cross-1"));
        reg.alias(ids::ui::SEARCH, IconId::new("radix.magnifying-glass"));
        reg.alias(ids::ui::SETTINGS, IconId::new("radix.gear"));
        reg.alias(ids::ui::PLAY, IconId::new("radix.play"));
    }
}
