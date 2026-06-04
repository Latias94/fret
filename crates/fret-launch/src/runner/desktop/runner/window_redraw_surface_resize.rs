use fret_core::{AppWindowId, Event, Px};

use super::redraw_hitch::quantize_logical_px;
use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_redraw_pending_surface_resize(&mut self, app_window: AppWindowId) {
        let Some(size) = self
            .windows
            .get_mut(app_window)
            .and_then(|state| state.pending_surface_resize.take())
        else {
            return;
        };

        // The platform event path now reconfigures the surface immediately. Keep the redraw-time
        // resize as an eventual-consistency fallback for windows that queued a size before their
        // surface/context existed.
        self.resize_surface(app_window, size.width, size.height);

        // Keep delivering size/scale events once per frame so interactive resizes do not spam
        // high-level relayout work even though the GPU surface has already been synchronized to
        // the latest physical size.
        let (logical_width, logical_height, scale_factor, should_deliver_resized) = {
            let Some(state) = self.windows.get_mut(app_window) else {
                return;
            };
            let scale_factor = state.window.scale_factor() as f32;
            let logical: winit::dpi::LogicalSize<f32> =
                size.to_logical(state.window.scale_factor());
            let logical_width = quantize_logical_px(logical.width);
            let logical_height = quantize_logical_px(logical.height);
            let bits = (logical_width.to_bits(), logical_height.to_bits());
            let should_deliver_resized = state
                .last_delivered_window_resized
                .is_none_or(|prev| prev != bits);
            if should_deliver_resized {
                state.last_delivered_window_resized = Some(bits);
            }
            (
                logical_width,
                logical_height,
                scale_factor,
                should_deliver_resized,
            )
        };

        if should_deliver_resized {
            self.deliver_window_event_now(
                app_window,
                &Event::WindowResized {
                    width: Px(logical_width),
                    height: Px(logical_height),
                },
            );
        }
        let should_deliver_scale_factor = self
            .app
            .global::<fret_core::WindowMetricsService>()
            .and_then(|svc| svc.scale_factor(app_window))
            .is_none_or(|prev| prev.to_bits() != scale_factor.to_bits());
        if should_deliver_scale_factor {
            self.deliver_window_event_now(
                app_window,
                &Event::WindowScaleFactorChanged(scale_factor),
            );
        }
    }
}
