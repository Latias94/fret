use crate::RenderPerfSnapshot;
use fret_core::AppWindowId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone, Copy)]
pub struct RendererPerfFrameSample {
    pub tick_id: u64,
    pub frame_id: u64,
    pub perf: RenderPerfSnapshot,
}

/// Best-effort store for the most recent per-frame renderer perf sample per window.
///
/// This is intended for diagnostics bundles (`fretboard-dev diag *`) and should not be used as a
/// correctness signal.
#[derive(Debug, Default, Clone)]
pub struct RendererPerfFrameStore {
    by_window: Arc<Mutex<HashMap<AppWindowId, RendererPerfFrameSample>>>,
}

impl RendererPerfFrameStore {
    pub fn record(
        &self,
        window: AppWindowId,
        tick_id: u64,
        frame_id: u64,
        perf: RenderPerfSnapshot,
    ) {
        self.by_window
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .insert(
                window,
                RendererPerfFrameSample {
                    tick_id,
                    frame_id,
                    perf,
                },
            );
    }

    pub fn latest_for_window(&self, window: AppWindowId) -> Option<RendererPerfFrameSample> {
        self.by_window
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .get(&window)
            .copied()
    }

    pub fn clear_window(&self, window: AppWindowId) -> Option<RendererPerfFrameSample> {
        self.by_window
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .remove(&window)
    }
}
