use fret_core::{Corners, Px};
use fret_ui::Theme;
use fret_ui::primitives::{VirtualListRowHeight, VirtualListStyle};

use crate::style::MetricRef;
use crate::{Size, Space};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListRowHeightMode {
    Fixed,
    Measured,
}

pub fn list_row_height(theme: &Theme, size: Size, mode: ListRowHeightMode) -> VirtualListRowHeight {
    let min = size.list_row_h(theme);
    match mode {
        ListRowHeightMode::Fixed => VirtualListRowHeight::Fixed(min),
        ListRowHeightMode::Measured => VirtualListRowHeight::Measured { min },
    }
}

pub fn list_style(theme: &Theme, size: Size) -> VirtualListStyle {
    let text_px = size.control_text_px(theme);

    let mut style = VirtualListStyle::default();
    style.background = theme
        .color_by_key("list.background")
        .unwrap_or(theme.colors.list_background);
    style.border_color = theme
        .color_by_key("border")
        .unwrap_or(theme.colors.list_border);
    style.corner_radii = Corners::all(theme.metrics.radius_md);
    style.row_hover = theme
        .color_by_key("list.hover.background")
        .unwrap_or(theme.colors.list_row_hover);
    style.row_selected = theme
        .color_by_key("list.active.background")
        .unwrap_or(theme.colors.list_row_selected);

    style.text_color = theme
        .color_by_key("foreground")
        .unwrap_or(theme.colors.text_primary);
    style.secondary_text_color = theme
        .color_by_key("muted.foreground")
        .unwrap_or(theme.colors.text_muted);
    style.trailing_text_color = theme
        .color_by_key("muted.foreground")
        .unwrap_or(theme.colors.text_muted);
    style.header_text_color = theme
        .color_by_key("muted.foreground")
        .unwrap_or(theme.colors.text_muted);
    style.separator_color = theme
        .color_by_key("border")
        .unwrap_or(theme.colors.panel_border);

    style.padding_x = size.list_px(theme);
    style.padding_y = size.list_py(theme);

    style.row_gap_y = theme
        .metric_by_key("component.list.row_gap_y")
        .or_else(|| theme.metric_by_key("metric.list.row_gap_y"))
        .unwrap_or_else(|| MetricRef::space(Space::N0p5).resolve(theme));
    style.trailing_gap_x = theme
        .metric_by_key("component.list.trailing_gap_x")
        .or_else(|| theme.metric_by_key("metric.list.trailing_gap_x"))
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme));
    style.separator_inset_x = theme
        .metric_by_key("component.list.separator_inset_x")
        .or_else(|| theme.metric_by_key("metric.list.separator_inset_x"))
        .unwrap_or(style.padding_x);
    style.row_highlight_inset_y = theme
        .metric_by_key("component.list.row_highlight_inset_y")
        .or_else(|| theme.metric_by_key("metric.list.row_highlight_inset_y"))
        .unwrap_or_else(|| MetricRef::space(Space::N0p5).resolve(theme));

    style.text_style.size = text_px;
    style.secondary_text_style.size = Px((text_px.0 - 1.0).max(0.0));
    style.trailing_text_style.size = Px((text_px.0 - 1.0).max(0.0));
    style.header_text_style.size = Px((text_px.0 - 1.0).max(0.0));

    style
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_row_highlight_inset_defaults_to_space_fallback() {
        let app = fret_app::App::default();
        let theme = Theme::global(&app);

        let style = list_style(theme, Size::Medium);
        assert_eq!(
            style.row_highlight_inset_y,
            MetricRef::space(Space::N0p5).resolve(theme)
        );
    }

    #[test]
    fn list_row_highlight_inset_can_be_overridden_by_component_token() {
        let mut app = fret_app::App::default();
        let cfg = fret_ui::ThemeConfig {
            name: "Test".to_string(),
            metrics: std::collections::HashMap::from([(
                "component.list.row_highlight_inset_y".to_string(),
                7.0,
            )]),
            ..fret_ui::ThemeConfig::default()
        };
        Theme::global_mut(&mut app).apply_config(&cfg);

        let theme = Theme::global(&app);
        let style = list_style(theme, Size::Medium);
        assert_eq!(style.row_highlight_inset_y, Px(7.0));
    }
}
