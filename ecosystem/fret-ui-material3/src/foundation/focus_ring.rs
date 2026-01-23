use fret_core::{Corners, Px};
use fret_ui::Theme;

pub fn material_focus_ring(theme: &Theme, corner_radii: Corners) -> fret_ui::element::RingStyle {
    let mut c = theme
        .color_by_key("md.sys.color.primary")
        .unwrap_or_else(|| theme.color_required("md.sys.color.primary"));
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
