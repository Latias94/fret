use fret_core::{Color, Corners, Edges, Px};
use fret_ui::Theme;
use fret_ui::element::{RingPlacement, RingStyle};

pub fn default_text_input_style(theme: &Theme) -> fret_ui::TextInputStyle {
    let ring_width = theme
        .metric_by_key("component.ring.width")
        .unwrap_or(Px(2.0));
    let ring_offset = theme
        .metric_by_key("component.ring.offset")
        .unwrap_or(Px(2.0));
    // shadcn/new-york-v4 uses `ring-ring/50` for the ring color.
    let ring_color = theme
        .color_by_key("ring/50")
        .or_else(|| theme.color_by_key("ring"))
        .unwrap_or_else(|| theme.color_token("ring"));
    let ring_offset_color = theme
        .color_by_key("ring-offset-background")
        .unwrap_or_else(|| theme.color_token("ring-offset-background"));

    let background = theme
        .color_by_key("component.input.bg")
        .unwrap_or_else(|| theme.color_token("background"));
    let border_color = theme
        .color_by_key("component.input.border")
        .unwrap_or_else(|| theme.color_token("input"));
    // shadcn/new-york-v4 uses `focus-visible:border-ring`.
    let border_color_focused = theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_token("ring"));
    let radius = theme
        .metric_by_key("component.input.radius")
        .unwrap_or_else(|| theme.metric_token("metric.radius.sm"));
    let selection = theme
        .color_by_key("component.input.selection")
        .unwrap_or_else(|| theme.color_token("selection.background"));
    let preedit_bg_color = {
        let mut bg = selection;
        bg.a = (bg.a * 0.35).clamp(0.0, 1.0);
        bg
    };

    fret_ui::TextInputStyle {
        padding: Edges::all(Px(0.0)),
        background,
        border: Edges::all(Px(1.0)),
        border_color,
        border_color_focused,
        focus_ring: Some(RingStyle {
            placement: RingPlacement::Outset,
            width: ring_width,
            offset: ring_offset,
            color: ring_color,
            offset_color: (ring_offset.0 > 0.0).then_some(ring_offset_color),
            corner_radii: Corners::all(radius),
        }),
        corner_radii: Corners::all(radius),
        text_color: theme.color_token("foreground"),
        placeholder_color: theme.color_token("muted-foreground"),
        selection_color: Color {
            a: 1.0,
            ..selection
        },
        caret_color: theme.color_token("foreground"),
        preedit_bg_color,
        preedit_color: theme.color_token("primary"),
        preedit_underline_color: theme.color_token("primary"),
    }
}
