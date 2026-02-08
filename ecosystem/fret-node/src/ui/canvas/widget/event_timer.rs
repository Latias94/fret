use super::*;
use crate::ui::canvas::state::{ViewportAnimationEase, ViewportAnimationInterpolate};

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_timer<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        token: fret_core::TimerToken,
    ) {
        if self
            .interaction
            .toast
            .as_ref()
            .is_some_and(|t| t.timer == token)
        {
            self.interaction.toast = None;
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
            return;
        }

        if self
            .interaction
            .pan_inertia
            .as_ref()
            .is_some_and(|i| i.timer == token)
        {
            let tuning = snapshot.interaction.pan_inertia.clone();
            let zoom = snapshot.zoom;
            let before = snapshot.pan;

            let Some(mut inertia) = self.interaction.pan_inertia.take() else {
                return;
            };
            let timer = inertia.timer;
            let mut end_move = false;

            if !tuning.enabled
                || !self.pan_inertia_should_tick()
                || !zoom.is_finite()
                || zoom <= 0.0
                || !tuning.decay_per_s.is_finite()
                || tuning.decay_per_s <= 0.0
            {
                cx.app.push_effect(Effect::CancelTimer { token: timer });
                end_move = true;
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                if end_move {
                    let snap = self.sync_view_state(cx.app);
                    self.emit_move_end(
                        &snap,
                        ViewportMoveKind::PanInertia,
                        ViewportMoveEndOutcome::Ended,
                    );
                }
                return;
            }

            let now = Instant::now();
            let dt = (now - inertia.last_tick_at).as_secs_f32().clamp(0.0, 0.2);
            inertia.last_tick_at = now;

            if dt > 0.0 {
                let dx = inertia.velocity.x * dt;
                let dy = inertia.velocity.y * dt;
                self.update_view_state(cx.app, |s| {
                    s.pan.x += dx;
                    s.pan.y += dy;
                });
            }

            let after = self.sync_view_state(cx.app).pan;
            let moved_x = after.x - before.x;
            let moved_y = after.y - before.y;
            let moved = (moved_x * moved_x + moved_y * moved_y).sqrt();

            let decay = (-tuning.decay_per_s * dt).exp();
            inertia.velocity.x *= decay;
            inertia.velocity.y *= decay;

            let speed_screen = (inertia.velocity.x * inertia.velocity.x
                + inertia.velocity.y * inertia.velocity.y)
                .sqrt()
                * zoom;
            let min_speed = tuning.min_speed.max(0.0);

            if moved <= 1.0e-6
                || !speed_screen.is_finite()
                || speed_screen <= min_speed
                || !inertia.velocity.x.is_finite()
                || !inertia.velocity.y.is_finite()
            {
                cx.app.push_effect(Effect::CancelTimer { token: timer });
                end_move = true;
            } else {
                self.interaction.pan_inertia = Some(inertia);
            }

            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
            if end_move {
                let snap = self.sync_view_state(cx.app);
                self.emit_move_end(
                    &snap,
                    ViewportMoveKind::PanInertia,
                    ViewportMoveEndOutcome::Ended,
                );
            }
            return;
        }

        if self
            .interaction
            .viewport_animation
            .as_ref()
            .is_some_and(|a| a.timer == token)
        {
            let Some(mut anim) = self.interaction.viewport_animation.take() else {
                return;
            };

            if anim.duration.is_zero() {
                cx.app
                    .push_effect(Effect::CancelTimer { token: anim.timer });
                return;
            }

            let now = std::time::Instant::now();
            let mut dt = now
                .checked_duration_since(anim.last_tick_at)
                .unwrap_or_default();
            if dt < Self::PAN_INERTIA_TICK_INTERVAL {
                dt = Self::PAN_INERTIA_TICK_INTERVAL;
            }
            if dt > std::time::Duration::from_millis(200) {
                dt = std::time::Duration::from_millis(200);
            }
            anim.last_tick_at = now;
            anim.elapsed = (anim.elapsed + dt).min(anim.duration);

            let denom = anim.duration.as_secs_f32();
            let t = if denom > 0.0 {
                (anim.elapsed.as_secs_f32() / denom).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let u = match anim.ease {
                Some(ease) => ease.apply(t),
                None => match anim.interpolate {
                    ViewportAnimationInterpolate::Linear => t,
                    ViewportAnimationInterpolate::Smooth => {
                        ViewportAnimationEase::Smoothstep.apply(t)
                    }
                },
            };

            let pan = CanvasPoint {
                x: anim.from_pan.x + (anim.to_pan.x - anim.from_pan.x) * u,
                y: anim.from_pan.y + (anim.to_pan.y - anim.from_pan.y) * u,
            };
            let zoom = anim.from_zoom + (anim.to_zoom - anim.from_zoom) * u;

            self.update_view_state(cx.app, |s| {
                s.pan = pan;
                s.zoom = zoom;
            });

            let done = t >= 1.0 - 1.0e-6;
            if done {
                cx.app
                    .push_effect(Effect::CancelTimer { token: anim.timer });
            } else {
                self.interaction.viewport_animation = Some(anim);
            }

            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
            return;
        }

        if self.interaction.auto_pan_timer == Some(token) {
            if !self.auto_pan_should_tick(snapshot, cx.bounds) {
                self.stop_auto_pan_timer(cx.app);
                return;
            }

            let pos = self.interaction.last_pos.unwrap_or_default();
            let mods = self.interaction.last_modifiers;
            let zoom = snapshot.zoom;

            if self.interaction.wire_drag.is_some() {
                let _ = wire_drag::handle_wire_drag_move(self, cx, snapshot, pos, mods, zoom);
            } else if self.interaction.node_drag.is_some() {
                let _ = node_drag::handle_node_drag_move(self, cx, snapshot, pos, mods, zoom);
            } else if self.interaction.group_drag.is_some() {
                let _ = group_drag::handle_group_drag_move(self, cx, snapshot, pos, mods, zoom);
            } else if self.interaction.group_resize.is_some() {
                let _ = group_resize::handle_group_resize_move(self, cx, snapshot, pos, mods, zoom);
            }

            let snapshot = self.sync_view_state(cx.app);
            self.sync_auto_pan_timer(cx.app, cx.window, &snapshot, cx.bounds);
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }

        if self
            .interaction
            .viewport_move_debounce
            .as_ref()
            .is_some_and(|s| s.timer == token)
        {
            let Some(state) = self.interaction.viewport_move_debounce.take() else {
                return;
            };
            let snapshot = self.sync_view_state(cx.app);
            self.emit_move_end(&snapshot, state.kind, ViewportMoveEndOutcome::Ended);
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
    }
}
