use fret_core::AppWindowId;
use winit::event::WindowEvent;

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_pre_dispatch_event(
        &mut self,
        app_window: AppWindowId,
        event: &WindowEvent,
    ) {
        if let Some(state) = self.windows.get_mut(app_window)
            && let Some(a11y) = state.accessibility.as_mut()
        {
            a11y.process_event(state.window.as_ref(), event);
        }

        if let WindowEvent::Ime(ime) = event
            && std::env::var_os("FRET_IME_DEBUG").is_some_and(|v| !v.is_empty())
            && let Some(state) = self.windows.get(app_window)
        {
            tracing::info!(
                "IME_DEBUG winit: WindowEvent::Ime({:?}) cached_rect={}",
                ime,
                state.platform.ime_cursor_area().is_some()
            );
        }
    }
}
