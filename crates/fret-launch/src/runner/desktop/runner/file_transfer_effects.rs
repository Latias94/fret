use fret_core::{AppWindowId, Event};
use fret_platform::external_drop::ExternalDropProvider as _;
use fret_platform::file_dialog::FileDialogProvider as _;
use fret_platform_native::external_drop::NativeExternalDrop;
use fret_platform_native::file_dialog::NativeFileDialog;
use fret_runtime::{PlatformCapabilities, PlatformCompletion};

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_external_drop_read_all(
        &mut self,
        window: AppWindowId,
        token: fret_core::ExternalDropToken,
    ) {
        let limits = fret_platform::external_drop::ExternalDropReadLimits {
            max_total_bytes: self.config.external_drop_max_total_bytes,
            max_file_bytes: self.config.external_drop_max_file_bytes,
            max_files: self.config.external_drop_max_files,
        };

        if let Some(paths) = self.external_drop.paths(token).map(|p| p.to_vec())
            && self.spawn_platform_completion_task(window, move || {
                let event = NativeExternalDrop::read_paths(token, paths, limits);
                PlatformCompletion::ExternalDropData(event)
            })
        {
            return;
        }

        let Some(event) = self.external_drop.read_all(token, limits) else {
            return;
        };
        self.deliver_window_event_now(window, &Event::ExternalDropData(event));
    }

    pub(super) fn handle_external_drop_read_all_with_limits(
        &mut self,
        window: AppWindowId,
        token: fret_core::ExternalDropToken,
        limits: fret_core::ExternalDropReadLimits,
    ) {
        let cap = fret_platform::external_drop::ExternalDropReadLimits {
            max_total_bytes: self.config.external_drop_max_total_bytes,
            max_file_bytes: self.config.external_drop_max_file_bytes,
            max_files: self.config.external_drop_max_files,
        };
        let limits = limits.capped_by(cap);

        if let Some(paths) = self.external_drop.paths(token).map(|p| p.to_vec())
            && self.spawn_platform_completion_task(window, move || {
                let event = NativeExternalDrop::read_paths(token, paths, limits);
                PlatformCompletion::ExternalDropData(event)
            })
        {
            return;
        }

        let Some(event) = self.external_drop.read_all(token, limits) else {
            return;
        };
        self.deliver_window_event_now(window, &Event::ExternalDropData(event));
    }

    pub(super) fn handle_external_drop_release(&mut self, token: fret_core::ExternalDropToken) {
        self.external_drop.release(token);
    }

    pub(super) fn handle_file_dialog_open(
        &mut self,
        window: AppWindowId,
        options: fret_core::FileDialogOptions,
    ) {
        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if !caps.fs.file_dialogs {
            return;
        }
        match self.file_dialog.open_files(&options) {
            Ok(Some(selection)) => {
                self.deliver_platform_completion_now(
                    window,
                    PlatformCompletion::FileDialogSelection(selection),
                );
            }
            Ok(None) => {
                self.deliver_platform_completion_now(
                    window,
                    PlatformCompletion::FileDialogCanceled,
                );
            }
            Err(err) => {
                tracing::debug!(?err, "file dialog open failed");
            }
        }
    }

    pub(super) fn handle_file_dialog_read_all(
        &mut self,
        window: AppWindowId,
        token: fret_core::FileDialogToken,
    ) {
        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if !caps.fs.file_dialogs {
            return;
        }
        let limits = fret_platform::external_drop::ExternalDropReadLimits {
            max_total_bytes: self.config.file_dialog_max_total_bytes,
            max_file_bytes: self.config.file_dialog_max_file_bytes,
            max_files: self.config.file_dialog_max_files,
        };

        if let Some(paths) = self.file_dialog.paths(token).map(|p| p.to_vec())
            && self.spawn_platform_completion_task(window, move || {
                let data = NativeFileDialog::read_paths(token, paths, limits);
                PlatformCompletion::FileDialogData(data)
            })
        {
            return;
        }

        let Some(data) = self.file_dialog.read_all(token, limits) else {
            return;
        };
        self.deliver_platform_completion_now(window, PlatformCompletion::FileDialogData(data));
    }

    pub(super) fn handle_file_dialog_read_all_with_limits(
        &mut self,
        window: AppWindowId,
        token: fret_core::FileDialogToken,
        limits: fret_core::ExternalDropReadLimits,
    ) {
        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if !caps.fs.file_dialogs {
            return;
        }
        let cap = fret_platform::external_drop::ExternalDropReadLimits {
            max_total_bytes: self.config.file_dialog_max_total_bytes,
            max_file_bytes: self.config.file_dialog_max_file_bytes,
            max_files: self.config.file_dialog_max_files,
        };
        let limits = limits.capped_by(cap);

        if let Some(paths) = self.file_dialog.paths(token).map(|p| p.to_vec())
            && self.spawn_platform_completion_task(window, move || {
                let data = NativeFileDialog::read_paths(token, paths, limits);
                PlatformCompletion::FileDialogData(data)
            })
        {
            return;
        }

        let Some(data) = self.file_dialog.read_all(token, limits) else {
            return;
        };
        self.deliver_platform_completion_now(window, PlatformCompletion::FileDialogData(data));
    }

    pub(super) fn handle_file_dialog_release(&mut self, token: fret_core::FileDialogToken) {
        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if !caps.fs.file_dialogs {
            return;
        }
        self.file_dialog.release(token);
    }
}
