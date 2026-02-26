use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::super::semantics::is_descendant;
use super::before_after_metas::read_window_before_after_metas;
use super::inline_semantics_lite::stream_read_inline_semantics_lite_for_pairs;
use super::types::resolve_semantics_lite;
use super::wheel_frames_min::read_wheel_frames_min_by_window;

pub(crate) fn check_bundle_for_wheel_scroll_streaming(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let wheel_frames = read_wheel_frames_min_by_window(bundle_path)?;
    if wheel_frames.is_empty() {
        return Err(format!(
            "wheel scroll check requires at least one pointer.wheel event in the bundle: {}",
            bundle_path.display()
        ));
    }

    let windows = read_window_before_after_metas(bundle_path, &wheel_frames, warmup_frames)?;

    let mut wanted: HashMap<u64, HashSet<u64>> = HashMap::new();
    for w in &windows {
        if let (Some(b), Some(a)) = (w.before.as_ref(), w.after.as_ref()) {
            wanted.entry(w.window_id).or_default().insert(b.frame_id);
            wanted.entry(w.window_id).or_default().insert(a.frame_id);
        }
    }
    let inline_sem = stream_read_inline_semantics_lite_for_pairs(bundle_path, &wanted, test_id)?;

    let mut failures: Vec<String> = Vec::new();
    for w in windows {
        let window_id = w.window_id;
        let wheel_frame = w.wheel_frame;
        let after_frame_id = w.after.as_ref().map(|m| m.frame_id).unwrap_or(0);

        let (Some(before), Some(after)) = (w.before.as_ref(), w.after.as_ref()) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_before_or_after_snapshot"
            ));
            continue;
        };

        let Some(hit_before) = before.hit else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_hit_before"
            ));
            continue;
        };
        let Some(hit_after) = after.hit else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} error=missing_hit_after"
            ));
            continue;
        };

        let Some(before_sem) =
            resolve_semantics_lite(bundle_path, &inline_sem, before, window_id, test_id)?
        else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(after_sem) =
            resolve_semantics_lite(bundle_path, &inline_sem, after, window_id, test_id)?
        else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        let Some(target_before) = before_sem.target_node_id else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(target_after) = after_sem.target_node_id else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        if !is_descendant(hit_before, target_before, &before_sem.parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=hit_not_within_target_before hit={hit_before} target={target_before}"
            ));
            continue;
        }

        if is_descendant(hit_after, target_after, &after_sem.parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=hit_still_within_target_after hit={hit_after} target={target_after}"
            ));
        }
    }

    if failures.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("wheel scroll check failed (expected hit-test result to move after wheel)\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in failures {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

pub(crate) fn check_bundle_for_wheel_scroll_hit_changes_streaming(
    bundle_path: &Path,
    test_id: &str,
    warmup_frames: u64,
) -> Result<(), String> {
    let wheel_frames = read_wheel_frames_min_by_window(bundle_path)?;
    if wheel_frames.is_empty() {
        return Err(format!(
            "wheel scroll hit-change check requires at least one pointer.wheel event in the bundle: {}",
            bundle_path.display()
        ));
    }

    let windows = read_window_before_after_metas(bundle_path, &wheel_frames, warmup_frames)?;

    let mut wanted: HashMap<u64, HashSet<u64>> = HashMap::new();
    for w in &windows {
        if let (Some(b), Some(a)) = (w.before.as_ref(), w.after.as_ref()) {
            wanted.entry(w.window_id).or_default().insert(b.frame_id);
            wanted.entry(w.window_id).or_default().insert(a.frame_id);
        }
    }
    let inline_sem = stream_read_inline_semantics_lite_for_pairs(bundle_path, &wanted, test_id)?;

    let mut failures: Vec<String> = Vec::new();
    for w in windows {
        let window_id = w.window_id;
        let wheel_frame = w.wheel_frame;
        let after_frame_id = w.after.as_ref().map(|m| m.frame_id).unwrap_or(0);

        let (Some(before), Some(after)) = (w.before.as_ref(), w.after.as_ref()) else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_before_or_after_snapshot"
            ));
            continue;
        };

        let Some(before_sem) =
            resolve_semantics_lite(bundle_path, &inline_sem, before, window_id, test_id)?
        else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(after_sem) =
            resolve_semantics_lite(bundle_path, &inline_sem, after, window_id, test_id)?
        else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        let Some(target_before) = before_sem.target_node_id else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=missing_test_id_before"
            ));
            continue;
        };
        let Some(target_after) = after_sem.target_node_id else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=missing_test_id_after"
            ));
            continue;
        };

        let Some(hit_before) = before.hit else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} error=missing_hit_before"
            ));
            continue;
        };
        let Some(hit_after) = after.hit else {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} error=missing_hit_after"
            ));
            continue;
        };

        if !is_descendant(hit_before, target_before, &before_sem.parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} test_id={test_id} error=hit_not_within_target_before hit={hit_before} target={target_before}"
            ));
            continue;
        }
        if !is_descendant(hit_after, target_after, &after_sem.parents) {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=hit_not_within_target_after hit={hit_after} target={target_after}"
            ));
            continue;
        }

        if let (Some(a), Some(b)) = (before.vlist_offset, after.vlist_offset)
            && (a - b).abs() > 0.1
        {
            continue;
        }

        if hit_before == hit_after {
            failures.push(format!(
                "window={window_id} wheel_frame={wheel_frame} after_frame={after_frame_id} test_id={test_id} error=hit_did_not_change hit={hit_after}"
            ));
        }
    }

    if failures.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("wheel scroll hit-change check failed (expected wheel to affect the scrolled content)\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    for line in failures {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}

