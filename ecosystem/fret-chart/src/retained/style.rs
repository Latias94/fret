use fret_core::{Color, DrawOrder, Edges, Px};
use fret_ui::Theme;

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
    pub draw_order: DrawOrder,
}

impl Default for ChartStyle {
    fn default() -> Self {
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

        let foreground = theme.color_required("foreground");
        let muted_foreground = theme.color_required("muted-foreground");
        let border = theme.color_required("border");
        let primary = theme.color_required("primary");
        let popover = theme.color_required("popover");

        let background = pick_color(theme, "chart.background", theme.color_required("card"));
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
            .unwrap_or_else(|| theme.metric_required("metric.padding.sm"));
        let padding = Edges::all(padding_all);

        let axis_band_x = pick_metric(theme, "metric.chart.axis.band.x", Px(56.0));
        let axis_band_y = pick_metric(theme, "metric.chart.axis.band.y", Px(36.0));

        let tooltip_padding_x = pick_metric(theme, "metric.chart.tooltip.padding.x", Px(8.0));
        let tooltip_padding_y = pick_metric(theme, "metric.chart.tooltip.padding.y", Px(6.0));
        let tooltip_corner_radius = pick_metric(
            theme,
            "metric.chart.tooltip.corner_radius",
            theme.metric_required("metric.radius.sm"),
        );

        let legend_padding_x = pick_metric(theme, "metric.chart.legend.padding.x", Px(10.0));
        let legend_padding_y = pick_metric(theme, "metric.chart.legend.padding.y", Px(8.0));
        let legend_corner_radius = pick_metric(
            theme,
            "metric.chart.legend.corner_radius",
            theme.metric_required("metric.radius.md"),
        );
        let legend_item_gap = pick_metric(theme, "metric.chart.legend.item.gap", Px(4.0));
        let legend_swatch_size = pick_metric(theme, "metric.chart.legend.swatch.size", Px(10.0));
        let legend_swatch_gap = pick_metric(theme, "metric.chart.legend.swatch.gap", Px(8.0));

        let legend_hover_background = pick_color(
            theme,
            "chart.legend.hover.background",
            with_alpha(foreground, 0.06),
        );

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
            draw_order: DrawOrder(100),
        }
    }
}
