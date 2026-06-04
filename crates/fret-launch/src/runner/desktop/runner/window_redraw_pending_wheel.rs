use fret_core::{AppWindowId, Event};

use super::wheel_coalescing::{wheel_coalesce_delta, wheel_split_delta_by_max_abs_px};
use super::window::PendingWheelEvent;
use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_redraw_pending_wheel(&mut self, app_window: AppWindowId) {
        if let Some(req) = self.poll_diag_wheel_burst_inject(app_window) {
            let total_dx = req.delta_x * (req.count.max(1) as f32);
            let total_dy = req.delta_y * (req.count.max(1) as f32);
            let injected = PendingWheelEvent {
                pointer_id: fret_core::PointerId(0),
                position: req.position,
                delta: fret_core::Point::new(fret_core::Px(total_dx), fret_core::Px(total_dy)),
                modifiers: req.modifiers,
                pointer_type: req.pointer_type,
            };

            if Self::wheel_coalescing_enabled() {
                if let Some(state) = self.windows.get_mut(app_window) {
                    state.pending_wheel = Some(match state.pending_wheel.take() {
                        Some(mut prev) => {
                            prev.delta = wheel_coalesce_delta(prev.delta, injected.delta);
                            prev.position = injected.position;
                            prev.modifiers = injected.modifiers;
                            prev.pointer_type = injected.pointer_type;
                            prev.pointer_id = injected.pointer_id;
                            prev
                        }
                        None => injected,
                    });
                }
                self.app.request_redraw(app_window);
            } else {
                self.deliver_pending_wheel_now(app_window, injected);
            }
        }

        if Self::wheel_coalescing_enabled() {
            let max_abs = Self::wheel_coalescing_max_abs_px();
            let mut to_deliver: Option<PendingWheelEvent> = None;
            let mut remainder: Option<PendingWheelEvent> = None;

            if let Some(state) = self.windows.get_mut(app_window)
                && let Some(pending) = state.pending_wheel.take()
            {
                let (delivered_delta, remainder_delta) =
                    wheel_split_delta_by_max_abs_px(pending.delta, max_abs);
                if delivered_delta.x.0.abs() > 0.0001 || delivered_delta.y.0.abs() > 0.0001 {
                    to_deliver = Some(PendingWheelEvent {
                        delta: delivered_delta,
                        ..pending
                    });
                }
                if remainder_delta.x.0.abs() > 0.0001 || remainder_delta.y.0.abs() > 0.0001 {
                    remainder = Some(PendingWheelEvent {
                        delta: remainder_delta,
                        ..pending
                    });
                }
            }

            if let Some(remainder) = remainder {
                if let Some(state) = self.windows.get_mut(app_window) {
                    state.pending_wheel = Some(remainder);
                }
                self.app.request_redraw(app_window);
            }

            if let Some(wheel) = to_deliver {
                self.deliver_pending_wheel_now(app_window, wheel);
            }
        }
    }

    fn deliver_pending_wheel_now(&mut self, app_window: AppWindowId, wheel: PendingWheelEvent) {
        self.deliver_window_event_now(
            app_window,
            &Event::Pointer(fret_core::PointerEvent::Wheel {
                pointer_id: wheel.pointer_id,
                position: wheel.position,
                delta: wheel.delta,
                modifiers: wheel.modifiers,
                pointer_type: wheel.pointer_type,
            }),
        );
    }
}
