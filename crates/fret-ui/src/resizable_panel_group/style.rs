use fret_core::{Color, Px};

use crate::Theme;

#[derive(Debug, Clone)]
pub struct ResizablePanelGroupStyle {
    /// Layout gap between panels in logical px.
    ///
    /// This does **not** need to match `hit_thickness`: it is common to keep the visual/layout gap
    /// small (or zero) while using a larger hit area for usability.
    pub gap: Px,
    /// Thickness of the handle region in logical px.
    ///
    /// This region is used for hit-testing (and can be larger than `gap`).
    pub hit_thickness: Px,
    /// Visual thickness in *device* pixels (converted using the current scale factor).
    pub paint_device_px: f32,
    pub handle_color: Color,
    pub handle_alpha: f32,
    pub handle_hover_alpha: f32,
    pub handle_drag_alpha: f32,
}

impl Default for ResizablePanelGroupStyle {
    fn default() -> Self {
        Self {
            gap: Px(0.0),
            hit_thickness: Px(6.0),
            paint_device_px: 1.0,
            handle_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            // Align with shadcn/ui: the handle line should use the border color at full opacity.
            // Contrast remains subtle because the `border` token is intentionally low-contrast.
            handle_alpha: 1.0,
            handle_hover_alpha: 1.0,
            handle_drag_alpha: 1.0,
        }
    }
}

impl ResizablePanelGroupStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        let handle_color = theme
            .color_by_key("border")
            .or_else(|| theme.color_by_key("input"))
            .unwrap_or(theme.snapshot().colors.panel_border);

        Self {
            // Note: this runtime crate intentionally avoids reading component-layer extension tokens
            // (e.g. `component.*`). Component libraries should resolve their own chrome and pass an
            // explicit `ResizablePanelGroupStyle` when needed.
            gap: Px(0.0),
            hit_thickness: Px(6.0),
            paint_device_px: 1.0,
            handle_color,
            ..Default::default()
        }
    }
}
