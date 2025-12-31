use std::collections::HashMap;

use crate::{AppWindowId, Point, Rect, Size};

/// Window position in screen space, expressed in **logical pixels** (see ADR 0017).
///
/// This is intended for best-effort window placement persistence and multi-window orchestration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowLogicalPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowAnchor {
    pub window: AppWindowId,
    pub position: Point,
}

#[derive(Debug, Default, Clone)]
pub struct WindowMetricsService {
    inner_sizes: HashMap<AppWindowId, Size>,
}

impl WindowMetricsService {
    pub fn set_inner_size(&mut self, window: AppWindowId, size: Size) {
        self.inner_sizes.insert(window, size);
    }

    pub fn inner_size(&self, window: AppWindowId) -> Option<Size> {
        self.inner_sizes.get(&window).copied()
    }

    pub fn inner_bounds(&self, window: AppWindowId) -> Option<Rect> {
        let size = self.inner_size(window)?;
        Some(Rect::new(Point::new(crate::Px(0.0), crate::Px(0.0)), size))
    }

    pub fn remove(&mut self, window: AppWindowId) {
        self.inner_sizes.remove(&window);
    }
}
