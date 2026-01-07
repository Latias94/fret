use fret_core::{Color, DrawOrder, Edges, Px};

#[derive(Debug, Clone, Copy)]
pub struct ChartStyle {
    pub background: Option<Color>,
    pub stroke_color: Color,
    pub stroke_width: Px,
    pub area_fill_color: Color,
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
    pub draw_order: DrawOrder,
}

impl Default for ChartStyle {
    fn default() -> Self {
        Self {
            background: None,
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
            draw_order: DrawOrder(100),
        }
    }
}
