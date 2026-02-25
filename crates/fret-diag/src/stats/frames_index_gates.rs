use std::path::{Path, PathBuf};

use serde_json::Value;

fn load_frames_index(bundle_path: &Path, warmup_frames: u64) -> Result<(PathBuf, Value), String> {
    let frames_index_path = crate::frames_index::ensure_frames_index_json(bundle_path, warmup_frames)?;
    let Some(frames_index) =
        crate::frames_index::read_frames_index_json_v1(&frames_index_path, warmup_frames)
    else {
        return Err(format!(
            "frames.index.json is missing or invalid (warmup_frames={warmup_frames}): {}",
            frames_index_path.display()
        ));
    };
    Ok((frames_index_path, frames_index))
}

fn window_agg_u64(window: &Value, key: &str) -> u64 {
    window
        .get("aggregates")
        .and_then(|v| v.as_object())
        .and_then(|m| m.get(key))
        .and_then(|v| v.as_u64())
        .unwrap_or(0)
}

pub(crate) fn check_frames_index_for_viewport_input_min(
    bundle_path: &Path,
    min_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let (frames_index_path, frames_index) = load_frames_index(bundle_path, warmup_frames)?;
    let windows = frames_index
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid frames.index.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut events: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    for w in windows {
        events = events.saturating_add(window_agg_u64(w, "viewport_input_events_post_warmup"));
        examined_snapshots =
            examined_snapshots.saturating_add(window_agg_u64(w, "examined_snapshots_post_warmup"));
    }

    if events >= min_events {
        return Ok(());
    }

    Err(format!(
        "expected at least {min_events} viewport input events, got {events} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}\n  frames_index: {}",
        bundle_path.display(),
        frames_index_path.display()
    ))
}

pub(crate) fn check_frames_index_for_dock_drag_min(
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let (frames_index_path, frames_index) = load_frames_index(bundle_path, warmup_frames)?;
    let windows = frames_index
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid frames.index.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut active_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    for w in windows {
        active_frames =
            active_frames.saturating_add(window_agg_u64(w, "dock_drag_active_frames_post_warmup"));
        examined_snapshots =
            examined_snapshots.saturating_add(window_agg_u64(w, "examined_snapshots_post_warmup"));
    }

    if active_frames >= min_active_frames {
        return Ok(());
    }

    Err(format!(
        "expected at least {min_active_frames} snapshots with an active dock drag, got {active_frames} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}\n  frames_index: {}",
        bundle_path.display(),
        frames_index_path.display()
    ))
}

pub(crate) fn check_frames_index_for_viewport_capture_min(
    bundle_path: &Path,
    min_active_frames: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let (frames_index_path, frames_index) = load_frames_index(bundle_path, warmup_frames)?;
    let windows = frames_index
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid frames.index.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut active_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    for w in windows {
        active_frames = active_frames.saturating_add(window_agg_u64(
            w,
            "viewport_capture_active_frames_post_warmup",
        ));
        examined_snapshots =
            examined_snapshots.saturating_add(window_agg_u64(w, "examined_snapshots_post_warmup"));
    }

    if active_frames >= min_active_frames {
        return Ok(());
    }

    Err(format!(
        "expected at least {min_active_frames} snapshots with an active viewport capture, got {active_frames} \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}\n  frames_index: {}",
        bundle_path.display(),
        frames_index_path.display()
    ))
}

pub(crate) fn check_frames_index_for_view_cache_reuse_min(
    bundle_path: &Path,
    min_reuse_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let (frames_index_path, frames_index) = load_frames_index(bundle_path, warmup_frames)?;
    let windows = frames_index
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid frames.index.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reuse_events: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut view_cache_active_snapshots: u64 = 0;
    for w in windows {
        reuse_events =
            reuse_events.saturating_add(window_agg_u64(w, "view_cache_reuse_events_post_warmup"));
        examined_snapshots =
            examined_snapshots.saturating_add(window_agg_u64(w, "examined_snapshots_post_warmup"));
        view_cache_active_snapshots = view_cache_active_snapshots.saturating_add(window_agg_u64(
            w,
            "view_cache_active_snapshots_post_warmup",
        ));
    }
    let any_view_cache_active = view_cache_active_snapshots > 0;

    if reuse_events >= min_reuse_events {
        return Ok(());
    }

    Err(format!(
        "expected at least {min_reuse_events} view-cache reuse events, got {reuse_events} \
 (any_view_cache_active={any_view_cache_active}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) \
 in bundle: {}\n  frames_index: {}",
        bundle_path.display(),
        frames_index_path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_index_window_aggregates_can_drive_basic_gates() {
        let mut dir = std::env::temp_dir();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        dir.push(format!(
            "fret-diag-frames-index-gates-test-{}-{}",
            std::process::id(),
            ts
        ));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let bundle_path = crate::resolve_bundle_artifact_path(&dir);
        std::fs::write(
            &bundle_path,
            r#"{
  "schema_version": 1,
  "tables": { "semantics": { "entries": [] } },
  "windows": [{
    "window": 1,
    "snapshots": [
      { "frame_id": 0, "debug": { "stats": { "total_time_us": 10 } } },
      { "frame_id": 5, "debug": { "viewport_input": [1,2], "docking_interaction": { "dock_drag": {} }, "stats": { "total_time_us": 20, "view_cache_active": true, "view_cache_roots_reused": 1 } } },
      { "frame_id": 6, "debug": { "viewport_input": [1], "docking_interaction": { "viewport_capture": {} }, "stats": { "total_time_us": 30, "view_cache_active": true, "view_cache_roots_reused": 2 } } }
    ]
  }]
}"#,
        )
        .expect("write bundle");

        check_frames_index_for_viewport_input_min(&bundle_path, 3, 5).expect("viewport input");
        check_frames_index_for_dock_drag_min(&bundle_path, 1, 5).expect("dock drag");
        check_frames_index_for_viewport_capture_min(&bundle_path, 1, 5).expect("viewport capture");
        check_frames_index_for_view_cache_reuse_min(&bundle_path, 3, 5).expect("view cache reuse");
    }
}
