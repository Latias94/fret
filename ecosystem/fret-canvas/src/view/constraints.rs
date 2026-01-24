use fret_core::{Point, Px, Rect, Size};

use super::PanZoom2D;

pub const DEFAULT_MIN_ZOOM: f32 = 0.05;
pub const DEFAULT_MAX_ZOOM: f32 = 64.0;
pub const DEFAULT_FIT_PADDING_SCREEN_PX: f32 = 24.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitMode {
    /// Fits the entire target rect inside the viewport.
    Contain,
    /// Fills the viewport (the target rect may be cropped).
    Cover,
}

impl Default for FitMode {
    fn default() -> Self {
        Self::Contain
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitViewOptions2D {
    pub mode: FitMode,
    pub padding_screen_px: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for FitViewOptions2D {
    fn default() -> Self {
        Self {
            mode: FitMode::Contain,
            padding_screen_px: DEFAULT_FIT_PADDING_SCREEN_PX,
            min_zoom: DEFAULT_MIN_ZOOM,
            max_zoom: DEFAULT_MAX_ZOOM,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanZoomConstraints2D {
    pub min_zoom: f32,
    pub max_zoom: f32,
    /// Optional canvas-space extent used to clamp panning.
    pub translate_extent_canvas: Option<Rect>,
}

impl Default for PanZoomConstraints2D {
    fn default() -> Self {
        Self {
            min_zoom: DEFAULT_MIN_ZOOM,
            max_zoom: DEFAULT_MAX_ZOOM,
            translate_extent_canvas: None,
        }
    }
}

impl PanZoomConstraints2D {
    /// A generally useful baseline for editor surfaces:
    /// - a wide zoom range,
    /// - no implicit translate extent.
    pub fn editor_default() -> Self {
        Self::default()
    }
}

fn rect_min_max(rect: Rect) -> (f32, f32, f32, f32) {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;
    (x0.min(x1), x0.max(x1), y0.min(y1), y0.max(y1))
}

fn rect_center(rect: Rect) -> (f32, f32) {
    let (min_x, max_x, min_y, max_y) = rect_min_max(rect);
    ((min_x + max_x) * 0.5, (min_y + max_y) * 0.5)
}

fn sanitize_padding_screen_px(padding_screen_px: f32) -> f32 {
    if padding_screen_px.is_finite() {
        padding_screen_px.max(0.0)
    } else {
        0.0
    }
}

/// Returns the visible canvas-space rect for a widget with `bounds` and a `PanZoom2D` view.
pub fn visible_canvas_rect(bounds: Rect, view: PanZoom2D) -> Rect {
    let zoom = PanZoom2D::sanitize_zoom(view.zoom, 1.0).max(1.0e-6);
    let w = bounds.size.width.0 / zoom;
    let h = bounds.size.height.0 / zoom;
    let origin = Point::new(Px(-view.pan.x.0), Px(-view.pan.y.0));
    Rect::new(origin, Size::new(Px(w), Px(h)))
}

/// Clamps `view.zoom` to the configured range and optionally clamps `view.pan` to the extent.
pub fn clamp_pan_zoom_view(
    bounds: Rect,
    view: PanZoom2D,
    constraints: PanZoomConstraints2D,
) -> PanZoom2D {
    let mut out = view;

    let min_zoom = PanZoom2D::sanitize_zoom(constraints.min_zoom, DEFAULT_MIN_ZOOM);
    let max_zoom = PanZoom2D::sanitize_zoom(constraints.max_zoom, DEFAULT_MAX_ZOOM);
    let (min_zoom, max_zoom) = if min_zoom <= max_zoom {
        (min_zoom, max_zoom)
    } else {
        (max_zoom, min_zoom)
    };
    out.zoom = PanZoom2D::sanitize_zoom(out.zoom, 1.0).clamp(min_zoom, max_zoom);

    let Some(extent) = constraints.translate_extent_canvas else {
        return out;
    };

    let (ex_min_x, ex_max_x, ex_min_y, ex_max_y) = rect_min_max(extent);
    if !(ex_min_x.is_finite()
        && ex_max_x.is_finite()
        && ex_min_y.is_finite()
        && ex_max_y.is_finite()
        && ex_min_x <= ex_max_x
        && ex_min_y <= ex_max_y)
    {
        return out;
    }

    let vis = visible_canvas_rect(bounds, out);
    let (vis_min_x, vis_max_x, vis_min_y, vis_max_y) = rect_min_max(vis);
    let vis_w = vis_max_x - vis_min_x;
    let vis_h = vis_max_y - vis_min_y;

    if !vis_w.is_finite() || !vis_h.is_finite() || vis_w <= 0.0 || vis_h <= 0.0 {
        return out;
    }

    let allowed_pan_min_x = vis_w - ex_max_x;
    let allowed_pan_max_x = -ex_min_x;
    let allowed_pan_min_y = vis_h - ex_max_y;
    let allowed_pan_max_y = -ex_min_y;

    let mut pan_x = out.pan.x.0;
    let mut pan_y = out.pan.y.0;

    if !(pan_x.is_finite() && pan_y.is_finite()) {
        pan_x = 0.0;
        pan_y = 0.0;
    }

    // If the viewport is larger than the extent, center the extent.
    if allowed_pan_min_x > allowed_pan_max_x {
        let (cx, _cy) = rect_center(extent);
        pan_x = vis_w * 0.5 - cx;
    } else {
        pan_x = pan_x.clamp(allowed_pan_min_x, allowed_pan_max_x);
    }

    if allowed_pan_min_y > allowed_pan_max_y {
        let (_cx, cy) = rect_center(extent);
        pan_y = vis_h * 0.5 - cy;
    } else {
        pan_y = pan_y.clamp(allowed_pan_min_y, allowed_pan_max_y);
    }

    out.pan = Point::new(Px(pan_x), Px(pan_y));
    out
}

/// Computes a `PanZoom2D` view that fits `target_canvas_rect` into the widget `bounds`.
///
/// This is headless math only; callers decide when/how to apply it.
pub fn fit_view_to_canvas_rect(
    bounds: Rect,
    target_canvas_rect: Rect,
    options: FitViewOptions2D,
) -> PanZoom2D {
    let (tmin_x, tmax_x, tmin_y, tmax_y) = rect_min_max(target_canvas_rect);
    let tw = (tmax_x - tmin_x).max(1.0e-6);
    let th = (tmax_y - tmin_y).max(1.0e-6);
    if !(tw.is_finite() && th.is_finite()) {
        return PanZoom2D::default();
    }

    let pad = sanitize_padding_screen_px(options.padding_screen_px);
    let avail_w = (bounds.size.width.0 - 2.0 * pad).max(1.0);
    let avail_h = (bounds.size.height.0 - 2.0 * pad).max(1.0);

    let zx = avail_w / tw;
    let zy = avail_h / th;
    let mut zoom = match options.mode {
        FitMode::Contain => zx.min(zy),
        FitMode::Cover => zx.max(zy),
    };
    if !zoom.is_finite() || zoom <= 0.0 {
        zoom = 1.0;
    }

    let min_zoom = PanZoom2D::sanitize_zoom(options.min_zoom, DEFAULT_MIN_ZOOM);
    let max_zoom = PanZoom2D::sanitize_zoom(options.max_zoom, DEFAULT_MAX_ZOOM);
    let (min_zoom, max_zoom) = if min_zoom <= max_zoom {
        (min_zoom, max_zoom)
    } else {
        (max_zoom, min_zoom)
    };
    zoom = zoom.clamp(min_zoom, max_zoom);

    let (cx, cy) = rect_center(target_canvas_rect);
    let screen_center_x = bounds.size.width.0 * 0.5;
    let screen_center_y = bounds.size.height.0 * 0.5;
    let pan_x = screen_center_x / zoom - cx;
    let pan_y = screen_center_y / zoom - cy;

    PanZoom2D {
        pan: Point::new(Px(pan_x), Px(pan_y)),
        zoom,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect_contains(outer: Rect, inner: Rect, eps: f32) -> bool {
        let (ox0, ox1, oy0, oy1) = rect_min_max(outer);
        let (ix0, ix1, iy0, iy1) = rect_min_max(inner);
        ix0 + eps >= ox0 && ix1 - eps <= ox1 && iy0 + eps >= oy0 && iy1 - eps <= oy1
    }

    #[test]
    fn visible_canvas_rect_matches_pan_zoom_convention() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let view = PanZoom2D {
            pan: Point::new(Px(-3.0), Px(5.0)),
            zoom: 2.0,
        };

        let vis = visible_canvas_rect(bounds, view);
        assert!((vis.origin.x.0 - 3.0).abs() <= 1.0e-6);
        assert!((vis.origin.y.0 + 5.0).abs() <= 1.0e-6);
        assert!((vis.size.width.0 - 400.0).abs() <= 1.0e-6);
        assert!((vis.size.height.0 - 300.0).abs() <= 1.0e-6);
    }

    #[test]
    fn fit_view_contain_makes_target_visible() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let target = Rect::new(
            Point::new(Px(100.0), Px(200.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let view = fit_view_to_canvas_rect(bounds, target, FitViewOptions2D::default());
        let vis = visible_canvas_rect(bounds, view);
        assert!(rect_contains(vis, target, 1.0e-3));
    }

    #[test]
    fn clamp_pan_zoom_keeps_view_within_extent_when_possible() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(200.0)),
        );
        let extent = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(500.0), Px(500.0)),
        );
        let constraints = PanZoomConstraints2D {
            translate_extent_canvas: Some(extent),
            ..PanZoomConstraints2D::default()
        };

        let view = PanZoom2D {
            pan: Point::new(Px(10_000.0), Px(10_000.0)),
            zoom: 1.0,
        };
        let clamped = clamp_pan_zoom_view(bounds, view, constraints);
        let vis = visible_canvas_rect(bounds, clamped);
        assert!(rect_contains(extent, vis, 1.0e-3));
    }
}
