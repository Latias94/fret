//! A vendored Lucide SVG icon pack for Fret demos/components.

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

    let bytes: Arc<[u8]> = match file.data {
        Cow::Borrowed(b) => Arc::from(b),
        Cow::Owned(v) => Arc::from(v),
    };

    let _ = reg.register_svg_bytes(IconId::new(format!("lucide.{icon_name}")), bytes);
}

#[cfg(feature = "semantic-ui")]
mod semantic_ui {
    use super::*;

    pub fn register(reg: &mut IconRegistry) {
        let _ = reg.alias(ids::ui::ARROW_LEFT, IconId::new("lucide.arrow-left"));
        let _ = reg.alias(ids::ui::ARROW_RIGHT, IconId::new("lucide.arrow-right"));
        let _ = reg.alias(ids::ui::BOOK, IconId::new("lucide.book"));
        let _ = reg.alias(ids::ui::CHECK, IconId::new("lucide.check"));
        let _ = reg.alias(ids::ui::CHEVRON_LEFT, IconId::new("lucide.chevron-left"));
        let _ = reg.alias(ids::ui::CHEVRON_DOWN, IconId::new("lucide.chevron-down"));
        let _ = reg.alias(ids::ui::CHEVRON_RIGHT, IconId::new("lucide.chevron-right"));
        let _ = reg.alias(ids::ui::CHEVRON_UP, IconId::new("lucide.chevron-up"));
        let _ = reg.alias(ids::ui::CLOSE, IconId::new("lucide.x"));
        let _ = reg.alias(ids::ui::FILE, IconId::new("lucide.file"));
        let _ = reg.alias(ids::ui::FOLDER, IconId::new("lucide.folder"));
        let _ = reg.alias(ids::ui::FOLDER_OPEN, IconId::new("lucide.folder-open"));
        let _ = reg.alias(ids::ui::LOADER, IconId::new("lucide.loader-circle"));
        let _ = reg.alias(ids::ui::MORE_HORIZONTAL, IconId::new("lucide.ellipsis"));
        let _ = reg.alias(ids::ui::MINUS, IconId::new("lucide.minus"));
        let _ = reg.alias(ids::ui::SEARCH, IconId::new("lucide.search"));
        let _ = reg.alias(ids::ui::SETTINGS, IconId::new("lucide.settings"));
        let _ = reg.alias(ids::ui::PLAY, IconId::new("lucide.play"));
        let _ = reg.alias(ids::ui::SLASH, IconId::new("lucide.slash"));
        let _ = reg.alias(ids::ui::STATUS_FAILED, IconId::new("lucide.circle-x"));
        let _ = reg.alias(ids::ui::STATUS_PENDING, IconId::new("lucide.circle"));
        let _ = reg.alias(ids::ui::STATUS_RUNNING, IconId::new("lucide.clock"));
        let _ = reg.alias(
            ids::ui::STATUS_SUCCEEDED,
            IconId::new("lucide.circle-check"),
        );
        let _ = reg.alias(ids::ui::TOOL, IconId::new("lucide.wrench"));
    }
}
