use fret_core::{Px, Size};

pub(super) struct ResizeDragBounds {
    min: Size,
    max: Option<Size>,
}

impl ResizeDragBounds {
    pub(super) fn new(min: Size, max: Option<Size>) -> Self {
        Self { min, max }
    }

    pub(super) fn clamp_width(&self, value: f32) -> Px {
        let mut out = value.max(self.min.width.0);
        if let Some(max) = self.max {
            out = out.min(max.width.0);
        }
        Px(out)
    }

    pub(super) fn clamp_height(&self, value: f32) -> Px {
        let mut out = value.max(self.min.height.0);
        if let Some(max) = self.max {
            out = out.min(max.height.0);
        }
        Px(out)
    }
}
