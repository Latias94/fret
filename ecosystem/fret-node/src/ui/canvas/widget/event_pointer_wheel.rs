use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_pointer_wheel<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        delta: Point,
        modifiers: fret_core::Modifiers,
        zoom: f32,
    ) {
        if self.interaction.pan_inertia.is_some() {
            self.stop_pan_inertia_timer(cx.app);
            self.emit_move_end(
                snapshot,
                ViewportMoveKind::PanInertia,
                ViewportMoveEndOutcome::Ended,
            );
        }
        self.interaction.last_modifiers = modifiers;
        self.interaction.multi_selection_active = snapshot
            .interaction
            .multi_selection_key
            .is_pressed(modifiers);
        if searcher::handle_searcher_wheel(self, cx, delta, modifiers, zoom) {
            return;
        }

        let zoom_active = snapshot
            .interaction
            .zoom_activation_key
            .is_pressed(modifiers);
        if snapshot.interaction.zoom_on_scroll && zoom_active {
            self.bump_viewport_move_debounce(
                cx.app,
                cx.window,
                snapshot,
                ViewportMoveKind::ZoomWheel,
            );
            let speed = snapshot.interaction.zoom_on_scroll_speed.max(0.0);
            let delta_screen_y = delta.y.0 * zoom;
            let factor = fret_canvas::view::wheel_zoom_factor(
                delta_screen_y,
                fret_canvas::view::DEFAULT_WHEEL_ZOOM_BASE,
                fret_canvas::view::DEFAULT_WHEEL_ZOOM_STEP,
                speed,
            )
            .unwrap_or(1.0);
            self.zoom_about_pointer_factor(position, factor);
            let pan = self.cached_pan;
            let zoom = self.cached_zoom;
            self.update_view_state(cx.app, |s| {
                s.pan = pan;
                s.zoom = zoom;
            });
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        } else if snapshot.interaction.pan_on_scroll
            || (snapshot.interaction.space_to_pan && self.interaction.pan_activation_key_held)
        {
            self.bump_viewport_move_debounce(
                cx.app,
                cx.window,
                snapshot,
                ViewportMoveKind::PanScroll,
            );
            let mode = snapshot.interaction.pan_on_scroll_mode;
            let speed = snapshot.interaction.pan_on_scroll_speed.max(0.0);
            let dy_for_shift = delta.y.0;

            let mut dx = delta.x.0;
            let mut dy = delta.y.0;
            match mode {
                crate::io::NodeGraphPanOnScrollMode::Free => {}
                crate::io::NodeGraphPanOnScrollMode::Horizontal => {
                    dy = 0.0;
                }
                crate::io::NodeGraphPanOnScrollMode::Vertical => {
                    dx = 0.0;
                }
            }

            if cx.input_ctx.platform != fret_runtime::Platform::Macos
                && modifiers.shift
                && !matches!(mode, crate::io::NodeGraphPanOnScrollMode::Vertical)
            {
                dx = dy_for_shift;
                dy = 0.0;
            }
            self.update_view_state(cx.app, |s| {
                s.pan.x += dx * speed;
                s.pan.y += dy * speed;
            });
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
    }

    pub(super) fn handle_pinch_gesture<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        delta: f32,
    ) {
        if self.interaction.pan_inertia.is_some() {
            self.stop_pan_inertia_timer(cx.app);
            self.emit_move_end(
                snapshot,
                ViewportMoveKind::PanInertia,
                ViewportMoveEndOutcome::Ended,
            );
        }
        if !snapshot.interaction.zoom_on_pinch {
            return;
        }
        if !delta.is_finite() {
            return;
        }

        self.bump_viewport_move_debounce(cx.app, cx.window, snapshot, ViewportMoveKind::ZoomPinch);

        let speed = snapshot.interaction.zoom_on_pinch_speed.max(0.0);
        let delta = delta.clamp(-0.95, 10.0);
        let factor = (1.0 + delta * speed).max(0.01);
        self.zoom_about_pointer_factor(position, factor);
        let pan = self.cached_pan;
        let zoom = self.cached_zoom;
        self.update_view_state(cx.app, |s| {
            s.pan = pan;
            s.zoom = zoom;
        });
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
    }
}
