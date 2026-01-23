use fret_core::{Corners, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

pub fn material_focus_ring(theme: &Theme, corner_radii: Corners) -> fret_ui::element::RingStyle {
    let tokens = MaterialTokenResolver::new(theme);
    let mut c = tokens.color_sys("md.sys.color.primary");
    c.a = 1.0;

    fret_ui::element::RingStyle {
        placement: fret_ui::element::RingPlacement::Outset,
        width: Px(2.0),
        offset: Px(2.0),
        color: c,
        offset_color: None,
        corner_radii,
    }
}
