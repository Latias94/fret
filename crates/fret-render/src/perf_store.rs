use crate::RenderPerfSnapshot;
use fret_core::AppWindowId;
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Copy)]
pub struct RendererPerfFrameSample {
    pub tick_id: u64,
    pub frame_id: u64,
    pub perf: RenderPerfSnapshot,
}

/// Best-effort store for the most recent per-frame renderer perf sample per window.
///
/// This is intended for diagnostics bundles (`fretboard diag *`) and should not be used as a
/// correctness signal.
#[derive(Debug, Default)]
pub struct RendererPerfFrameStore {
    by_window: HashMap<AppWindowId, RendererPerfFrameSample>,
}

impl RendererPerfFrameStore {
    pub fn record(
        &mut self,
        window: AppWindowId,
        tick_id: u64,
        frame_id: u64,
        perf: RenderPerfSnapshot,
    ) {
        self.by_window.insert(
            window,
            RendererPerfFrameSample {
                tick_id,
                frame_id,
                perf,
            },
        );
    }

    pub fn latest_for_window(&self, window: AppWindowId) -> Option<RendererPerfFrameSample> {
        self.by_window.get(&window).copied()
    }
}
