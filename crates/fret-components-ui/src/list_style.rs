use fret_core::{Corners, Px};
use fret_ui::{Theme, VirtualListStyle};

use crate::style::MetricRef;
use crate::{Size, Space};

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
    style.trailing_gap_x = theme
        .metric_by_key("metric.list.trailing_gap_x")
        .unwrap_or(theme.metrics.padding_sm);
    style.separator_inset_x = theme
        .metric_by_key("metric.list.separator_inset_x")
        .unwrap_or(theme.metrics.padding_md);

    style.row_highlight_inset_y = theme
        .metric_by_key("metric.list.row_highlight_inset_y")
        .unwrap_or_else(|| MetricRef::space(Space::N0p5).resolve(theme));
    if let Some(gap) = theme.metric_by_key("metric.list.row_gap_y") {
        style.row_gap_y = gap;
    }

    style.text_style.size = text_px;
    style.secondary_text_style.size = Px((text_px.0 - 1.0).max(0.0));
    style.trailing_text_style.size = Px((text_px.0 - 1.0).max(0.0));
    style.header_text_style.size = Px((text_px.0 - 1.0).max(0.0));

    style
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui::ThemeConfig;

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
    fn list_row_highlight_inset_can_be_overridden_by_metric_token() {
        let mut app = fret_app::App::default();
        let cfg = ThemeConfig {
            name: "Test".to_string(),
            metrics: std::collections::HashMap::from([(
                "metric.list.row_highlight_inset_y".to_string(),
                7.0,
            )]),
            ..ThemeConfig::default()
        };
        Theme::global_mut(&mut app).apply_config(&cfg);

        let theme = Theme::global(&app);
        let style = list_style(theme, Size::Medium);
        assert_eq!(style.row_highlight_inset_y, Px(7.0));
    }
}
