//! Minimal Radix Icons SVG icon pack for Fret demos and component bootstrap.
//!
//! This crate registers a small curated set of icons used by the in-tree demos/components.

use fret_components_icons::{IconRegistry, ids};

pub fn register_icons(reg: &mut IconRegistry) {
    #[cfg(feature = "semantic-ui")]
    semantic_ui::register(reg);
}

#[cfg(feature = "semantic-ui")]
mod semantic_ui {
    use super::*;

    const CHECK: &[u8] = include_bytes!("../assets/icons/check.svg");
    const CHEVRON_DOWN: &[u8] = include_bytes!("../assets/icons/chevron-down.svg");
    const CROSS_1: &[u8] = include_bytes!("../assets/icons/cross-1.svg");
    const MAGNIFYING_GLASS: &[u8] = include_bytes!("../assets/icons/magnifying-glass.svg");
    const GEAR: &[u8] = include_bytes!("../assets/icons/gear.svg");
    const PLAY: &[u8] = include_bytes!("../assets/icons/play.svg");

    pub fn register(reg: &mut IconRegistry) {
        reg.register_svg_static(
            fret_components_icons::IconId::new_static("radix.check"),
            CHECK,
        );
        reg.register_svg_static(
            fret_components_icons::IconId::new_static("radix.chevron-down"),
            CHEVRON_DOWN,
        );
        reg.register_svg_static(
            fret_components_icons::IconId::new_static("radix.cross-1"),
            CROSS_1,
        );
        reg.register_svg_static(
            fret_components_icons::IconId::new_static("radix.magnifying-glass"),
            MAGNIFYING_GLASS,
        );
        reg.register_svg_static(
            fret_components_icons::IconId::new_static("radix.gear"),
            GEAR,
        );
        reg.register_svg_static(
            fret_components_icons::IconId::new_static("radix.play"),
            PLAY,
        );

        reg.register_svg_static(ids::ui::CHECK, CHECK);
        reg.register_svg_static(ids::ui::CHEVRON_DOWN, CHEVRON_DOWN);
        reg.register_svg_static(ids::ui::CLOSE, CROSS_1);
        reg.register_svg_static(ids::ui::SEARCH, MAGNIFYING_GLASS);
        reg.register_svg_static(ids::ui::SETTINGS, GEAR);
        reg.register_svg_static(ids::ui::PLAY, PLAY);
    }
}
