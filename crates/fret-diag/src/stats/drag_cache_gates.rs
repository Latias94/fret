use std::path::Path;

use super::drag_cache_gates_streaming::check_bundle_for_drag_cache_root_paint_only_streaming;

pub(crate) fn check_bundle_for_drag_cache_root_paint_only(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    check_bundle_for_drag_cache_root_paint_only_streaming(bundle_path, test_id, warmup_frames)
}
