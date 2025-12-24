use fret_core::{Corners, Px};
use fret_ui::{Theme, VirtualListStyle};

use crate::Size;

pub fn list_style(theme: &Theme, size: Size) -> VirtualListStyle {
    let text_px = size.control_text_px(theme);

    let mut style = VirtualListStyle::default();
    style.background = theme.colors.list_background;
    style.border_color = theme.colors.list_border;
    style.corner_radii = Corners::all(theme.metrics.radius_md);
    style.row_hover = theme.colors.list_row_hover;
    style.row_selected = theme.colors.list_row_selected;
    style.row_highlight_inset_y = theme
        .metric_by_key("metric.list.row_highlight_inset_y")
        .unwrap_or(Px(0.0));

    style.text_color = theme.colors.text_primary;
    style.secondary_text_color = theme.colors.text_muted;
    style.trailing_text_color = theme.colors.text_muted;
    style.header_text_color = theme.colors.text_muted;
    style.separator_color = theme.colors.panel_border;

    style.padding_x = size.list_px(theme);
    style.padding_y = size.list_py(theme);
    if let Some(gap) = theme.metric_by_key("metric.list.row_gap_y") {
        style.row_gap_y = gap;
    }
    style.trailing_gap_x = theme
        .metric_by_key("metric.list.trailing_gap_x")
        .unwrap_or(theme.metrics.padding_sm);
    style.separator_inset_x = theme
        .metric_by_key("metric.list.separator_inset_x")
        .unwrap_or(theme.metrics.padding_md);

    style.text_style.size = text_px;
    style.secondary_text_style.size = Px((text_px.0 - 1.0).max(0.0));
    style.trailing_text_style.size = Px((text_px.0 - 1.0).max(0.0));
    style.header_text_style.size = Px((text_px.0 - 1.0).max(0.0));

    style
}
