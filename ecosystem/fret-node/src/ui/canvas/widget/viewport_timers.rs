use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn auto_pan_delta(snapshot: &ViewSnapshot, pos: Point, bounds: Rect) -> CanvasPoint {
        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return CanvasPoint::default();
        }

        let margin_screen = snapshot.interaction.auto_pan.margin;
        let speed_screen_per_s = snapshot.interaction.auto_pan.speed;
        if !margin_screen.is_finite() || margin_screen <= 0.0 {
            return CanvasPoint::default();
        }
        if !speed_screen_per_s.is_finite() || speed_screen_per_s <= 0.0 {
            return CanvasPoint::default();
        }

        let viewport_w = bounds.size.width.0;
        let viewport_h = bounds.size.height.0;
        if !viewport_w.is_finite()
            || viewport_w <= 0.0
            || !viewport_h.is_finite()
            || viewport_h <= 0.0
        {
            return CanvasPoint::default();
        }

        let pan = snapshot.pan;
        let pos_screen_x = (pos.x.0 + pan.x) * zoom;
        let pos_screen_y = (pos.y.0 + pan.y) * zoom;

        let dist_left = pos_screen_x;
        let dist_right = viewport_w - pos_screen_x;
        let dist_top = pos_screen_y;
        let dist_bottom = viewport_h - pos_screen_y;

        let step_screen = speed_screen_per_s / Self::AUTO_PAN_TICK_HZ;
        let step_graph = step_screen / zoom;

        let mut delta_x = 0.0;
        let mut delta_y = 0.0;

        if dist_left.is_finite() && dist_left < margin_screen {
            let factor = ((margin_screen - dist_left) / margin_screen).clamp(0.0, 1.0);
            delta_x += step_graph * factor;
        }
        if dist_right.is_finite() && dist_right < margin_screen {
            let factor = ((margin_screen - dist_right) / margin_screen).clamp(0.0, 1.0);
            delta_x -= step_graph * factor;
        }
        if dist_top.is_finite() && dist_top < margin_screen {
            let factor = ((margin_screen - dist_top) / margin_screen).clamp(0.0, 1.0);
            delta_y += step_graph * factor;
        }
        if dist_bottom.is_finite() && dist_bottom < margin_screen {
            let factor = ((margin_screen - dist_bottom) / margin_screen).clamp(0.0, 1.0);
            delta_y -= step_graph * factor;
        }

        if !delta_x.is_finite() || !delta_y.is_finite() {
            return CanvasPoint::default();
        }

        CanvasPoint {
            x: delta_x,
            y: delta_y,
        }
    }

    pub(super) fn stop_auto_pan_timer<H: UiHost>(&mut self, host: &mut H) {
        let Some(timer) = self.interaction.auto_pan_timer.take() else {
            return;
        };
        host.push_effect(Effect::CancelTimer { token: timer });
    }

    pub(super) fn stop_pan_inertia_timer<H: UiHost>(&mut self, host: &mut H) {
        let Some(inertia) = self.interaction.pan_inertia.take() else {
            return;
        };
        host.push_effect(Effect::CancelTimer {
            token: inertia.timer,
        });
    }

    pub(super) fn bump_viewport_move_debounce<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        kind: ViewportMoveKind,
    ) {
        if let Some(prev) = self.interaction.viewport_move_debounce.take() {
            host.push_effect(Effect::CancelTimer { token: prev.timer });
            if prev.kind != kind {
                self.emit_move_end(snapshot, prev.kind, ViewportMoveEndOutcome::Ended);
                self.emit_move_start(snapshot, kind);
            }
        } else {
            self.emit_move_start(snapshot, kind);
        }

        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Self::VIEWPORT_MOVE_END_DEBOUNCE,
            repeat: None,
        });
        self.interaction.viewport_move_debounce = Some(ViewportMoveDebounceState { kind, timer });
    }

    pub(super) fn pan_inertia_should_tick(&self) -> bool {
        if self.interaction.searcher.is_some() || self.interaction.context_menu.is_some() {
            return false;
        }
        if self.interaction.panning {
            return false;
        }
        self.interaction.pending_marquee.is_none()
            && self.interaction.marquee.is_none()
            && self.interaction.pending_node_drag.is_none()
            && self.interaction.node_drag.is_none()
            && self.interaction.pending_group_drag.is_none()
            && self.interaction.group_drag.is_none()
            && self.interaction.pending_group_resize.is_none()
            && self.interaction.group_resize.is_none()
            && self.interaction.pending_node_resize.is_none()
            && self.interaction.node_resize.is_none()
            && self.interaction.pending_wire_drag.is_none()
            && self.interaction.wire_drag.is_none()
            && self.interaction.edge_drag.is_none()
    }

    pub(super) fn maybe_start_pan_inertia_timer<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        self.stop_pan_inertia_timer(host);

        let tuning = &snapshot.interaction.pan_inertia;
        if !tuning.enabled {
            return false;
        }

        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return false;
        }

        let mut velocity = self.interaction.pan_velocity;
        if !velocity.x.is_finite() || !velocity.y.is_finite() {
            return false;
        }

        let speed_screen = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt() * zoom;
        let min_speed = tuning.min_speed.max(0.0);
        if !speed_screen.is_finite() || speed_screen < min_speed {
            return false;
        }

        let max_speed = tuning.max_speed.max(min_speed);
        if max_speed.is_finite() && max_speed > 0.0 {
            let max_speed_canvas = max_speed / zoom;
            let speed_canvas = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
            if speed_canvas.is_finite() && speed_canvas > max_speed_canvas && speed_canvas > 0.0 {
                let scale = max_speed_canvas / speed_canvas;
                velocity.x *= scale;
                velocity.y *= scale;
            }
        }

        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Self::PAN_INERTIA_TICK_INTERVAL,
            repeat: Some(Self::PAN_INERTIA_TICK_INTERVAL),
        });
        self.interaction.pan_inertia = Some(PanInertiaState {
            timer,
            velocity,
            last_tick_at: Instant::now(),
        });
        true
    }

    pub(super) fn ensure_auto_pan_timer_running<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
    ) {
        if self.interaction.auto_pan_timer.is_some() {
            return;
        }
        let timer = host.next_timer_token();
        host.push_effect(Effect::SetTimer {
            window,
            token: timer,
            after: Self::AUTO_PAN_TICK_INTERVAL,
            repeat: Some(Self::AUTO_PAN_TICK_INTERVAL),
        });
        self.interaction.auto_pan_timer = Some(timer);
    }

    pub(super) fn auto_pan_should_tick(&self, snapshot: &ViewSnapshot, bounds: Rect) -> bool {
        if self.interaction.searcher.is_some() || self.interaction.context_menu.is_some() {
            return false;
        }
        let Some(pos) = self.interaction.last_pos else {
            return false;
        };

        let wants_node_drag = snapshot.interaction.auto_pan.on_node_drag
            && (self.interaction.node_drag.is_some()
                || self.interaction.group_drag.is_some()
                || self.interaction.group_resize.is_some());
        let wants_connect =
            snapshot.interaction.auto_pan.on_connect && self.interaction.wire_drag.is_some();

        if !wants_node_drag && !wants_connect {
            return false;
        }

        let delta = Self::auto_pan_delta(snapshot, pos, bounds);
        delta.x != 0.0 || delta.y != 0.0
    }

    pub(super) fn sync_auto_pan_timer<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        bounds: Rect,
    ) {
        if self.auto_pan_should_tick(snapshot, bounds) {
            self.ensure_auto_pan_timer_running(host, window);
        } else {
            self.stop_auto_pan_timer(host);
        }
    }
}
