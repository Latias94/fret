use super::debug_dump_gate::{DumpFrameEnv, emit_dump_file, should_emit_dump_frame};
use std::sync::atomic::AtomicBool;

const RENDER_PLAN_DUMP_ENV: DumpFrameEnv = DumpFrameEnv::new(
    "FRET_RENDERPLAN_DUMP",
    "FRET_RENDERPLAN_DUMP_FRAME",
    "FRET_RENDERPLAN_DUMP_AFTER_FRAMES",
    "FRET_RENDERPLAN_DUMP_EVERY",
    "FRET_RENDERPLAN_DUMP_DIR",
    "renderplan",
);
static RENDER_PLAN_DUMPED: AtomicBool = AtomicBool::new(false);

pub(super) fn should_emit_render_plan_dump(frame_index: u64) -> bool {
    should_emit_dump_frame(frame_index, RENDER_PLAN_DUMP_ENV, &RENDER_PLAN_DUMPED)
}

pub(super) fn emit_render_plan_dump_json(frame_index: u64, bytes: &[u8]) {
    emit_dump_file(
        RENDER_PLAN_DUMP_ENV,
        format!("renderplan.frame{frame_index}.json"),
        bytes,
    );
}
