use fret_core::{AppWindowId, Event};
use fret_platform::clipboard::Clipboard as _;
use fret_runtime::{ClipboardToken, PlatformCapabilities};

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn apply_diag_clipboard_force_unavailable(
        &mut self,
        window: AppWindowId,
        enabled: bool,
    ) {
        if enabled {
            self.diag_clipboard_force_unavailable_windows.insert(window);
        } else {
            self.diag_clipboard_force_unavailable_windows
                .remove(&window);
        }
    }

    pub(super) fn handle_clipboard_write_text(
        &mut self,
        window: AppWindowId,
        token: ClipboardToken,
        text: String,
    ) {
        let outcome = if self
            .diag_clipboard_force_unavailable_windows
            .contains(&window)
        {
            fret_core::ClipboardWriteOutcome::Failed {
                error: fret_core::ClipboardAccessError {
                    kind: fret_core::ClipboardAccessErrorKind::Unavailable,
                    message: Some("diagnostics forced clipboard unavailable".to_string()),
                },
            }
        } else {
            match self.clipboard.set_text(&text) {
                Ok(()) => fret_core::ClipboardWriteOutcome::Succeeded,
                Err(error) => {
                    tracing::debug!(?error, "failed to set clipboard text");
                    fret_core::ClipboardWriteOutcome::Failed { error }
                }
            }
        };

        self.deliver_window_event_now(window, &Event::ClipboardWriteCompleted { token, outcome });
    }

    pub(super) fn handle_clipboard_read_text(
        &mut self,
        window: AppWindowId,
        token: ClipboardToken,
    ) {
        if self
            .diag_clipboard_force_unavailable_windows
            .contains(&window)
        {
            self.deliver_window_event_now(
                window,
                &Event::ClipboardReadFailed {
                    token,
                    error: fret_core::ClipboardAccessError {
                        kind: fret_core::ClipboardAccessErrorKind::Unavailable,
                        message: Some("diagnostics forced clipboard unavailable".to_string()),
                    },
                },
            );
            return;
        }

        match self.clipboard.get_text() {
            Ok(Some(text)) => {
                self.deliver_window_event_now(window, &Event::ClipboardReadText { token, text })
            }
            Ok(None) => self.deliver_window_event_now(
                window,
                &Event::ClipboardReadFailed {
                    token,
                    error: fret_core::ClipboardAccessError {
                        kind: fret_core::ClipboardAccessErrorKind::Unavailable,
                        message: None,
                    },
                },
            ),
            Err(error) => {
                tracing::debug!(?error, "failed to read clipboard text");
                self.deliver_window_event_now(window, &Event::ClipboardReadFailed { token, error });
            }
        }
    }

    pub(super) fn handle_primary_selection_set_text(&mut self, text: String) {
        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if !caps.clipboard.primary_text {
            return;
        }
        if let Err(err) = self.clipboard.set_primary_text(&text) {
            tracing::debug!(?err, "failed to set primary selection text");
        }
    }

    pub(super) fn handle_primary_selection_get_text(
        &mut self,
        window: AppWindowId,
        token: ClipboardToken,
    ) {
        if self
            .diag_clipboard_force_unavailable_windows
            .contains(&window)
        {
            self.deliver_window_event_now(
                window,
                &Event::PrimarySelectionTextUnavailable { token },
            );
            return;
        }

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if !caps.clipboard.primary_text {
            self.deliver_window_event_now(
                window,
                &Event::PrimarySelectionTextUnavailable { token },
            );
            return;
        }

        match self.clipboard.get_primary_text() {
            Ok(Some(text)) => {
                self.deliver_window_event_now(window, &Event::PrimarySelectionText { token, text })
            }
            Ok(None) | Err(_) => self.deliver_window_event_now(
                window,
                &Event::PrimarySelectionTextUnavailable { token },
            ),
        }
    }
}
