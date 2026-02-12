use std::collections::HashMap;

use fret_core::time::Duration;
use fret_core::{AppWindowId, FrameId, Point, PointerId};

#[derive(Debug, Clone, Copy)]
pub struct PointerMotionSnapshot {
    pub position_window: Point,
    pub velocity_window: Option<Point>,
    #[allow(dead_code)]
    pub updated_frame_id: FrameId,
    pub updated_at: Option<Duration>,
}

#[derive(Debug, Default)]
pub(crate) struct WindowPointerMotionService {
    windows: HashMap<AppWindowId, HashMap<PointerId, PointerMotionSnapshot>>,
}

impl WindowPointerMotionService {
    pub fn update_position(
        &mut self,
        window: AppWindowId,
        pointer_id: PointerId,
        position_window: Point,
        frame_id: FrameId,
        now_monotonic: Option<Duration>,
    ) {
        let pointers = self.windows.entry(window).or_default();
        let prev = pointers.get(&pointer_id).copied();

        let velocity_window = if let Some(prev) = prev
            && let Some(prev_at) = prev.updated_at
            && let Some(now_at) = now_monotonic
        {
            let dt = now_at
                .checked_sub(prev_at)
                .unwrap_or_else(|| Duration::default());
            let dt = dt.as_secs_f32();
            if dt > 0.0 {
                let dx = position_window.x.0 - prev.position_window.x.0;
                let dy = position_window.y.0 - prev.position_window.y.0;
                Some(Point::new(fret_core::Px(dx / dt), fret_core::Px(dy / dt)))
            } else {
                None
            }
        } else {
            None
        };

        pointers.insert(
            pointer_id,
            PointerMotionSnapshot {
                position_window,
                velocity_window,
                updated_frame_id: frame_id,
                updated_at: now_monotonic,
            },
        );
    }

    pub fn snapshot(
        &self,
        window: AppWindowId,
        pointer_id: PointerId,
    ) -> Option<PointerMotionSnapshot> {
        self.windows
            .get(&window)
            .and_then(|m| m.get(&pointer_id))
            .copied()
    }

    pub fn position_window(&self, window: AppWindowId, pointer_id: PointerId) -> Option<Point> {
        self.snapshot(window, pointer_id).map(|s| s.position_window)
    }

    pub fn velocity_window(&self, window: AppWindowId, pointer_id: PointerId) -> Option<Point> {
        self.snapshot(window, pointer_id)
            .and_then(|s| s.velocity_window)
    }

    #[allow(dead_code)]
    pub fn clear_window(&mut self, window: AppWindowId) {
        self.windows.remove(&window);
    }
}
