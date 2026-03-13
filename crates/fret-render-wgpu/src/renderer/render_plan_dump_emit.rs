#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
fn parse_env_u64(name: &str) -> Option<u64> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
}

#[cfg(not(target_arch = "wasm32"))]
fn dump_dir_from_env() -> PathBuf {
    std::env::var_os("FRET_RENDERPLAN_DUMP_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".fret").join("renderplan"))
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn should_emit_render_plan_dump(frame_index: u64) -> bool {
    if std::env::var_os("FRET_RENDERPLAN_DUMP")
        .filter(|value| !value.is_empty())
        .is_none()
    {
        return false;
    }

    if let Some(frame) = parse_env_u64("FRET_RENDERPLAN_DUMP_FRAME") {
        return frame_index == frame;
    }

    let after = parse_env_u64("FRET_RENDERPLAN_DUMP_AFTER_FRAMES").unwrap_or(1);
    if frame_index < after {
        return false;
    }

    if let Some(every) = parse_env_u64("FRET_RENDERPLAN_DUMP_EVERY") {
        return every > 0 && (frame_index - after).is_multiple_of(every);
    }

    static DUMPED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    !DUMPED.swap(true, std::sync::atomic::Ordering::SeqCst)
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn emit_render_plan_dump_json(frame_index: u64, bytes: &[u8]) {
    let dir = dump_dir_from_env();
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join(format!("renderplan.frame{frame_index}.json"));
    let _ = std::fs::write(file, bytes);
}

#[cfg(target_arch = "wasm32")]
pub(super) fn should_emit_render_plan_dump(_frame_index: u64) -> bool {
    false
}

#[cfg(target_arch = "wasm32")]
pub(super) fn emit_render_plan_dump_json(_frame_index: u64, _bytes: &[u8]) {}
