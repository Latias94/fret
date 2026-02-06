use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_nudge_selection<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
        dir: CanvasPoint,
        fast: bool,
    ) -> bool {
        let mode = snapshot.interaction.nudge_step_mode;
        match mode {
            crate::io::NodeGraphNudgeStepMode::ScreenPx => {
                let step = if fast {
                    snapshot.interaction.nudge_fast_step_px
                } else {
                    snapshot.interaction.nudge_step_px
                };
                let delta = CanvasPoint {
                    x: dir.x * step,
                    y: dir.y * step,
                };
                self.nudge_selection_by_screen_delta(cx.app, cx.window, snapshot, delta);
            }
            crate::io::NodeGraphNudgeStepMode::Grid => {
                let grid = snapshot.interaction.snap_grid;
                let grid_w = grid.width.max(0.0);
                let grid_h = grid.height.max(0.0);
                if !(grid_w.is_finite() && grid_w > 0.0 && grid_h.is_finite() && grid_h > 0.0) {
                    // Fallback to screen-px mode for invalid/degenerate grids.
                    let step = if fast {
                        snapshot.interaction.nudge_fast_step_px
                    } else {
                        snapshot.interaction.nudge_step_px
                    };
                    let delta = CanvasPoint {
                        x: dir.x * step,
                        y: dir.y * step,
                    };
                    self.nudge_selection_by_screen_delta(cx.app, cx.window, snapshot, delta);
                } else {
                    let mul = if fast { 10.0 } else { 1.0 };
                    let steps = CanvasPoint {
                        x: dir.x * mul,
                        y: dir.y * mul,
                    };
                    self.nudge_selection_by_grid_step(cx.app, cx.window, snapshot, steps);
                }
            }
        }
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        true
    }

    pub(super) fn cmd_align_or_distribute_selection<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
        mode: AlignDistributeMode,
    ) -> bool {
        self.align_or_distribute_selection(cx.app, cx.window, snapshot, mode);
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        true
    }
}
