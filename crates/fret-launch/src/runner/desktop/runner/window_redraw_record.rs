use fret_app::App;
use fret_core::AppWindowId;
use fret_core::time::Duration;
use fret_render::{Renderer, WgpuContext};
use fret_runtime::{FrameId, TickId};

use super::redraw_hitch::{RedrawPhase, measure_redraw_phase};
use super::{EngineFrameUpdate, WinitAppDriver};

pub(super) struct WindowRedrawRecordInput<'a, D: WinitAppDriver> {
    pub(super) app: &'a mut App,
    pub(super) driver: &'a mut D,
    pub(super) app_window: AppWindowId,
    pub(super) user: &'a mut D::WindowState,
    pub(super) context: &'a WgpuContext,
    pub(super) renderer: &'a mut Renderer,
    pub(super) scale_factor: f32,
    pub(super) tick_id: TickId,
    pub(super) frame_id: FrameId,
    pub(super) scene_ops: usize,
    pub(super) hitch_enabled: bool,
}

pub(super) fn record_window_redraw_frame<D: WinitAppDriver>(
    input: WindowRedrawRecordInput<'_, D>,
) -> (EngineFrameUpdate, Option<Duration>) {
    measure_redraw_phase(
        RedrawPhase::Record {
            scene_ops: input.scene_ops,
        },
        input.hitch_enabled,
        || {
            input.driver.record_engine_frame(
                input.app,
                input.app_window,
                input.user,
                input.context,
                input.renderer,
                input.scale_factor,
                input.tick_id,
                input.frame_id,
            )
        },
    )
}
