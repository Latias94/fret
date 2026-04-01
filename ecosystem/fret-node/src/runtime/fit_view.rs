//! Fit-view helpers (XyFlow-style viewport framing).
//!
//! This module is intentionally headless-safe and does not depend on `fret-ui`.

use crate::core::CanvasPoint;
use fret_core::Rect;

#[derive(Debug, Clone, Copy)]
pub struct FitViewComputeOptions {
    /// Viewport width in logical px.
    pub viewport_width_px: f32,
    /// Viewport height in logical px.
    pub viewport_height_px: f32,
    /// Node origin (anchor) used to interpret `FitViewNodeInfo.pos`.
    ///
    /// When `(0.0, 0.0)`, `pos` is treated as the node's top-left.
    /// When `(0.5, 0.5)`, `pos` is treated as the node's center.
    pub node_origin: (f32, f32),
    /// Extra padding as a fraction of viewport size (0.0 .. 0.45 recommended).
    ///
    /// When `0.0`, `margin_px_fallback` is used instead.
    pub padding: f32,
    /// Fixed margin in logical px used when `padding == 0.0`.
    pub margin_px_fallback: f32,
    /// Minimum zoom clamp.
    pub min_zoom: f32,
    /// Maximum zoom clamp.
    pub max_zoom: f32,
}

impl FitViewComputeOptions {
    pub fn normalized(mut self) -> Option<Self> {
        if !self.viewport_width_px.is_finite()
            || !self.viewport_height_px.is_finite()
            || self.viewport_width_px <= 1.0
            || self.viewport_height_px <= 1.0
        {
            return None;
        }

        if !self.node_origin.0.is_finite() {
            self.node_origin.0 = 0.0;
        }
        if !self.node_origin.1.is_finite() {
            self.node_origin.1 = 0.0;
        }
        self.node_origin.0 = self.node_origin.0.clamp(0.0, 1.0);
        self.node_origin.1 = self.node_origin.1.clamp(0.0, 1.0);

        if !self.margin_px_fallback.is_finite() || self.margin_px_fallback < 0.0 {
            self.margin_px_fallback = 0.0;
        }

        if !self.padding.is_finite() {
            self.padding = 0.0;
        }
        self.padding = self.padding.clamp(0.0, 0.45);

        if !self.min_zoom.is_finite() || self.min_zoom <= 0.0 {
            self.min_zoom = 1.0;
        }
        if !self.max_zoom.is_finite() || self.max_zoom <= 0.0 {
            self.max_zoom = 1.0;
        }
        if self.min_zoom > self.max_zoom {
            std::mem::swap(&mut self.min_zoom, &mut self.max_zoom);
        }

        Some(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FitViewNodeInfo {
    pub pos: CanvasPoint,
    /// Node size in logical px at zoom=1 (semantic zoom sizing).
    pub size_px: (f32, f32),
}

/// Computes the viewport pan/zoom that frames the given nodes in view.
///
/// The returned pan/zoom matches the UI contract: `pan` is in canvas space (added in the render
/// transform) and `zoom` is a scalar.
pub fn compute_fit_view_target(
    nodes: &[FitViewNodeInfo],
    options: FitViewComputeOptions,
) -> Option<(CanvasPoint, f32)> {
    let options = options.normalized()?;
    if nodes.is_empty() {
        return None;
    }

    fn compute_fit_view_target_top_left(
        nodes: &[FitViewNodeInfo],
        options: FitViewComputeOptions,
    ) -> Option<(CanvasPoint, f32)> {
        let (viewport_w, viewport_h) = (options.viewport_width_px, options.viewport_height_px);
        let (margin_x, margin_y) = if options.padding > 0.0 {
            (viewport_w * options.padding, viewport_h * options.padding)
        } else {
            (options.margin_px_fallback, options.margin_px_fallback)
        };

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_w = 0.0f32;
        let mut max_h = 0.0f32;

        for n in nodes {
            let (w, h) = n.size_px;
            if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
                continue;
            }
            min_x = min_x.min(n.pos.x);
            min_y = min_y.min(n.pos.y);
            max_x = max_x.max(n.pos.x);
            max_y = max_y.max(n.pos.y);
            max_w = max_w.max(w);
            max_h = max_h.max(h);
        }

        if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
            return None;
        }

        let spread_x = (max_x - min_x).max(0.0);
        let spread_y = (max_y - min_y).max(0.0);

        let mut zoom_x = options.max_zoom;
        let mut zoom_y = options.max_zoom;
        if spread_x > 1.0e-3 {
            zoom_x = (viewport_w - max_w - 2.0 * margin_x) / spread_x;
        }
        if spread_y > 1.0e-3 {
            zoom_y = (viewport_h - max_h - 2.0 * margin_y) / spread_y;
        }

        let mut zoom = zoom_x.min(zoom_y);
        if !zoom.is_finite() {
            zoom = 1.0;
        }
        zoom = zoom.clamp(options.min_zoom, options.max_zoom);

        let mut rect_min_x = f32::INFINITY;
        let mut rect_min_y = f32::INFINITY;
        let mut rect_max_x = f32::NEG_INFINITY;
        let mut rect_max_y = f32::NEG_INFINITY;
        for n in nodes {
            let (w_px, h_px) = n.size_px;
            if !w_px.is_finite() || !h_px.is_finite() || w_px <= 0.0 || h_px <= 0.0 {
                continue;
            }
            let w = w_px / zoom;
            let h = h_px / zoom;
            rect_min_x = rect_min_x.min(n.pos.x);
            rect_min_y = rect_min_y.min(n.pos.y);
            rect_max_x = rect_max_x.max(n.pos.x + w);
            rect_max_y = rect_max_y.max(n.pos.y + h);
        }

        if !rect_min_x.is_finite()
            || !rect_min_y.is_finite()
            || !rect_max_x.is_finite()
            || !rect_max_y.is_finite()
        {
            return None;
        }

        let center_x = 0.5 * (rect_min_x + rect_max_x);
        let center_y = 0.5 * (rect_min_y + rect_max_y);

        let viewport_w_canvas = viewport_w / zoom;
        let viewport_h_canvas = viewport_h / zoom;
        let target_center_x = 0.5 * viewport_w_canvas;
        let target_center_y = 0.5 * viewport_h_canvas;

        let pan = CanvasPoint {
            x: target_center_x - center_x,
            y: target_center_y - center_y,
        };

        Some((pan, zoom))
    }

    let (viewport_w, viewport_h) = (options.viewport_width_px, options.viewport_height_px);
    if !viewport_w.is_finite() || !viewport_h.is_finite() {
        return None;
    }

    let mut zoom_guess = options.max_zoom;
    let mut best: Option<(CanvasPoint, f32)> = None;

    for _ in 0..4 {
        if !zoom_guess.is_finite() || zoom_guess <= 0.0 {
            zoom_guess = 1.0;
        }
        zoom_guess = zoom_guess.clamp(options.min_zoom, options.max_zoom);

        let mut scratch: Vec<FitViewNodeInfo> = Vec::with_capacity(nodes.len());
        let ox = options.node_origin.0;
        let oy = options.node_origin.1;
        for n in nodes {
            let (w_px, h_px) = n.size_px;
            if !w_px.is_finite() || !h_px.is_finite() || w_px <= 0.0 || h_px <= 0.0 {
                continue;
            }
            let w_canvas = w_px / zoom_guess;
            let h_canvas = h_px / zoom_guess;
            scratch.push(FitViewNodeInfo {
                pos: CanvasPoint {
                    x: n.pos.x - ox * w_canvas,
                    y: n.pos.y - oy * h_canvas,
                },
                size_px: n.size_px,
            });
        }

        let Some((pan, zoom_next)) = compute_fit_view_target_top_left(&scratch, options) else {
            return best;
        };
        best = Some((pan, zoom_next));
        zoom_guess = zoom_next;
    }

    best
}

/// Computes the viewport pan/zoom that frames the given canvas-space rect in view.
pub fn compute_fit_view_target_for_canvas_rect(
    target_canvas: Rect,
    options: FitViewComputeOptions,
) -> Option<(CanvasPoint, f32)> {
    let options = options.normalized()?;
    if !target_canvas.size.width.0.is_finite()
        || !target_canvas.size.height.0.is_finite()
        || target_canvas.size.width.0 <= 0.0
        || target_canvas.size.height.0 <= 0.0
        || !target_canvas.origin.x.0.is_finite()
        || !target_canvas.origin.y.0.is_finite()
    {
        return None;
    }

    let (viewport_w, viewport_h) = (options.viewport_width_px, options.viewport_height_px);
    let (margin_x, margin_y) = if options.padding > 0.0 {
        (viewport_w * options.padding, viewport_h * options.padding)
    } else {
        (options.margin_px_fallback, options.margin_px_fallback)
    };

    let zoom_x = (viewport_w - 2.0 * margin_x) / target_canvas.size.width.0;
    let zoom_y = (viewport_h - 2.0 * margin_y) / target_canvas.size.height.0;
    if !zoom_x.is_finite() || !zoom_y.is_finite() {
        return None;
    }

    let zoom = zoom_x.min(zoom_y).clamp(options.min_zoom, options.max_zoom);
    if !zoom.is_finite() || zoom <= 0.0 {
        return None;
    }

    let center_x = target_canvas.origin.x.0 + 0.5 * target_canvas.size.width.0;
    let center_y = target_canvas.origin.y.0 + 0.5 * target_canvas.size.height.0;
    let pan = CanvasPoint {
        x: 0.5 * viewport_w / zoom - center_x,
        y: 0.5 * viewport_h / zoom - center_y,
    };
    if !pan.x.is_finite() || !pan.y.is_finite() {
        return None;
    }

    Some((pan, zoom))
}

#[cfg(test)]
mod tests {
    use super::{
        FitViewComputeOptions, FitViewNodeInfo, compute_fit_view_target,
        compute_fit_view_target_for_canvas_rect,
    };
    use crate::core::CanvasPoint;
    use fret_core::{Point, Px, Rect, Size};

    #[test]
    fn compute_fit_view_target_returns_valid_viewport() {
        let nodes = [
            FitViewNodeInfo {
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                size_px: (200.0, 100.0),
            },
            FitViewNodeInfo {
                pos: CanvasPoint { x: 1000.0, y: 0.0 },
                size_px: (200.0, 100.0),
            },
        ];

        let (pan, zoom) = compute_fit_view_target(
            &nodes,
            FitViewComputeOptions {
                viewport_width_px: 800.0,
                viewport_height_px: 600.0,
                node_origin: (0.0, 0.0),
                padding: 0.1,
                margin_px_fallback: 48.0,
                min_zoom: 0.1,
                max_zoom: 4.0,
            },
        )
        .expect("target");

        assert!(pan.x.is_finite() && pan.y.is_finite());
        assert!(zoom.is_finite() && zoom > 0.0);
    }

    #[test]
    fn compute_fit_view_target_for_canvas_rect_returns_valid_viewport() {
        let (pan, zoom) = compute_fit_view_target_for_canvas_rect(
            Rect::new(
                Point::new(Px(100.0), Px(50.0)),
                Size::new(Px(400.0), Px(200.0)),
            ),
            FitViewComputeOptions {
                viewport_width_px: 800.0,
                viewport_height_px: 600.0,
                node_origin: (0.0, 0.0),
                padding: 0.0,
                margin_px_fallback: 24.0,
                min_zoom: 0.1,
                max_zoom: 4.0,
            },
        )
        .expect("target");

        assert!((zoom - 1.88).abs() <= 1.0e-6);
        assert!((pan.x - (-87.23404)).abs() <= 1.0e-4);
        assert!((pan.y - (9.574471)).abs() <= 1.0e-4);
    }
}
