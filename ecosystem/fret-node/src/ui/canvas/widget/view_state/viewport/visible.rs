use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super::super) fn ensure_canvas_point_visible<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        point: CanvasPoint,
    ) {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
            return;
        }

        let margin_screen = 24.0f32;
        let margin = margin_screen / zoom;
        if !margin.is_finite() {
            return;
        }

        let viewport = Self::viewport_from_pan_zoom(bounds, snapshot.pan, zoom);
        let vis = viewport.visible_canvas_rect();
        let view_w = vis.size.width.0;
        let view_h = vis.size.height.0;

        let view_min_x = vis.origin.x.0;
        let view_min_y = vis.origin.y.0;

        let mut pan = snapshot.pan;

        let min_x = view_min_x + margin;
        let max_x = view_min_x + view_w - margin;
        if point.x < min_x {
            pan.x = margin - point.x;
        } else if point.x > max_x {
            pan.x = (view_w - margin) - point.x;
        }

        let min_y = view_min_y + margin;
        let max_y = view_min_y + view_h - margin;
        if point.y < min_y {
            pan.y = margin - point.y;
        } else if point.y > max_y {
            pan.y = (view_h - margin) - point.y;
        }

        if pan != snapshot.pan {
            self.update_view_state(host, |s| {
                s.pan = pan;
            });
        }
    }
}
