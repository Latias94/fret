use fret_core::{Color, DrawOrder, Px};

#[derive(Debug, Clone, Copy)]
pub struct ChartStyle {
    pub background: Option<Color>,
    pub stroke_color: Color,
    pub stroke_width: Px,
    pub selection_fill: Color,
    pub selection_stroke: Color,
    pub selection_stroke_width: Px,
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
            draw_order: DrawOrder(100),
        }
    }
}
