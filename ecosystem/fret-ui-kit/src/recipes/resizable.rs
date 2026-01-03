use fret_core::{Color, Px};
use fret_ui::{ResizablePanelGroupStyle, Theme};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

/// Component-layer default chrome for the runtime `ResizablePanelGroup` mechanism.
///
/// This lives in the component layer so we can:
/// - read component extension tokens (`component.*`, ADR 0050),
/// - evolve defaults without expanding the `fret-ui` runtime contract surface (ADR 0066).
pub fn default_resizable_panel_group_style(theme: &Theme) -> ResizablePanelGroupStyle {
    let handle_color = theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("input"))
        .unwrap_or(theme.snapshot().colors.panel_border);

    let mut style = ResizablePanelGroupStyle {
        gap: theme
            .metric_by_key("component.resizable.gap")
            .unwrap_or(Px(0.0)),
        hit_thickness: theme
            .metric_by_key("component.resizable.hit_thickness")
            .unwrap_or(Px(6.0)),
        paint_device_px: theme
            .metric_by_key("component.resizable.paint_device_px")
            .map(|p| p.0.max(1.0))
            .unwrap_or(1.0),
        handle_color,
        ..Default::default()
    };

    // A slightly softer default alpha tends to match shadcn-ish dividers.
    style.handle_color = alpha_mul(style.handle_color, 0.9);

    style
}
