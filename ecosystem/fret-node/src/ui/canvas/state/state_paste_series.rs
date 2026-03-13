use super::*;

impl PasteSeries {
    pub(crate) fn next(prev: Option<Self>, anchor: CanvasPoint, zoom: f32) -> (Self, CanvasPoint) {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };

        let threshold = 6.0 / zoom;
        let step = 24.0 / zoom;

        let mut count = 0u32;
        if let Some(series) = prev {
            let dx = anchor.x - series.anchor.x;
            let dy = anchor.y - series.anchor.y;
            let d2 = dx * dx + dy * dy;
            if d2.is_finite() && d2 <= threshold * threshold {
                count = series.count.saturating_add(1);
            }
        }

        let next = Self { anchor, count };
        let at = CanvasPoint {
            x: anchor.x + count as f32 * step,
            y: anchor.y + count as f32 * step,
        };
        (next, at)
    }
}
