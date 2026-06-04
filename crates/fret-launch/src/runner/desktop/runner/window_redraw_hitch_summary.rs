use fret_core::time::Instant;
use fret_core::{AppWindowId, Rect};
use fret_runtime::{FrameId, TickId};

use super::redraw_hitch::{RedrawHitchConfig, write_redraw_hitch_log};

pub(super) struct WindowRedrawHitchSummaryInput {
    pub(super) config: Option<RedrawHitchConfig>,
    pub(super) started: Option<Instant>,
    pub(super) app_window: AppWindowId,
    pub(super) tick_id: TickId,
    pub(super) frame_id: FrameId,
    pub(super) prepare_ms: Option<u64>,
    pub(super) render_ms: Option<u64>,
    pub(super) record_ms: Option<u64>,
    pub(super) present_ms: Option<u64>,
    pub(super) scene_ops: usize,
    pub(super) bounds: Rect,
    pub(super) scale_factor: f32,
}

pub(super) fn maybe_write_window_redraw_hitch_summary(input: WindowRedrawHitchSummaryInput) {
    let (Some(config), Some(started)) = (input.config, input.started) else {
        return;
    };

    let total_ms = started.elapsed().as_millis() as u64;
    if total_ms < config.hitch_ms {
        return;
    }

    write_redraw_hitch_log(&format!(
        "redraw hitch window={app_window:?} tick_id={tick_id} frame_id={frame_id} total_ms={total_ms} prepare_ms={prepare_ms:?} render_ms={render_ms:?} record_ms={record_ms:?} present_ms={present_ms:?} scene_ops={scene_ops} bounds={bounds:?} scale_factor={scale_factor}",
        app_window = input.app_window,
        tick_id = input.tick_id.0,
        frame_id = input.frame_id.0,
        prepare_ms = input.prepare_ms,
        render_ms = input.render_ms,
        record_ms = input.record_ms,
        present_ms = input.present_ms,
        scene_ops = input.scene_ops,
        bounds = input.bounds,
        scale_factor = input.scale_factor,
    ));
}
