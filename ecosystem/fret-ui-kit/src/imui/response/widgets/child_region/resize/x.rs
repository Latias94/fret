use fret_core::Px;

use super::super::super::super::drag::DragResponse;

#[derive(Debug, Clone, Default)]
pub struct ChildRegionResizeXResponse {
    pub(crate) enabled: bool,
    pub(crate) min_width: Option<Px>,
    pub(crate) max_width: Option<Px>,
    pub(crate) drag: DragResponse,
}

impl ChildRegionResizeXResponse {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn min_width(&self) -> Option<Px> {
        self.min_width
    }

    pub fn max_width(&self) -> Option<Px> {
        self.max_width
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

    pub fn drag_delta_x(&self) -> f32 {
        self.drag.delta.x.0
    }

    pub fn drag_total_x(&self) -> f32 {
        self.drag.total.x.0
    }

    pub fn width_from_start(&self, start_width: Px) -> Px {
        let min = self.min_width.map(|width| width.0).unwrap_or(0.0).max(0.0);
        let max = self.max_width.map(|width| width.0).unwrap_or(f32::INFINITY);
        Px((start_width.0 + self.drag_total_x()).clamp(min, max.max(min)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Point;

    #[test]
    fn child_region_resize_x_width_from_start_clamps_to_min_and_max() {
        let mut response = ChildRegionResizeXResponse {
            min_width: Some(Px(80.0)),
            max_width: Some(Px(320.0)),
            ..Default::default()
        };

        response
            .drag
            .set_motion(Point::new(Px(0.0), Px(0.0)), Point::new(Px(24.0), Px(0.0)));
        assert_eq!(response.width_from_start(Px(160.0)), Px(184.0));

        response.drag.set_motion(
            Point::new(Px(0.0), Px(0.0)),
            Point::new(Px(-120.0), Px(0.0)),
        );
        assert_eq!(response.width_from_start(Px(160.0)), Px(80.0));

        response
            .drag
            .set_motion(Point::new(Px(0.0), Px(0.0)), Point::new(Px(240.0), Px(0.0)));
        assert_eq!(response.width_from_start(Px(160.0)), Px(320.0));
    }
}
