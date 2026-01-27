use fret_core::{Color, Corners, Px};
use fret_ui::Theme;
use fret_ui::element::ShadowStyle;

use crate::foundation::elevation::{
    apply_surface_tint_if_surface, shadow_for_elevation_with_color,
};

#[derive(Debug, Clone, Copy)]
pub struct MaterialSurfaceStyle {
    pub background: Color,
    pub shadow: Option<ShadowStyle>,
}

pub fn material_surface_style(
    theme: &Theme,
    background: Color,
    elevation: Px,
    shadow_color: Option<Color>,
    corner_radii: Corners,
) -> MaterialSurfaceStyle {
    let background = apply_surface_tint_if_surface(theme, background, elevation);
    let shadow = shadow_for_elevation_with_color(theme, elevation, shadow_color, corner_radii);
    MaterialSurfaceStyle { background, shadow }
}
