//! A small curated set of Lucide SVG icons (vendored) for Fret demos/components.

use fret_icons::{IconId, IconRegistry, ids};
use rust_embed::RustEmbed;
use std::{borrow::Cow, sync::Arc};

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/*.svg"]
struct Assets;

pub fn register_icons(reg: &mut IconRegistry) {
    register_curated(reg);

    #[cfg(feature = "semantic-ui")]
    semantic_ui::register(reg);
}

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

    reg.register_svg_bytes(IconId::new(format!("lucide.{icon_name}")), bytes);
}

#[cfg(feature = "semantic-ui")]
mod semantic_ui {
    use super::*;

    pub fn register(reg: &mut IconRegistry) {
        reg.alias(ids::ui::CHECK, IconId::new("lucide.check"));
        reg.alias(ids::ui::CHEVRON_DOWN, IconId::new("lucide.chevron-down"));
        reg.alias(ids::ui::CLOSE, IconId::new("lucide.x"));
        reg.alias(ids::ui::SEARCH, IconId::new("lucide.search"));
        reg.alias(ids::ui::SETTINGS, IconId::new("lucide.settings"));
        reg.alias(ids::ui::PLAY, IconId::new("lucide.play"));
    }
}
