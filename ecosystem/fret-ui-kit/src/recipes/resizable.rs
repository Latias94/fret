use fret_core::Px;
use fret_ui::{ResizablePanelGroupStyle, Theme};

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

    ResizablePanelGroupStyle {
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
        // shadcn/ui uses `bg-border` for the handle line (no extra opacity multiplier).
        handle_alpha: 1.0,
        handle_hover_alpha: 1.0,
        handle_drag_alpha: 1.0,
        ..Default::default()
    }
}
