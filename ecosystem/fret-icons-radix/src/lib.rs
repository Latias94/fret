//! A vendored Radix Icons SVG icon pack for Fret demos/components.

use fret_icons::{IconId, IconRegistry, ids};
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
        let _ = reg.alias(
            ids::ui::ALERT_TRIANGLE,
            IconId::new("radix.exclamation-triangle"),
        );
        let _ = reg.alias(ids::ui::ARROW_LEFT, IconId::new("radix.arrow-left"));
        let _ = reg.alias(ids::ui::ARROW_RIGHT, IconId::new("radix.arrow-right"));
        let _ = reg.alias(ids::ui::BOOK, IconId::new("radix.bookmark"));
        let _ = reg.alias(ids::ui::CHECK, IconId::new("radix.check"));
        let _ = reg.alias(ids::ui::CHEVRON_LEFT, IconId::new("radix.chevron-left"));
        let _ = reg.alias(ids::ui::CHEVRON_DOWN, IconId::new("radix.chevron-down"));
        let _ = reg.alias(ids::ui::CHEVRON_RIGHT, IconId::new("radix.chevron-right"));
        let _ = reg.alias(ids::ui::CHEVRON_UP, IconId::new("radix.chevron-up"));
        let _ = reg.alias(ids::ui::CLOSE, IconId::new("radix.cross-1"));
        let _ = reg.alias(ids::ui::LOADER, IconId::new("radix.update-icon"));
        let _ = reg.alias(ids::ui::SEARCH, IconId::new("radix.magnifying-glass"));
        let _ = reg.alias(ids::ui::SETTINGS, IconId::new("radix.gear"));
        let _ = reg.alias(ids::ui::PLAY, IconId::new("radix.play"));
        let _ = reg.alias(ids::ui::EYE, IconId::new("radix.eye-open"));
        let _ = reg.alias(ids::ui::EYE_OFF, IconId::new("radix.eye-closed"));
        let _ = reg.alias(ids::ui::STATUS_FAILED, IconId::new("radix.cross-circled"));
        let _ = reg.alias(ids::ui::STATUS_PENDING, IconId::new("radix.circle"));
        let _ = reg.alias(ids::ui::STATUS_RUNNING, IconId::new("radix.dot-filled"));
        let _ = reg.alias(
            ids::ui::STATUS_SUCCEEDED,
            IconId::new("radix.check-circled"),
        );
        let _ = reg.alias(ids::ui::TOOL, IconId::new("radix.gear"));
    }
}
