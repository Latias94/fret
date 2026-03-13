use super::nudge_support;
use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn nudge_selection_by_screen_delta<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        delta_screen_px: CanvasPoint,
    ) {
        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }

        let delta = CanvasPoint {
            x: fret_canvas::scale::canvas_units_from_screen_px(delta_screen_px.x, zoom),
            y: fret_canvas::scale::canvas_units_from_screen_px(delta_screen_px.y, zoom),
        };
        self.nudge_selection_by_canvas_delta(
            host,
            window,
            snapshot,
            delta,
            snapshot.interaction.snap_to_grid,
        );
    }

    pub(in super::super) fn nudge_selection_by_grid_step<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        steps: CanvasPoint,
    ) {
        let grid = snapshot.interaction.snap_grid;
        let delta = CanvasPoint {
            x: steps.x * grid.width,
            y: steps.y * grid.height,
        };
        self.nudge_selection_by_canvas_delta(host, window, snapshot, delta, true);
    }

    fn nudge_selection_by_canvas_delta<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        mut delta: CanvasPoint,
        snap_to_grid: bool,
    ) {
        let selected_nodes = snapshot.selected_nodes.clone();
        let selected_groups = snapshot.selected_groups.clone();
        if selected_nodes.is_empty() && selected_groups.is_empty() {
            return;
        }

        if !delta.x.is_finite() || !delta.y.is_finite() {
            return;
        }

        if snap_to_grid {
            if let Some(primary) = selected_nodes.first().copied() {
                let primary_start = self
                    .graph
                    .read_ref(host, |g| g.nodes.get(&primary).map(|n| n.pos))
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                let primary_target = CanvasPoint {
                    x: primary_start.x + delta.x,
                    y: primary_start.y + delta.y,
                };
                let snapped =
                    Self::snap_canvas_point(primary_target, snapshot.interaction.snap_grid);
                delta = CanvasPoint {
                    x: snapped.x - primary_start.x,
                    y: snapped.y - primary_start.y,
                };
            } else if let Some(primary) = selected_groups.first().copied() {
                let primary_start = self
                    .graph
                    .read_ref(host, |g| g.groups.get(&primary).map(|gr| gr.rect.origin))
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                let primary_target = CanvasPoint {
                    x: primary_start.x + delta.x,
                    y: primary_start.y + delta.y,
                };
                let snapped =
                    Self::snap_canvas_point(primary_target, snapshot.interaction.snap_grid);
                delta = CanvasPoint {
                    x: snapped.x - primary_start.x,
                    y: snapped.y - primary_start.y,
                };
            }
        }

        if delta.x.abs() <= 1.0e-9 && delta.y.abs() <= 1.0e-9 {
            return;
        }

        let geom_for_extent = self.canvas_geometry(&*host, snapshot);
        let ops = self
            .graph
            .read_ref(host, |g| {
                nudge_support::plan_nudge_ops(
                    g,
                    geom_for_extent.as_ref(),
                    &selected_nodes,
                    &selected_groups,
                    snapshot,
                    delta,
                )
            })
            .ok()
            .unwrap_or_default();

        if ops.is_empty() {
            return;
        }

        let _ = self.commit_ops(host, window, Some("Nudge"), ops);
    }
}
