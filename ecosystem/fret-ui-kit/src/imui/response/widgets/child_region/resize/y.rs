use fret_core::Px;

use super::super::super::super::drag::DragResponse;

#[derive(Debug, Clone, Default)]
pub struct ChildRegionResizeYResponse {
    pub(crate) enabled: bool,
    pub(crate) min_height: Option<Px>,
    pub(crate) max_height: Option<Px>,
    pub(crate) drag: DragResponse,
}

impl ChildRegionResizeYResponse {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn min_height(&self) -> Option<Px> {
        self.min_height
    }

    pub fn max_height(&self) -> Option<Px> {
        self.max_height
    }

    pub fn dragging(&self) -> bool {
        self.drag.dragging
    }

    pub fn drag_started(&self) -> bool {
        self.drag.started
    }

    pub fn drag_stopped(&self) -> bool {
        self.drag.stopped
    }

    pub fn drag_delta_y(&self) -> f32 {
        self.drag.delta.y.0
    }

    pub fn drag_total_y(&self) -> f32 {
        self.drag.total.y.0
    }

    pub fn height_from_start(&self, start_height: Px) -> Px {
        let min = self
            .min_height
            .map(|height| height.0)
            .unwrap_or(0.0)
            .max(0.0);
        let max = self
            .max_height
            .map(|height| height.0)
            .unwrap_or(f32::INFINITY);
        Px((start_height.0 + self.drag_total_y()).clamp(min, max.max(min)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Point;

    #[test]
    fn child_region_resize_y_height_from_start_clamps_to_min_and_max() {
        let mut response = ChildRegionResizeYResponse {
            min_height: Some(Px(48.0)),
            max_height: Some(Px(160.0)),
            ..Default::default()
        };

        response
            .drag
            .set_motion(Point::new(Px(0.0), Px(0.0)), Point::new(Px(0.0), Px(24.0)));
        assert_eq!(response.height_from_start(Px(100.0)), Px(124.0));

        response.drag.set_motion(
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(0.0), Px(-120.0)),
        );
        assert_eq!(response.height_from_start(Px(100.0)), Px(48.0));

        response
            .drag
            .set_motion(Point::new(Px(0.0), Px(0.0)), Point::new(Px(0.0), Px(120.0)));
        assert_eq!(response.height_from_start(Px(100.0)), Px(160.0));
    }
}
