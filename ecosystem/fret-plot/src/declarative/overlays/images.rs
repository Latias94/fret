//! Declarative line-plot image overlay paint owner.

use fret_core::{DrawOrder, Point, Px, Rect, SceneOp, Size};
use fret_ui::canvas::CanvasPainter;

use crate::cartesian::{AxisScale, DataPoint, DataRect, PlotTransform};
use crate::models::YAxis;
use crate::state::{PlotImageLayer, PlotOverlays};

use super::super::geometry::line_plot_view_bounds_for_y_axis;

pub(in crate::declarative) fn paint_line_plot_images(
    painter: &mut CanvasPainter<'_>,
    plot: Rect,
    view_bounds: DataRect,
    view_bounds_y2: Option<DataRect>,
    view_bounds_y3: Option<DataRect>,
    view_bounds_y4: Option<DataRect>,
    overlays: &PlotOverlays,
    layer: PlotImageLayer,
    x_scale: AxisScale,
    y_scale: AxisScale,
) {
    if overlays.images.is_empty() {
        return;
    }
    let Some(transform) = (PlotTransform {
        viewport: plot,
        data: view_bounds,
        x_scale,
        y_scale,
    })
    .prepare() else {
        return;
    };
    let transform_y2 = view_bounds_y2.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y3 = view_bounds_y3.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });
    let transform_y4 = view_bounds_y4.and_then(|axis_bounds| {
        (PlotTransform {
            viewport: plot,
            data: line_plot_view_bounds_for_y_axis(view_bounds, axis_bounds),
            x_scale,
            y_scale,
        })
        .prepare()
    });

    painter.with_clip_rect(plot, |painter| {
        for image in overlays.images.iter().filter(|image| image.layer == layer) {
            let transform = match image.axis {
                YAxis::Left => transform,
                YAxis::Right => transform_y2.unwrap_or(transform),
                YAxis::Right2 => transform_y3.unwrap_or(transform),
                YAxis::Right3 => transform_y4.unwrap_or(transform),
            };
            let rect = image.rect;
            if !rect.x_min.is_finite()
                || !rect.x_max.is_finite()
                || !rect.y_min.is_finite()
                || !rect.y_max.is_finite()
            {
                continue;
            }

            let a = transform.data_to_px(DataPoint {
                x: rect.x_min,
                y: rect.y_min,
            });
            let b = transform.data_to_px(DataPoint {
                x: rect.x_max,
                y: rect.y_max,
            });
            if !a.x.0.is_finite() || !a.y.0.is_finite() || !b.x.0.is_finite() || !b.y.0.is_finite()
            {
                continue;
            }

            let left = a.x.0.min(b.x.0);
            let right = a.x.0.max(b.x.0);
            let top = a.y.0.min(b.y.0);
            let bottom = a.y.0.max(b.y.0);
            let width = (right - left).max(0.0);
            let height = (bottom - top).max(0.0);
            if width <= 0.0 || height <= 0.0 {
                continue;
            }

            let opacity = image.opacity.clamp(0.0, 1.0);
            if opacity <= 0.0 {
                continue;
            }

            painter.scene().push(SceneOp::ImageRegion {
                order: DrawOrder(1),
                rect: Rect::new(
                    Point::new(Px(left), Px(top)),
                    Size::new(Px(width), Px(height)),
                ),
                image: image.image,
                uv: image.uv,
                sampling: fret_core::scene::ImageSamplingHint::Default,
                opacity,
            });
        }
    });
}
