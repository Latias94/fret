use fret_core::geometry::{Point, Px, Rect};

#[derive(Debug, Clone, Copy)]
pub struct GridLines {
    pub x: &'static [f32],
    pub y: &'static [f32],
}

impl Default for GridLines {
    fn default() -> Self {
        // Normalized tick positions in [0, 1].
        Self {
            x: &[0.0, 0.25, 0.5, 0.75, 1.0],
            y: &[0.0, 0.25, 0.5, 0.75, 1.0],
        }
    }
}

impl GridLines {
    pub fn x_lines(self, plot: Rect) -> impl Iterator<Item = (Point, Point)> {
        self.x.iter().copied().filter_map(move |t| {
            if !t.is_finite() {
                return None;
            }
            let x = plot.origin.x.0 + plot.size.width.0 * t;
            let x = Px(x);
            Some((
                Point::new(x, plot.origin.y),
                Point::new(x, Px(plot.origin.y.0 + plot.size.height.0)),
            ))
        })
    }

    pub fn y_lines(self, plot: Rect) -> impl Iterator<Item = (Point, Point)> {
        self.y.iter().copied().filter_map(move |t| {
            if !t.is_finite() {
                return None;
            }
            let y = plot.origin.y.0 + plot.size.height.0 * t;
            let y = Px(y);
            Some((
                Point::new(plot.origin.x, y),
                Point::new(Px(plot.origin.x.0 + plot.size.width.0), y),
            ))
        })
    }
}
