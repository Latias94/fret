use super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn deliver_window_event_now(
        &mut self,
        window: fret_core::AppWindowId,
        event: &Event,
    ) {
        if self.maybe_handle_hotpatch_event(window, event) {
            return;
        }
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        fret_runtime::apply_window_metrics_event(&mut self.app, window, event);
        let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
        self.driver.handle_event(
            WinitEventContext {
                app: &mut self.app,
                services,
                window,
                state: &mut state.user,
            },
            event,
        );
    }

    pub(super) fn deliver_platform_completion_now(
        &mut self,
        window: fret_core::AppWindowId,
        completion: PlatformCompletion,
    ) {
        match completion {
            PlatformCompletion::ClipboardText { token, text } => {
                self.deliver_window_event_now(window, &Event::ClipboardText { token, text });
            }
            PlatformCompletion::ClipboardTextUnavailable { token } => {
                self.deliver_window_event_now(window, &Event::ClipboardTextUnavailable { token });
            }
            PlatformCompletion::ExternalDropData(data) => {
                self.deliver_window_event_now(window, &Event::ExternalDropData(data));
            }
            PlatformCompletion::FileDialogSelection(selection) => {
                self.deliver_window_event_now(window, &Event::FileDialogSelection(selection));
            }
            PlatformCompletion::FileDialogData(data) => {
                self.deliver_window_event_now(window, &Event::FileDialogData(data));
            }
            PlatformCompletion::FileDialogCanceled => {
                self.deliver_window_event_now(window, &Event::FileDialogCanceled);
            }
        }
    }
}
