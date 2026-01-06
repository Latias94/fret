use fret_core::geometry::{Point, Px, Rect, Size};

use super::YAxis;
use super::canvas::contains_point;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotLayout {
    pub(crate) plot: Rect,
    pub(crate) y_axis_left: Rect,
    pub(crate) y_axis_right: Rect,
    pub(crate) y_axis_right2: Rect,
    pub(crate) y_axis_right3: Rect,
    pub(crate) x_axis: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlotRegion {
    Plot,
    XAxis,
    YAxis(YAxis),
}

impl PlotLayout {
    pub(crate) fn from_bounds(
        bounds: Rect,
        padding: Px,
        y_axis_left_gap: Px,
        y_axis_right_gap: Px,
        y_axis_right2_gap: Px,
        y_axis_right3_gap: Px,
        x_axis_gap: Px,
    ) -> Self {
        let pad = padding.0.max(0.0);
        let y_axis_left_gap = y_axis_left_gap.0.max(0.0);
        let y_axis_right_gap = y_axis_right_gap.0.max(0.0);
        let y_axis_right2_gap = y_axis_right2_gap.0.max(0.0);
        let y_axis_right3_gap = y_axis_right3_gap.0.max(0.0);
        let x_axis_gap = x_axis_gap.0.max(0.0);

        let content = Rect::new(
            Point::new(Px(bounds.origin.x.0 + pad), Px(bounds.origin.y.0 + pad)),
            Size::new(
                Px((bounds.size.width.0 - pad * 2.0).max(0.0)),
                Px((bounds.size.height.0 - pad * 2.0).max(0.0)),
            ),
        );

        let plot_w = (content.size.width.0
            - y_axis_left_gap
            - y_axis_right_gap
            - y_axis_right2_gap
            - y_axis_right3_gap)
            .max(0.0);
        let plot_h = (content.size.height.0 - x_axis_gap).max(0.0);

        let plot = Rect::new(
            Point::new(Px(content.origin.x.0 + y_axis_left_gap), content.origin.y),
            Size::new(Px(plot_w), Px(plot_h)),
        );

        let y_axis_left = Rect::new(content.origin, Size::new(Px(y_axis_left_gap), Px(plot_h)));

        let y_axis_right = Rect::new(
            Point::new(Px(plot.origin.x.0 + plot.size.width.0), plot.origin.y),
            Size::new(Px(y_axis_right_gap), Px(plot_h)),
        );
        let y_axis_right2 = Rect::new(
            Point::new(
                Px(y_axis_right.origin.x.0 + y_axis_right.size.width.0),
                plot.origin.y,
            ),
            Size::new(Px(y_axis_right2_gap), Px(plot_h)),
        );
        let y_axis_right3 = Rect::new(
            Point::new(
                Px(y_axis_right2.origin.x.0 + y_axis_right2.size.width.0),
                plot.origin.y,
            ),
            Size::new(Px(y_axis_right3_gap), Px(plot_h)),
        );

        let x_axis = Rect::new(
            Point::new(plot.origin.x, Px(plot.origin.y.0 + plot.size.height.0)),
            Size::new(Px(plot_w), Px(x_axis_gap)),
        );

        Self {
            plot,
            y_axis_left,
            y_axis_right,
            y_axis_right2,
            y_axis_right3,
            x_axis,
        }
    }

    pub(crate) fn hit_test_region(&self, position: Point) -> Option<PlotRegion> {
        if contains_point(self.plot, position) {
            return Some(PlotRegion::Plot);
        }
        if contains_point(self.x_axis, position) {
            return Some(PlotRegion::XAxis);
        }
        if contains_point(self.y_axis_left, position) {
            return Some(PlotRegion::YAxis(YAxis::Left));
        }
        if contains_point(self.y_axis_right, position) {
            return Some(PlotRegion::YAxis(YAxis::Right));
        }
        if contains_point(self.y_axis_right2, position) {
            return Some(PlotRegion::YAxis(YAxis::Right2));
        }
        if contains_point(self.y_axis_right3, position) {
            return Some(PlotRegion::YAxis(YAxis::Right3));
        }
        None
    }
}
