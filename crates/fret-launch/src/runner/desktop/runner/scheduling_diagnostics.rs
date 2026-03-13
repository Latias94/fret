use crate::runner::common::scheduling;

use super::*;

pub(super) fn commit_presented_frame_for_window(
    app: &mut App,
    frame_id: &mut FrameId,
    window: fret_core::AppWindowId,
) {
    scheduling::commit_presented_frame_and_then(frame_id, |committed_frame_id| {
        app.set_frame_id(committed_frame_id);
        app.with_global_mut_untracked(
            fret_runtime::RunnerPresentDiagnosticsStore::default,
            |store, _app| {
                store.record_present(window, committed_frame_id);
            },
        );
    });
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn record_frame_drive_reason(
        &mut self,
        window: fret_core::AppWindowId,
        reason: fret_runtime::RunnerFrameDriveReason,
    ) {
        self.app.with_global_mut_untracked(
            fret_runtime::RunnerFrameDriveDiagnosticsStore::default,
            |store, _app| {
                store.record(window, self.frame_id, reason);
            },
        );
    }

    pub(super) fn request_window_redraw_with_reason(
        &mut self,
        window: fret_core::AppWindowId,
        reason: fret_runtime::RunnerFrameDriveReason,
    ) -> bool {
        let Some(window_handle) = self.windows.get(window).map(|state| state.window.clone()) else {
            return false;
        };
        window_handle.request_redraw();
        self.record_frame_drive_reason(window, reason);
        true
    }

    pub(super) fn request_all_windows_redraw_with_reason(
        &mut self,
        reason: fret_runtime::RunnerFrameDriveReason,
    ) {
        let windows: Vec<fret_core::AppWindowId> = self.windows.keys().collect();
        for window in windows {
            let _ = self.request_window_redraw_with_reason(window, reason);
        }
    }

    pub(super) fn request_streaming_pending_redraws(&mut self) {
        let Some(windows) = self.streaming_uploads.pending_redraw_hint() else {
            return;
        };
        let reason =
            fret_runtime::RunnerFrameDriveReason::for_streaming_pending_hint(windows.len());
        if windows.is_empty() {
            self.request_all_windows_redraw_with_reason(reason);
            return;
        }

        for window in windows {
            let _ = self.request_window_redraw_with_reason(window, reason);
        }
    }

    pub(super) fn flush_raf_redraw_requests(&mut self) {
        let windows: Vec<fret_core::AppWindowId> = self.raf_windows.drain().collect();
        for window in windows {
            let _ = self.request_window_redraw_with_reason(
                window,
                fret_runtime::RunnerFrameDriveReason::AboutToWaitRaf,
            );
        }
    }
}
