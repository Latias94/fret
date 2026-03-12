use super::super::*;

const ZOOM_STEP_FACTOR: f32 = 1.2;

fn reset_view_state(view_state: &mut NodeGraphViewState) {
    view_state.pan = CanvasPoint::default();
    view_state.zoom = 1.0;
}

fn zoom_command_factor(zoom_in: bool) -> f32 {
    if zoom_in {
        ZOOM_STEP_FACTOR
    } else {
        1.0 / ZOOM_STEP_FACTOR
    }
}

fn apply_cached_viewport(view_state: &mut NodeGraphViewState, pan: CanvasPoint, zoom: f32) {
    view_state.pan = pan;
    view_state.zoom = zoom;
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn cmd_reset_view<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
    ) -> bool {
        self.update_view_state(cx.app, reset_view_state);
        super::super::command_ui::finish_command_paint(cx)
    }

    pub(in super::super) fn cmd_zoom_in<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        self.zoom_about_center_factor(bounds, zoom_command_factor(true));
        let pan = self.cached_pan;
        let zoom = self.cached_zoom;
        self.update_view_state(cx.app, |view_state| {
            apply_cached_viewport(view_state, pan, zoom);
        });
        super::super::command_ui::finish_command_paint(cx)
    }

    pub(in super::super) fn cmd_zoom_out<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        self.zoom_about_center_factor(bounds, zoom_command_factor(false));
        let pan = self.cached_pan;
        let zoom = self.cached_zoom;
        self.update_view_state(cx.app, |view_state| {
            apply_cached_viewport(view_state, pan, zoom);
        });
        super::super::command_ui::finish_command_paint(cx)
    }
}

#[cfg(test)]
mod tests;
