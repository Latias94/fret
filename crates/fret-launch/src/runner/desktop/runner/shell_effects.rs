use fret_core::{AppWindowId, Event, ShareItem, ShareSheetOutcome, ShareSheetToken};
use fret_platform::open_url::OpenUrl as _;
use fret_runtime::PlatformCapabilities;

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_show_about_panel(&self) {
        let _ = self;
        #[cfg(target_os = "macos")]
        {
            super::macos_menu::show_about_panel();
        }
    }

    pub(super) fn handle_hide_app(&self) {
        let _ = self;
        #[cfg(target_os = "macos")]
        {
            super::macos_menu::hide_app();
        }
    }

    pub(super) fn handle_hide_other_apps(&self) {
        let _ = self;
        #[cfg(target_os = "macos")]
        {
            super::macos_menu::hide_other_apps();
        }
    }

    pub(super) fn handle_unhide_all_apps(&self) {
        let _ = self;
        #[cfg(target_os = "macos")]
        {
            super::macos_menu::unhide_all_apps();
        }
    }

    pub(super) fn handle_open_url(&mut self, url: String) {
        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if !caps.shell.open_url {
            return;
        }
        if let Err(err) = self.open_url.open_url(&url) {
            tracing::debug!(?err, url = %url, "failed to open url");
        }
    }

    pub(super) fn handle_share_sheet_show(
        &mut self,
        window: AppWindowId,
        token: ShareSheetToken,
        items: Vec<ShareItem>,
    ) {
        let _ = items;
        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if !caps.shell.share_sheet {
            self.deliver_window_event_now(
                window,
                &Event::ShareSheetCompleted {
                    token,
                    outcome: ShareSheetOutcome::Unavailable,
                },
            );
            return;
        }

        self.deliver_window_event_now(
            window,
            &Event::ShareSheetCompleted {
                token,
                outcome: ShareSheetOutcome::Unavailable,
            },
        );
    }
}
