use fret_core::{Color, DrawOrder, Px};

#[derive(Debug, Clone, Copy)]
pub struct ChartStyle {
    pub background: Option<Color>,
    pub stroke_color: Color,
    pub stroke_width: Px,
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
            draw_order: DrawOrder(100),
        }
    }
}
