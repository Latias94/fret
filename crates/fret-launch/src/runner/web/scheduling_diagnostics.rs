use crate::runner::common::scheduling;

use super::*;

pub(super) fn commit_presented_frame_for_window(
    app: &mut App,
    frame_id: &mut FrameId,
    window: AppWindowId,
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
        reason: fret_runtime::RunnerFrameDriveReason,
    ) {
        self.app.with_global_mut_untracked(
            fret_runtime::RunnerFrameDriveDiagnosticsStore::default,
            |store, _app| {
                store.record(self.app_window, self.frame_id, reason);
            },
        );
    }

    pub(super) fn request_redraw_with_reason(
        &mut self,
        window: &dyn Window,
        reason: fret_runtime::RunnerFrameDriveReason,
    ) {
        window.request_redraw();
        self.record_frame_drive_reason(reason);
    }

    pub(super) fn request_streaming_pending_redraw(&mut self, window: &dyn Window) {
        let Some(windows) = self.streaming_uploads.pending_redraw_hint() else {
            return;
        };
        let reason =
            fret_runtime::RunnerFrameDriveReason::for_streaming_pending_hint(windows.len());
        self.request_redraw_with_reason(window, reason);
    }

    pub(super) fn flush_raf_redraw_requests(&mut self, window: &dyn Window) {
        let raf_windows: Vec<AppWindowId> = self.raf_windows.drain().collect();
        for app_window in raf_windows {
            if app_window != self.app_window {
                continue;
            }
            self.request_redraw_with_reason(
                window,
                fret_runtime::RunnerFrameDriveReason::AboutToWaitRaf,
            );
        }
    }
}
