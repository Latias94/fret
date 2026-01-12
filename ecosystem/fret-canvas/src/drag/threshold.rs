/// A drag threshold expressed in screen-space logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragThreshold {
    /// Minimum pointer distance (screen px) before a press becomes a drag.
    pub screen_px: f32,
}

impl DragThreshold {
    /// Converts the threshold to canvas-space units under a uniform zoom.
    pub fn to_canvas_units(self, zoom: f32) -> f32 {
        crate::scale::canvas_units_from_screen_px(self.screen_px, zoom)
    }
}
