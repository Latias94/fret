use fret_core::{Color, DrawOrder, Edges, Px};

#[derive(Debug, Clone, Copy)]
pub struct ChartStyle {
    pub background: Option<Color>,
    pub stroke_color: Color,
    pub stroke_width: Px,
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
            draw_order: DrawOrder(100),
        }
    }
}
