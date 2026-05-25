use fret_core::Px;

use super::super::drag::DragResponse;

#[derive(Debug, Clone, Default)]
pub struct ChildRegionResponse {
    pub(crate) resize_x: ChildRegionResizeXResponse,
    pub(crate) resize_y: ChildRegionResizeYResponse,
}

#[derive(Debug, Clone, Default)]
pub struct ChildRegionResizeXResponse {
    pub(crate) enabled: bool,
    pub(crate) min_width: Option<Px>,
    pub(crate) max_width: Option<Px>,
    pub(crate) drag: DragResponse,
}

#[derive(Debug, Clone, Default)]
pub struct ChildRegionResizeYResponse {
    pub(crate) enabled: bool,
    pub(crate) min_height: Option<Px>,
    pub(crate) max_height: Option<Px>,
    pub(crate) drag: DragResponse,
}

impl ChildRegionResponse {
    pub(crate) fn empty() -> Self {
        Self::default()
    }

    pub(crate) fn resize_y_mut(&mut self) -> &mut ChildRegionResizeYResponse {
        &mut self.resize_y
    }

    pub(crate) fn resize_x_mut(&mut self) -> &mut ChildRegionResizeXResponse {
        &mut self.resize_x
    }

    pub fn resize_x(&self) -> &ChildRegionResizeXResponse {
        &self.resize_x
    }

    pub fn resize_y(&self) -> &ChildRegionResizeYResponse {
        &self.resize_y
    }

    pub fn resizing_x(&self) -> bool {
        self.resize_x.dragging()
    }

    pub fn resizing_y(&self) -> bool {
        self.resize_y.dragging()
    }
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
    use fret_core::{Point, Px};

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
