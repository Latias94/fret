use fret_core::{Color, DrawOrder, Edges, Px};
use fret_ui::Theme;
use fret_ui_kit::colors;

fn color_from_srgb8(r: u8, g: u8, b: u8) -> Color {
    colors::linear_from_hex_rgb(((r as u32) << 16) | ((g as u32) << 8) | (b as u32))
}

fn default_series_palette() -> [Color; 10] {
    // ECharts default palette (concept reference): https://echarts.apache.org/en/option.html#color
    [
        color_from_srgb8(0x54, 0x70, 0xC6),
        color_from_srgb8(0x91, 0xCC, 0x75),
        color_from_srgb8(0xEE, 0x66, 0x66),
        color_from_srgb8(0x73, 0xC0, 0xDE),
        color_from_srgb8(0x3B, 0xA2, 0x72),
        color_from_srgb8(0xFC, 0x84, 0x52),
        color_from_srgb8(0x9A, 0x60, 0xB4),
        color_from_srgb8(0xEA, 0x7C, 0xCC),
        color_from_srgb8(0xFA, 0xC8, 0x58),
        color_from_srgb8(0x6E, 0x70, 0x74),
    ]
}

#[derive(Debug, Clone, Copy)]
pub struct ChartStyle {
    pub background: Option<Color>,
    pub stroke_color: Color,
    pub stroke_width: Px,
    pub area_fill_color: Color,
    pub band_fill_color: Color,
    pub bar_fill_alpha: f32,
    pub scatter_point_radius: Px,
    pub scatter_fill_alpha: f32,
    pub selection_fill: Color,
    pub selection_stroke: Color,
    pub selection_stroke_width: Px,

    pub padding: Edges,
    pub axis_band_x: Px,
    pub axis_band_y: Px,
    pub visual_map_band_x: Px,
    pub visual_map_padding: Px,
    pub visual_map_item_gap: Px,
    pub visual_map_corner_radius: Px,
    pub visual_map_track_color: Color,
    pub visual_map_range_fill: Color,
    pub visual_map_range_stroke: Color,
    pub visual_map_handle_color: Color,
    pub axis_line_color: Color,
    pub axis_tick_color: Color,
    pub axis_label_color: Color,
    pub axis_line_width: Px,
    pub axis_tick_length: Px,

    pub crosshair_color: Color,
    pub crosshair_width: Px,
    pub hover_point_color: Color,
    pub hover_point_size: Px,

    pub tooltip_background: Color,
    pub tooltip_border_color: Color,
    pub tooltip_border_width: Px,
    pub tooltip_text_color: Color,
    pub tooltip_padding: Edges,
    pub tooltip_corner_radius: Px,
    pub tooltip_marker_size: Px,
    pub tooltip_marker_gap: Px,
    pub tooltip_column_gap: Px,

    pub legend_background: Color,
    pub legend_border_color: Color,
    pub legend_border_width: Px,
    pub legend_text_color: Color,
    pub legend_padding: Edges,
    pub legend_corner_radius: Px,
    pub legend_item_gap: Px,
    pub legend_swatch_size: Px,
    pub legend_swatch_gap: Px,
    pub legend_hover_background: Color,

    pub series_palette: [Color; 10],
    pub draw_order: DrawOrder,
}

impl Default for ChartStyle {
    fn default() -> Self {
        let series_palette = default_series_palette();

        Self {
            background: Some(Color {
                r: 0.06,
                g: 0.06,
                b: 0.07,
                a: 1.0,
            }),
            stroke_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.9,
            },
            stroke_width: Px(1.0),
            area_fill_color: Color {
                r: 0.2,
                g: 0.6,
                b: 1.0,
                a: 0.18,
            },
            band_fill_color: Color {
                r: 0.2,
                g: 0.6,
                b: 1.0,
                a: 0.12,
            },
            bar_fill_alpha: 0.7,
            scatter_point_radius: Px(5.0),
            scatter_fill_alpha: 0.9,
            selection_fill: Color {
                r: 0.2,
                g: 0.6,
                b: 1.0,
                a: 0.12,
            },
            selection_stroke: Color {
                r: 0.2,
                g: 0.6,
                b: 1.0,
                a: 0.75,
            },
            selection_stroke_width: Px(1.0),
            padding: Edges::all(Px(8.0)),
            axis_band_x: Px(56.0),
            axis_band_y: Px(36.0),
            visual_map_band_x: Px(22.0),
            visual_map_padding: Px(6.0),
            visual_map_item_gap: Px(8.0),
            visual_map_corner_radius: Px(4.0),
            visual_map_track_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.18,
            },
            visual_map_range_fill: Color {
                r: 0.2,
                g: 0.6,
                b: 1.0,
                a: 0.12,
            },
            visual_map_range_stroke: Color {
                r: 0.2,
                g: 0.6,
                b: 1.0,
                a: 0.75,
            },
            visual_map_handle_color: Color {
                r: 0.2,
                g: 0.6,
                b: 1.0,
                a: 0.75,
            },
            axis_line_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.7,
            },
            axis_tick_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.55,
            },
            axis_label_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.8,
            },
            axis_line_width: Px(1.0),
            axis_tick_length: Px(6.0),
            crosshair_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.25,
            },
            crosshair_width: Px(1.0),
            hover_point_color: Color {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 0.9,
            },
            hover_point_size: Px(4.0),
            tooltip_background: Color {
                r: 0.08,
                g: 0.08,
                b: 0.1,
                a: 0.9,
            },
            tooltip_border_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.15,
            },
            tooltip_border_width: Px(1.0),
            tooltip_text_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.9,
            },
            tooltip_padding: Edges::symmetric(Px(8.0), Px(6.0)),
            tooltip_corner_radius: Px(6.0),
            tooltip_marker_size: Px(8.0),
            tooltip_marker_gap: Px(6.0),
            tooltip_column_gap: Px(10.0),
            legend_background: Color {
                r: 0.08,
                g: 0.08,
                b: 0.1,
                a: 0.9,
            },
            legend_border_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.15,
            },
            legend_border_width: Px(1.0),
            legend_text_color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.9,
            },
            legend_padding: Edges::symmetric(Px(10.0), Px(8.0)),
            legend_corner_radius: Px(8.0),
            legend_item_gap: Px(4.0),
            legend_swatch_size: Px(10.0),
            legend_swatch_gap: Px(8.0),
            legend_hover_background: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 0.06,
            },
            series_palette,
            draw_order: DrawOrder(100),
        }
    }
}

impl ChartStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        fn color(theme: &Theme, key: &str) -> Option<Color> {
            theme.color_by_key(key)
        }

        fn metric(theme: &Theme, key: &str) -> Option<Px> {
            theme.metric_by_key(key)
        }

        fn with_alpha(mut c: Color, a: f32) -> Color {
            c.a *= a.clamp(0.0, 1.0);
            c
        }

        fn pick_color(theme: &Theme, key: &str, fallback: Color) -> Color {
            color(theme, key).unwrap_or(fallback)
        }

        fn pick_metric(theme: &Theme, key: &str, fallback: Px) -> Px {
            metric(theme, key).unwrap_or(fallback)
        }

        let foreground = theme.color_token("foreground");
        let muted_foreground = theme.color_token("muted-foreground");
        let border = theme.color_token("border");
        let primary = theme.color_token("primary");
        let popover = theme.color_token("popover");

        let background = pick_color(theme, "chart.background", theme.color_token("card"));
        let tooltip_background =
            pick_color(theme, "chart.tooltip.background", with_alpha(popover, 0.9));
        let tooltip_border = pick_color(theme, "chart.tooltip.border", with_alpha(border, 0.15));
        let tooltip_text = pick_color(theme, "chart.tooltip.text", with_alpha(foreground, 0.9));

        let legend_background =
            pick_color(theme, "chart.legend.background", with_alpha(popover, 0.9));
        let legend_border = pick_color(theme, "chart.legend.border", with_alpha(border, 0.15));
        let legend_text = pick_color(theme, "chart.legend.text", with_alpha(foreground, 0.9));

        let axis_line_color =
            pick_color(theme, "chart.axis.line", with_alpha(muted_foreground, 0.7));
        let axis_tick_color =
            pick_color(theme, "chart.axis.tick", with_alpha(muted_foreground, 0.55));
        let axis_label_color = pick_color(theme, "chart.axis.label", with_alpha(foreground, 0.8));

        let crosshair_color = pick_color(theme, "chart.crosshair", with_alpha(foreground, 0.25));

        let selection_fill = pick_color(theme, "chart.selection.fill", with_alpha(primary, 0.12));
        let selection_stroke =
            pick_color(theme, "chart.selection.stroke", with_alpha(primary, 0.75));

        let stroke_width = pick_metric(theme, "metric.chart.stroke.width", Px(1.0));
        let axis_line_width = pick_metric(theme, "metric.chart.axis.line.width", Px(1.0));
        let axis_tick_length = pick_metric(theme, "metric.chart.axis.tick.length", Px(6.0));
        let scatter_point_radius = pick_metric(theme, "metric.chart.scatter.point_radius", Px(5.0));
        let hover_point_size = pick_metric(theme, "metric.chart.hover.point_size", Px(4.0));
        let tooltip_border_width = pick_metric(theme, "metric.chart.tooltip.border.width", Px(1.0));
        let legend_border_width = pick_metric(theme, "metric.chart.legend.border.width", Px(1.0));
        let selection_stroke_width =
            pick_metric(theme, "metric.chart.selection.stroke.width", Px(1.0));

        let padding_all = metric(theme, "metric.chart.padding")
            .unwrap_or_else(|| theme.metric_token("metric.padding.sm"));
        let padding = Edges::all(padding_all);

        let axis_band_x = pick_metric(theme, "metric.chart.axis.band.x", Px(56.0));
        let axis_band_y = pick_metric(theme, "metric.chart.axis.band.y", Px(36.0));
        let visual_map_band_x = pick_metric(theme, "metric.chart.visualmap.band.x", Px(22.0));
        let visual_map_padding = pick_metric(theme, "metric.chart.visualmap.pad", Px(6.0));
        let visual_map_item_gap = pick_metric(theme, "metric.chart.visualmap.item.gap", Px(8.0));
        let visual_map_corner_radius =
            pick_metric(theme, "metric.chart.visualmap.corner_radius", Px(4.0));

        let tooltip_padding_x = pick_metric(theme, "metric.chart.tooltip.padding.x", Px(8.0));
        let tooltip_padding_y = pick_metric(theme, "metric.chart.tooltip.padding.y", Px(6.0));
        let tooltip_corner_radius = pick_metric(
            theme,
            "metric.chart.tooltip.corner_radius",
            theme.metric_token("metric.radius.sm"),
        );
        let tooltip_marker_size = pick_metric(theme, "metric.chart.tooltip.marker.size", Px(8.0));
        let tooltip_marker_gap = pick_metric(theme, "metric.chart.tooltip.marker.gap", Px(6.0));
        let tooltip_column_gap = pick_metric(theme, "metric.chart.tooltip.column.gap", Px(10.0));

        let legend_padding_x = pick_metric(theme, "metric.chart.legend.padding.x", Px(10.0));
        let legend_padding_y = pick_metric(theme, "metric.chart.legend.padding.y", Px(8.0));
        let legend_corner_radius = pick_metric(
            theme,
            "metric.chart.legend.corner_radius",
            theme.metric_token("metric.radius.md"),
        );
        let legend_item_gap = pick_metric(theme, "metric.chart.legend.item.gap", Px(4.0));
        let legend_swatch_size = pick_metric(theme, "metric.chart.legend.swatch.size", Px(10.0));
        let legend_swatch_gap = pick_metric(theme, "metric.chart.legend.swatch.gap", Px(8.0));

        let legend_hover_background = pick_color(
            theme,
            "chart.legend.hover.background",
            with_alpha(foreground, 0.06),
        );

        let visual_map_track_color = pick_color(
            theme,
            "chart.visualmap.track",
            with_alpha(axis_line_color, 0.25),
        );
        let visual_map_range_fill = pick_color(theme, "chart.visualmap.range.fill", selection_fill);
        let visual_map_range_stroke =
            pick_color(theme, "chart.visualmap.range.stroke", selection_stroke);
        let visual_map_handle_color =
            pick_color(theme, "chart.visualmap.handle", visual_map_range_stroke);

        const PALETTE_KEYS: [&str; 10] = [
            "chart.palette.0",
            "chart.palette.1",
            "chart.palette.2",
            "chart.palette.3",
            "chart.palette.4",
            "chart.palette.5",
            "chart.palette.6",
            "chart.palette.7",
            "chart.palette.8",
            "chart.palette.9",
        ];
        const SHADCN_CHART_KEYS: [&str; 5] =
            ["chart-1", "chart-2", "chart-3", "chart-4", "chart-5"];

        let fallback_palette = default_series_palette();
        let mut series_palette = fallback_palette;
        for (index, key) in PALETTE_KEYS.iter().enumerate() {
            if let Some(c) = color(theme, key) {
                series_palette[index] = c;
                continue;
            }
            if index < SHADCN_CHART_KEYS.len()
                && let Some(c) = color(theme, SHADCN_CHART_KEYS[index]) {
                    series_palette[index] = c;
                }
        }

        Self {
            background: Some(background),
            stroke_color: with_alpha(foreground, 0.9),
            stroke_width,
            area_fill_color: with_alpha(primary, 0.18),
            band_fill_color: with_alpha(primary, 0.12),
            bar_fill_alpha: 0.7,
            scatter_point_radius,
            scatter_fill_alpha: 0.9,
            selection_fill,
            selection_stroke,
            selection_stroke_width,
            padding,
            axis_band_x,
            axis_band_y,
            visual_map_band_x,
            visual_map_padding,
            visual_map_item_gap,
            visual_map_corner_radius,
            visual_map_track_color,
            visual_map_range_fill,
            visual_map_range_stroke,
            visual_map_handle_color,
            axis_line_color,
            axis_tick_color,
            axis_label_color,
            axis_line_width,
            axis_tick_length,
            crosshair_color,
            crosshair_width: Px(1.0),
            hover_point_color: with_alpha(foreground, 0.9),
            hover_point_size,
            tooltip_background,
            tooltip_border_color: tooltip_border,
            tooltip_border_width,
            tooltip_text_color: tooltip_text,
            tooltip_padding: Edges::symmetric(tooltip_padding_x, tooltip_padding_y),
            tooltip_corner_radius,
            tooltip_marker_size,
            tooltip_marker_gap,
            tooltip_column_gap,
            legend_background,
            legend_border_color: legend_border,
            legend_border_width,
            legend_text_color: legend_text,
            legend_padding: Edges::symmetric(legend_padding_x, legend_padding_y),
            legend_corner_radius,
            legend_item_gap,
            legend_swatch_size,
            legend_swatch_gap,
            legend_hover_background,
            series_palette,
            draw_order: DrawOrder(100),
        }
    }
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_ui::{Theme, ThemeConfig};

    use super::ChartStyle;

    #[test]
    fn series_palette_prefers_chart_palette_tokens_over_shadcn_aliases() {
        let mut app = App::new();
        let mut cfg = ThemeConfig::default();
        cfg.colors
            .insert("chart.palette.0".to_string(), "#FF0000".to_string());
        cfg.colors
            .insert("chart-1".to_string(), "#00FF00".to_string());
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&cfg));

        let theme = Theme::global(&app);
        let style = ChartStyle::from_theme(theme);
        assert_eq!(
            style.series_palette[0],
            theme.color_token("chart.palette.0")
        );
    }
}
