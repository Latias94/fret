use std::path::Path;

use crate::util::{now_unix_ms, write_json_value};

use super::first_wheel_frame_id_for_window;
use super::parse_redacted_len_bytes;
use super::{semantics_node_id_for_test_id, semantics_parent_map};
pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_marker_present(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_marker_present_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_marker_present_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut any_marker_present = false;
    let mut last_observed: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let marker_present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("marker_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if marker_present {
                any_marker_present = true;
            }

            let text_len_bytes = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let anchor = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("anchor"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "marker_present": marker_present,
                "text_len_bytes": text_len_bytes,
                "selection": { "anchor": anchor, "caret": caret },
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_marker_present.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_marker_present",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "any_marker_present": any_marker_present,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor marker gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if any_marker_present {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor marker gate failed (expected code_editor.torture.marker_present=true after warmup)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_marker_undo_redo_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;
    let mut max_caret: u64 = 0;

    // State machine over `marker_present`:
    // 0: waiting for insert (marker=true)
    // 1: waiting for undo (marker=false)
    // 2: waiting for redo (marker=true)
    // 3: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let marker_present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("marker_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            match state {
                0 if marker_present => state = 1,
                1 if !marker_present => state = 2,
                2 if marker_present => state = 3,
                _ => {}
            }

            let text_len_bytes = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let anchor = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("anchor"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            max_caret = max_caret.max(caret);

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "marker_present": marker_present,
                "text_len_bytes": text_len_bytes,
                "selection": { "anchor": anchor, "caret": caret },
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_marker_undo_redo.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_marker_undo_redo",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "max_caret": max_caret,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor undo/redo gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 && max_caret > 0 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor undo/redo gate failed (expected marker present, then absent, then present again; and caret to advance)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for editable baseline snapshot
    // 1: waiting for an edit to apply (rev/len increase)
    // 2: waiting for read-only snapshot
    // 3: ensure read-only does not mutate (rev/len stable for >=2 snapshots)
    // 4: success
    let mut state: u8 = 0;

    let mut edit_before_rev: u64 = 0;
    let mut edit_before_len: u64 = 0;
    let mut edit_after_rev: u64 = 0;
    let mut edit_after_len: u64 = 0;
    let mut ro_rev: u64 = 0;
    let mut ro_len: u64 = 0;
    let mut ro_samples: u64 = 0;

    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let editable = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("editable"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            match state {
                0 if enabled && editable => {
                    edit_before_rev = rev;
                    edit_before_len = len;
                    state = 1;
                }
                1 if enabled && editable && (rev > edit_before_rev || len > edit_before_len) => {
                    edit_after_rev = rev;
                    edit_after_len = len;
                    state = 2;
                }
                2 if enabled && !editable => {
                    ro_rev = rev;
                    ro_len = len;
                    ro_samples = 0;
                    state = 3;
                }
                3 if enabled && !editable => {
                    ro_samples = ro_samples.saturating_add(1);
                    if rev != ro_rev || len != ro_len {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "expected": { "buffer_revision": ro_rev, "text_len_bytes": ro_len },
                            "observed": { "buffer_revision": rev, "text_len_bytes": len },
                        }));
                        state = 4;
                        break;
                    }
                    if ro_samples >= 2 {
                        state = 4;
                        break;
                    }
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "enabled": enabled,
                "editable": editable,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "state": state,
            }));
        }
        if state == 4 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_read_only_blocks_edits.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_read_only_blocks_edits",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "edit_before": { "buffer_revision": edit_before_rev, "text_len_bytes": edit_before_len },
        "edit_after": { "buffer_revision": edit_after_rev, "text_len_bytes": edit_after_len },
        "read_only_baseline": { "buffer_revision": ro_rev, "text_len_bytes": ro_len },
        "read_only_samples": ro_samples,
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor read-only gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if let Some(violation) = violation {
        return Err(format!(
            "ui-gallery code-editor read-only gate failed (buffer mutated while interaction.editable=false)\n  bundle: {}\n  evidence: {}\n  violation: {}",
            bundle_path.display(),
            evidence_path.display(),
            violation
        ));
    }

    if state == 4 && edit_after_rev > edit_before_rev && ro_samples >= 2 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor read-only gate failed (expected: edit applies, then read-only holds revision stable)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_geom_fallbacks_low(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_geom_fallbacks_low_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_geom_fallbacks_low_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    // This gate is intentionally strict: after warmup + stats reset, code-editor-grade interactions
    // (soft-wrap, pointer drag, vertical moves) should route through renderer geometry rather than
    // the MVP monospace heuristic.
    const MAX_POINTER_FALLBACKS: u64 = 0;
    const MAX_CARET_RECT_FALLBACKS: u64 = 0;
    const MAX_VERTICAL_MOVE_FALLBACKS: u64 = 0;

    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut max_pointer_fallbacks_observed = 0u64;
    let mut max_caret_rect_fallbacks_observed = 0u64;
    let mut max_vertical_move_fallbacks_observed = 0u64;
    let mut max_pointer_fallbacks_observed_global = 0u64;
    let mut max_caret_rect_fallbacks_observed_global = 0u64;
    let mut max_vertical_move_fallbacks_observed_global = 0u64;
    let mut resets_observed = 0u64;
    let mut segment_start_observed = None::<serde_json::Value>;
    let mut prev_fallbacks = None::<(u64, u64, u64)>;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            let cache_stats = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("cache_stats"));

            let pointer_fallbacks = cache_stats
                .and_then(|v| v.get("geom_pointer_hit_test_fallbacks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret_rect_fallbacks = cache_stats
                .and_then(|v| v.get("geom_caret_rect_fallbacks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let vertical_move_fallbacks = cache_stats
                .and_then(|v| v.get("geom_vertical_move_fallbacks"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            max_pointer_fallbacks_observed_global =
                max_pointer_fallbacks_observed_global.max(pointer_fallbacks);
            max_caret_rect_fallbacks_observed_global =
                max_caret_rect_fallbacks_observed_global.max(caret_rect_fallbacks);
            max_vertical_move_fallbacks_observed_global =
                max_vertical_move_fallbacks_observed_global.max(vertical_move_fallbacks);

            let reset_detected = prev_fallbacks.is_some_and(|prev| {
                pointer_fallbacks < prev.0
                    || caret_rect_fallbacks < prev.1
                    || vertical_move_fallbacks < prev.2
            });
            if reset_detected {
                resets_observed = resets_observed.saturating_add(1);
                segment_start_observed = Some(serde_json::json!( {
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "selected_page": selected_page,
                    "soft_wrap_cols": soft_wrap_cols,
                    "geom_pointer_hit_test_fallbacks": pointer_fallbacks,
                    "geom_caret_rect_fallbacks": caret_rect_fallbacks,
                    "geom_vertical_move_fallbacks": vertical_move_fallbacks,
                }));

                // Start a new “post-reset” segment. We intentionally gate only against the latest
                // segment so scripts can isolate interactions via a "Reset stats" step.
                max_pointer_fallbacks_observed = pointer_fallbacks;
                max_caret_rect_fallbacks_observed = caret_rect_fallbacks;
                max_vertical_move_fallbacks_observed = vertical_move_fallbacks;
            } else {
                max_pointer_fallbacks_observed =
                    max_pointer_fallbacks_observed.max(pointer_fallbacks);
                max_caret_rect_fallbacks_observed =
                    max_caret_rect_fallbacks_observed.max(caret_rect_fallbacks);
                max_vertical_move_fallbacks_observed =
                    max_vertical_move_fallbacks_observed.max(vertical_move_fallbacks);
            }
            prev_fallbacks = Some((
                pointer_fallbacks,
                caret_rect_fallbacks,
                vertical_move_fallbacks,
            ));

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "geom_pointer_hit_test_fallbacks": pointer_fallbacks,
                "geom_caret_rect_fallbacks": caret_rect_fallbacks,
                "geom_vertical_move_fallbacks": vertical_move_fallbacks,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_geom_fallbacks_low.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_geom_fallbacks_low",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "max_pointer_fallbacks": MAX_POINTER_FALLBACKS,
        "max_caret_rect_fallbacks": MAX_CARET_RECT_FALLBACKS,
        "max_vertical_move_fallbacks": MAX_VERTICAL_MOVE_FALLBACKS,
        "max_pointer_fallbacks_observed": max_pointer_fallbacks_observed,
        "max_caret_rect_fallbacks_observed": max_caret_rect_fallbacks_observed,
        "max_vertical_move_fallbacks_observed": max_vertical_move_fallbacks_observed,
        "max_pointer_fallbacks_observed_global": max_pointer_fallbacks_observed_global,
        "max_caret_rect_fallbacks_observed_global": max_caret_rect_fallbacks_observed_global,
        "max_vertical_move_fallbacks_observed_global": max_vertical_move_fallbacks_observed_global,
        "resets_observed": resets_observed,
        "segment_start_observed": segment_start_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor geom fallback gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    let Some(_last) = last_observed.as_ref() else {
        return Err(format!(
            "ui-gallery code-editor geom fallback gate failed (no code_editor_torture snapshot observed after warmup)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    };

    if max_pointer_fallbacks_observed == MAX_POINTER_FALLBACKS
        && max_caret_rect_fallbacks_observed == MAX_CARET_RECT_FALLBACKS
        && max_vertical_move_fallbacks_observed == MAX_VERTICAL_MOVE_FALLBACKS
    {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor geom fallback gate failed (expected fallbacks <= {MAX_POINTER_FALLBACKS}/{MAX_CARET_RECT_FALLBACKS}/{MAX_VERTICAL_MOVE_FALLBACKS}, got pointer={max_pointer_fallbacks_observed} caret_rect={max_caret_rect_fallbacks_observed} vertical_move={max_vertical_move_fallbacks_observed})\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_folds_placeholder_present.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_present",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor folds gate failed (expected fold placeholder to be observed at least once)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_under_wrap_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_under_wrap_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_under_wrap_observed": placeholder_present_under_wrap_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-wrap gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-wrap gate requires soft_wrap_cols != null and folds_fixture=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_under_wrap_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor folds-under-wrap gate failed (expected fold placeholder to be observed at least once under soft wrap)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "preedit_active": preedit_active,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit gate requires soft_wrap_cols != null and folds_fixture=true and torture.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit gate failed (expected fold placeholder to be absent while inline preedit is active)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_some() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "preedit_active": preedit_active,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_unwrapped",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-unwrapped gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-unwrapped gate requires soft_wrap_cols == null and folds_fixture=true and torture.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor folds-under-inline-preedit-unwrapped gate failed (expected fold placeholder to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            let allow_decorations_under_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-with-decorations gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-with-decorations gate requires soft_wrap_cols != null and folds_fixture=true and torture.preedit_active=true and torture.allow_decorations_under_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor folds-under-inline-preedit-with-decorations gate failed (expected fold placeholder to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut placeholder_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !folds_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            let allow_decorations_under_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            let compose_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("compose_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !compose_inline_preedit {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            placeholder_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "compose_inline_preedit": compose_inline_preedit,
                "folds_line0_placeholder_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_folds_placeholder_present_under_inline_preedit_with_decorations_composed",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_observed": placeholder_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-with-decorations-composed gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor folds-under-inline-preedit-with-decorations-composed gate requires soft_wrap_cols != null and folds_fixture=true and torture.preedit_active=true and torture.allow_decorations_under_inline_preedit=true and torture.compose_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor folds-under-inline-preedit-with-decorations-composed gate failed (expected fold placeholder to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_torture_inlays_present.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_present",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor inlays gate failed (expected inlay fixture to be observed at least once)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_under_wrap_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_under_wrap_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_code_editor_torture_inlays_present_under_soft_wrap.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_present_under_soft_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_under_wrap_observed": inlay_present_under_wrap_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-wrap gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-wrap gate requires soft_wrap_cols != null and inlays_fixture=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_under_wrap_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor inlays-under-wrap gate failed (expected inlay text to be observed at least once under soft wrap)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit gate requires soft_wrap_cols != null and inlays_fixture=true and torture.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit gate failed (expected inlay text to be absent while inline preedit is active)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_some() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_unwrapped",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-unwrapped gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-unwrapped gate requires soft_wrap_cols == null and inlays_fixture=true and torture.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor inlays-under-inline-preedit-unwrapped gate failed (expected inlay text to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            let allow_decorations_under_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-with-decorations gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-with-decorations gate requires soft_wrap_cols != null and inlays_fixture=true and torture.preedit_active=true and torture.allow_decorations_under_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor inlays-under-inline-preedit-with-decorations gate failed (expected inlay text to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let mut examined_snapshots = 0u64;
    let mut ui_gallery_snapshots = 0u64;
    let mut matching_snapshots = 0u64;
    let mut last_observed = None::<serde_json::Value>;
    let mut inlay_present_observed = false;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);
    for w in windows {
        let window_id = w.get("window_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !inlays_fixture {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            let allow_decorations_under_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            let compose_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("compose_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !compose_inline_preedit {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_inlay_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            inlay_present_observed |= present;
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "compose_inline_preedit": compose_inline_preedit,
                "inlays_line0_present": present,
            }));
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_inlays_present_under_inline_preedit_with_decorations_composed",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_observed": inlay_present_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-with-decorations-composed gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor inlays-under-inline-preedit-with-decorations-composed gate requires soft_wrap_cols != null and inlays_fixture=true and torture.preedit_active=true and torture.allow_decorations_under_inline_preedit=true and torture.compose_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor inlays-under-inline-preedit-with-decorations-composed gate failed (expected inlay text to be observed at least once while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline snapshot (folds+inlays A)
    // 1: waiting for folds to toggle to B (folds != A)
    // 2: waiting for inlays to toggle to B (inlays != A)
    // 3: waiting for both to return to A
    // 4: success
    let mut state: u8 = 0;

    let mut baseline_folds: Option<bool> = None;
    let mut baseline_inlays: Option<bool> = None;
    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;
    let mut baseline_anchor: u64 = 0;
    let mut baseline_caret: u64 = 0;

    let mut toggled_folds: Option<bool> = None;
    let mut toggled_inlays: Option<bool> = None;
    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let code_editor = app_snapshot.and_then(|v| v.get("code_editor"));
            let folds_fixture = code_editor
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let inlays_fixture = code_editor
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let torture = code_editor.and_then(|v| v.get("torture"));
            let preedit_active = torture
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }
            let allow_decorations_under_inline_preedit = torture
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }
            let compose_inline_preedit = torture
                .and_then(|v| v.get("compose_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !compose_inline_preedit {
                continue;
            }

            let rev = torture
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = torture
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let anchor = torture
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("anchor"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = torture
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            match state {
                0 => {
                    baseline_folds = Some(folds_fixture);
                    baseline_inlays = Some(inlays_fixture);
                    baseline_rev = rev;
                    baseline_len = len;
                    baseline_anchor = anchor;
                    baseline_caret = caret;
                    state = 1;
                }
                1..=3 => {
                    if rev != baseline_rev
                        || len != baseline_len
                        || anchor != baseline_anchor
                        || caret != baseline_caret
                    {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "state": state,
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "anchor": baseline_anchor,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "anchor": anchor,
                                "caret": caret,
                            },
                            "fixtures": {
                                "folds_fixture": folds_fixture,
                                "inlays_fixture": inlays_fixture,
                            },
                        }));
                        state = 4;
                        break;
                    }

                    if toggled_folds.is_none() && baseline_folds.is_some_and(|b| folds_fixture != b)
                    {
                        toggled_folds = Some(folds_fixture);
                    }
                    if toggled_inlays.is_none()
                        && baseline_inlays.is_some_and(|b| inlays_fixture != b)
                    {
                        toggled_inlays = Some(inlays_fixture);
                    }

                    if toggled_folds.is_some() && toggled_inlays.is_some() {
                        state = 3;
                    } else if toggled_folds.is_some() || toggled_inlays.is_some() {
                        state = 2;
                    }

                    if state == 3
                        && baseline_folds.is_some_and(|b| folds_fixture == b)
                        && baseline_inlays.is_some_and(|b| inlays_fixture == b)
                    {
                        state = 4;
                        break;
                    }
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "compose_inline_preedit": compose_inline_preedit,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "anchor": anchor,
                "caret": caret,
                "state": state,
            }));
        }
        if state == 4 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_decorations_toggle_stable_under_inline_preedit_composed",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "folds_fixture": baseline_folds,
            "inlays_fixture": baseline_inlays,
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "anchor": baseline_anchor,
            "caret": baseline_caret,
        },
        "toggled": {
            "folds_fixture": toggled_folds,
            "inlays_fixture": toggled_inlays,
        },
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed decorations toggle gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 4 && violation.is_none() && toggled_folds.is_some() && toggled_inlays.is_some() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor composed decorations toggle gate failed (expected: folds_fixture and inlays_fixture both toggle at least once while compose_inline_preedit=true, then return without changing buffer_revision/text_len_bytes/anchor/caret)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-torture-viewport";
    const EXPECTED_PREEDIT: &[u8] = b"ab";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut matching_snapshots: u64 = 0;
    let mut matched_semantics_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine (mirrors the non-a11y toggle gate):
    // 0: waiting for baseline snapshot (folds+inlays A)
    // 1: waiting for folds to toggle to B (folds != A)
    // 2: waiting for inlays to toggle to B (inlays != A)
    // 3: waiting for both to return to A
    // 4: success
    let mut state: u8 = 0;

    let mut baseline_folds: Option<bool> = None;
    let mut baseline_inlays: Option<bool> = None;
    let mut toggled_folds: Option<bool> = None;
    let mut toggled_inlays: Option<bool> = None;

    let mut violation: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            let allow_decorations_under_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("allow_decorations_under_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            let compose_inline_preedit = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"))
                .and_then(|v| v.get("compose_inline_preedit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !compose_inline_preedit {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            matching_snapshots = matching_snapshots.saturating_add(1);

            // Track the toggle state machine using the fixture booleans.
            if state == 0 {
                baseline_folds = Some(folds_fixture);
                baseline_inlays = Some(inlays_fixture);
                state = 1;
            } else if state == 1 {
                if let Some(base) = baseline_folds
                    && folds_fixture != base
                {
                    toggled_folds = Some(folds_fixture);
                    state = 2;
                }
            } else if state == 2 {
                if let Some(base) = baseline_inlays
                    && inlays_fixture != base
                {
                    toggled_inlays = Some(inlays_fixture);
                    state = 3;
                }
            } else if state == 3
                && baseline_folds.is_some_and(|b| folds_fixture == b)
                && baseline_inlays.is_some_and(|b| inlays_fixture == b)
            {
                state = 4;
            }

            let viewport_node_id = semantics_node_id_for_test_id(&semantics, s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                last_observed = Some(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "selected_page": selected_page,
                    "soft_wrap_cols": soft_wrap_cols,
                    "folds_fixture": folds_fixture,
                    "inlays_fixture": inlays_fixture,
                    "preedit_active": preedit_active,
                    "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                    "compose_inline_preedit": compose_inline_preedit,
                    "state": state,
                    "semantics": "missing_viewport_node",
                }));
                continue;
            };
            matched_semantics_snapshots = matched_semantics_snapshots.saturating_add(1);

            let nodes = semantics.nodes(s).unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(&semantics, s);
            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let value = text_field
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let value_bytes = value.as_bytes();

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    if let Some((a, b)) = obj
                        .get("anchor")
                        .and_then(|a| Some((a.as_u64()?, obj.get("focus")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                    if let Some((a, b)) = obj
                        .get("start")
                        .and_then(|a| Some((a.as_u64()?, obj.get("end")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                }
                None
            });

            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });
            let sel_norm = selection.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "soft_wrap_cols": soft_wrap_cols,
                "folds_fixture": folds_fixture,
                "inlays_fixture": inlays_fixture,
                "preedit_active": preedit_active,
                "allow_decorations_under_inline_preedit": allow_decorations_under_inline_preedit,
                "compose_inline_preedit": compose_inline_preedit,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "value_len_bytes": value_bytes.len(),
                "text_selection": sel_norm.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": comp_norm.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((sel_lo, sel_hi)) = sel_norm else {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "missing_text_selection",
                        "last_observed": last_observed,
                    }));
                }
                continue;
            };
            if sel_lo != sel_hi {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "selection_not_collapsed",
                        "selection": [sel_lo, sel_hi],
                        "last_observed": last_observed,
                    }));
                }
                continue;
            }

            let Some((comp_lo, comp_hi)) = comp_norm else {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "missing_text_composition",
                        "last_observed": last_observed,
                    }));
                }
                continue;
            };
            let value_len = value_bytes.len() as u64;
            if comp_hi > value_len || comp_lo > comp_hi {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "composition_out_of_bounds",
                        "composition": [comp_lo, comp_hi],
                        "value_len_bytes": value_len,
                        "last_observed": last_observed,
                    }));
                }
                continue;
            }

            let comp_len = (comp_hi - comp_lo) as usize;
            if comp_len != EXPECTED_PREEDIT.len() {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "composition_len_mismatch",
                        "expected_len": EXPECTED_PREEDIT.len(),
                        "composition_len": comp_len,
                        "composition": [comp_lo, comp_hi],
                        "last_observed": last_observed,
                    }));
                }
                continue;
            }

            let lo = comp_lo as usize;
            let hi = comp_hi as usize;
            if hi > value_bytes.len() || &value_bytes[lo..hi] != EXPECTED_PREEDIT {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "composition_text_mismatch",
                        "expected_preedit_utf8": std::str::from_utf8(EXPECTED_PREEDIT).unwrap_or(""),
                        "observed_preedit_bytes": value_bytes.get(lo..hi).map(|s| s.to_vec()),
                        "composition": [comp_lo, comp_hi],
                        "last_observed": last_observed,
                    }));
                }
                continue;
            }

            if sel_lo != comp_hi {
                if violation.is_none() {
                    violation = Some(serde_json::json!({
                        "reason": "selection_not_at_composition_end",
                        "selection": [sel_lo, sel_hi],
                        "composition": [comp_lo, comp_hi],
                        "last_observed": last_observed,
                    }));
                }
                continue;
            }

            if state == 4 && violation.is_none() {
                break;
            }
        }
        if state == 4 && violation.is_none() {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_decorations_toggle_a11y_composition_consistent_under_inline_preedit_composed",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "matched_semantics_snapshots": matched_semantics_snapshots,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_preedit_utf8": std::str::from_utf8(EXPECTED_PREEDIT).unwrap_or(""),
        "state": state,
        "baseline": {
            "folds_fixture": baseline_folds,
            "inlays_fixture": baseline_inlays,
        },
        "toggled": {
            "folds_fixture": toggled_folds,
            "inlays_fixture": toggled_inlays,
        },
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed decorations toggle a11y gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed decorations toggle a11y gate requires soft_wrap_cols != null and torture.preedit_active=true and torture.allow_decorations_under_inline_preedit=true and torture.compose_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matched_semantics_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed decorations toggle a11y gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 4 && violation.is_none() && toggled_folds.is_some() && toggled_inlays.is_some() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor composed decorations toggle a11y gate failed (expected: folds_fixture and inlays_fixture both toggle at least once while compose_inline_preedit=true, and TextField text_composition always points at the expected preedit text)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-torture-viewport";
    const EXPECTED_PREEDIT: &[u8] = b"ab";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut any_wheel = false;
    let mut examined_windows: u64 = 0;
    let mut matched_windows: u64 = 0;
    let mut failures: Vec<serde_json::Value> = Vec::new();

    for w in windows {
        let window_id = w
            .get("window")
            .or_else(|| w.get("window_id"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let Some(wheel_frame) = first_wheel_frame_id_for_window(w) else {
            continue;
        };
        any_wheel = true;
        examined_windows = examined_windows.saturating_add(1);

        let after_frame = wheel_frame.max(warmup_frames);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);

        let mut before: Option<&serde_json::Value> = None;
        let mut before_frame: u64 = 0;
        let mut after: Option<&serde_json::Value> = None;
        let mut after_frame_id: u64 = 0;

        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < after_frame {
                if frame_id >= before_frame && frame_id < after_frame {
                    before = Some(s);
                    before_frame = frame_id;
                }
                continue;
            }
            after = Some(s);
            after_frame_id = frame_id;
            break;
        }

        let (Some(before), Some(after)) = (before, after) else {
            failures.push(serde_json::json!({
                "window": window_id,
                "wheel_frame": wheel_frame,
                "error": "missing_before_or_after_snapshot",
            }));
            continue;
        };

        let extract = |s: &serde_json::Value| -> Option<serde_json::Value> {
            let app_snapshot = s.get("app_snapshot")?;
            if app_snapshot.get("kind")?.as_str()? != "fret_ui_gallery" {
                return None;
            }
            if app_snapshot.get("selected_page")?.as_str()? != "code_editor_torture" {
                return None;
            }
            if app_snapshot
                .get("code_editor")?
                .get("soft_wrap_cols")?
                .is_null()
            {
                return None;
            }

            let torture = app_snapshot.get("code_editor")?.get("torture")?;
            if !(torture.get("preedit_active")?.as_bool()?) {
                return None;
            }
            if !(torture
                .get("allow_decorations_under_inline_preedit")?
                .as_bool()?)
            {
                return None;
            }
            if !(torture.get("compose_inline_preedit")?.as_bool()?) {
                return None;
            }

            let viewport_node_id = semantics_node_id_for_test_id(&semantics, s, VIEWPORT_TEST_ID)?;
            let nodes = semantics.nodes(s)?;
            if nodes.is_empty() {
                return None;
            }

            let parents = semantics_parent_map(&semantics, s);
            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }
            let text_field = text_field?;

            let value = text_field
                .get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let value_bytes = value.as_bytes();

            let selection = text_field.get("text_selection").and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    return Some((arr[0].as_u64()?, arr[1].as_u64()?));
                }
                if let Some(obj) = v.as_object() {
                    return Some((
                        obj.get("anchor").and_then(|v| v.as_u64())?,
                        obj.get("focus").and_then(|v| v.as_u64())?,
                    ));
                }
                None
            })?;
            let (sel_lo, sel_hi) = if selection.0 <= selection.1 {
                selection
            } else {
                (selection.1, selection.0)
            };

            let composition = text_field.get("text_composition").and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    return Some((arr[0].as_u64()?, arr[1].as_u64()?));
                }
                if let Some(obj) = v.as_object() {
                    if let Some((a, b)) = obj
                        .get("anchor")
                        .and_then(|a| Some((a.as_u64()?, obj.get("focus")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                    if let Some((a, b)) = obj
                        .get("start")
                        .and_then(|a| Some((a.as_u64()?, obj.get("end")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                }
                None
            })?;
            let (comp_lo, comp_hi) = if composition.0 <= composition.1 {
                composition
            } else {
                (composition.1, composition.0)
            };

            let value_len = value_bytes.len() as u64;
            if comp_hi > value_len || comp_lo > comp_hi {
                return None;
            }
            let lo = comp_lo as usize;
            let hi = comp_hi as usize;
            if hi > value_bytes.len() || &value_bytes[lo..hi] != EXPECTED_PREEDIT {
                return None;
            }
            if sel_lo != sel_hi || sel_lo != comp_hi {
                return None;
            }

            Some(serde_json::json!({
                "frame_id": s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0),
                "tick_id": s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                "value_len_bytes": value_bytes.len(),
                "text_selection": [sel_lo, sel_hi],
                "text_composition": [comp_lo, comp_hi],
                "buffer_revision": torture.get("buffer_revision").and_then(|v| v.as_u64()).unwrap_or(0),
                "text_len_bytes": torture.get("text_len_bytes").and_then(|v| v.as_u64()).unwrap_or(0),
            }))
        };

        let before_obs = extract(before);
        let after_obs = extract(after);
        let (Some(before_obs), Some(after_obs)) = (before_obs, after_obs) else {
            failures.push(serde_json::json!({
                "window": window_id,
                "wheel_frame": wheel_frame,
                "after_frame": after_frame,
                "after_frame_id": after_frame_id,
                "error": "missing_matching_before_or_after_observation",
            }));
            continue;
        };

        let before_sel = before_obs.get("text_selection").and_then(|v| v.as_array());
        let after_sel = after_obs.get("text_selection").and_then(|v| v.as_array());
        let before_comp = before_obs
            .get("text_composition")
            .and_then(|v| v.as_array());
        let after_comp = after_obs.get("text_composition").and_then(|v| v.as_array());

        let before_rev = before_obs
            .get("buffer_revision")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let after_rev = after_obs
            .get("buffer_revision")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let before_len = before_obs
            .get("text_len_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let after_len = after_obs
            .get("text_len_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if before_sel != after_sel
            || before_comp != after_comp
            || before_rev != after_rev
            || before_len != after_len
        {
            failures.push(serde_json::json!({
                "window": window_id,
                "wheel_frame": wheel_frame,
                "after_frame": after_frame,
                "after_frame_id": after_frame_id,
                "before": before_obs,
                "after": after_obs,
                "error": "selection_or_composition_or_buffer_changed",
            }));
            continue;
        }

        matched_windows = matched_windows.saturating_add(1);
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_composed_preedit_stable_after_wheel_scroll",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_windows": examined_windows,
        "matched_windows": matched_windows,
        "failures": failures,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_preedit_utf8": std::str::from_utf8(EXPECTED_PREEDIT).unwrap_or(""),
    });
    write_json_value(&evidence_path, &payload)?;

    if !any_wheel {
        return Err(format!(
            "ui-gallery code-editor composed preedit wheel gate requires at least one pointer.wheel event in the bundle\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display(),
        ));
    }

    if matched_windows > 0 && failures.is_empty() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor composed preedit wheel gate failed (expected selection+composition+buffer len/rev to be stable across a wheel scroll while inline preedit is active)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display(),
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-torture-viewport";
    const EXPECTED_PREEDIT: &[u8] = b"ab";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut matching_snapshots: u64 = 0;
    let mut state: u8 = 0;
    let mut baseline: Option<serde_json::Value> = None;
    let mut after: Option<serde_json::Value> = None;

    for w in windows {
        let window_id = w
            .get("window")
            .or_else(|| w.get("window_id"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let app_snapshot = s.get("app_snapshot");
            let kind = app_snapshot
                .and_then(|v| v.get("kind"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if kind != "fret_ui_gallery" {
                continue;
            }
            ui_gallery_snapshots = ui_gallery_snapshots.saturating_add(1);

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "code_editor_torture" {
                continue;
            }

            let soft_wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if soft_wrap_cols.is_none() {
                continue;
            }

            let torture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("torture"));
            let Some(torture) = torture else {
                continue;
            };

            let allow_decorations_under_inline_preedit = torture
                .get("allow_decorations_under_inline_preedit")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !allow_decorations_under_inline_preedit {
                continue;
            }

            let compose_inline_preedit = torture
                .get("compose_inline_preedit")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !compose_inline_preedit {
                continue;
            }

            let preedit_active = torture
                .get("preedit_active")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let rev = torture
                .get("buffer_revision")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = torture
                .get("text_len_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let selection_anchor = torture
                .get("selection")
                .and_then(|v| v.get("anchor"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let selection_caret = torture
                .get("selection")
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            matching_snapshots = matching_snapshots.saturating_add(1);

            // Extract semantics for validation (best-effort).
            let mut sem_value_len: Option<u64> = None;
            let mut sem_sel: Option<(u64, u64)> = None;
            let mut sem_comp: Option<(u64, u64)> = None;
            if let Some(viewport_node_id) =
                semantics_node_id_for_test_id(&semantics, s, VIEWPORT_TEST_ID)
            {
                let nodes = semantics.nodes(s).unwrap_or(&[]);
                if !nodes.is_empty() {
                    let parents = semantics_parent_map(&semantics, s);
                    let mut cur = viewport_node_id;
                    let mut text_field: Option<&serde_json::Value> = None;
                    for _ in 0..128 {
                        let node = nodes
                            .iter()
                            .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                        let Some(node) = node else {
                            break;
                        };
                        if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                            text_field = Some(node);
                            break;
                        }
                        let Some(parent) = parents.get(&cur).copied() else {
                            break;
                        };
                        cur = parent;
                    }

                    if let Some(text_field) = text_field {
                        let value = text_field
                            .get("value")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let value_bytes = value.as_bytes();
                        sem_value_len = Some(value_bytes.len() as u64);

                        sem_sel = text_field.get("text_selection").and_then(|v| {
                            if let Some(arr) = v.as_array()
                                && arr.len() == 2
                            {
                                return Some((arr[0].as_u64()?, arr[1].as_u64()?));
                            }
                            if let Some(obj) = v.as_object() {
                                return Some((
                                    obj.get("anchor").and_then(|v| v.as_u64())?,
                                    obj.get("focus").and_then(|v| v.as_u64())?,
                                ));
                            }
                            None
                        });

                        sem_comp =
                            text_field.get("text_composition").and_then(|v| {
                                if let Some(arr) = v.as_array()
                                    && arr.len() == 2
                                {
                                    return Some((arr[0].as_u64()?, arr[1].as_u64()?));
                                }
                                if let Some(obj) = v.as_object() {
                                    if let Some((a, b)) = obj.get("anchor").and_then(|a| {
                                        Some((a.as_u64()?, obj.get("focus")?.as_u64()?))
                                    }) {
                                        return Some((a, b));
                                    }
                                    if let Some((a, b)) = obj.get("start").and_then(|a| {
                                        Some((a.as_u64()?, obj.get("end")?.as_u64()?))
                                    }) {
                                        return Some((a, b));
                                    }
                                }
                                None
                            });

                        if let Some((a, b)) = sem_comp {
                            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                            let lo = lo as usize;
                            let hi = hi as usize;
                            if hi <= value_bytes.len()
                                && lo <= hi
                                && &value_bytes[lo..hi] != EXPECTED_PREEDIT
                            {
                                sem_comp = None;
                            }
                        }
                    }
                }
            }

            match state {
                0 => {
                    if preedit_active {
                        let Some((a, b)) = sem_comp else {
                            continue;
                        };
                        let (comp_lo, comp_hi) = if a <= b { (a, b) } else { (b, a) };
                        let Some((sa, sb)) = sem_sel else {
                            continue;
                        };
                        let (sel_lo, sel_hi) = if sa <= sb { (sa, sb) } else { (sb, sa) };
                        if sel_lo == sel_hi && sel_lo == comp_hi {
                            baseline = Some(serde_json::json!({
                                "window": window_id,
                                "frame_id": frame_id,
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "selection_anchor": selection_anchor,
                                "selection_caret": selection_caret,
                                "text_selection": [sel_lo, sel_hi],
                                "text_composition": [comp_lo, comp_hi],
                                "value_len_bytes": sem_value_len,
                            }));
                            state = 1;
                        }
                    }
                }
                1 => {
                    if !preedit_active && selection_anchor != selection_caret {
                        // Preedit cancellation must be non-mutating.
                        let Some(base) = baseline.as_ref() else {
                            continue;
                        };
                        let base_rev = base
                            .get("buffer_revision")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        let base_len = base
                            .get("text_len_bytes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        if rev != base_rev || len != base_len {
                            after = Some(serde_json::json!({
                                "window": window_id,
                                "frame_id": frame_id,
                                "error": "buffer_changed",
                                "baseline": base,
                                "after": {
                                    "buffer_revision": rev,
                                    "text_len_bytes": len,
                                    "selection_anchor": selection_anchor,
                                    "selection_caret": selection_caret,
                                    "text_selection": sem_sel.map(|(a,b)| if a <= b { [a,b] } else { [b,a] }),
                                    "text_composition": sem_comp.map(|(a,b)| if a <= b { [a,b] } else { [b,a] }),
                                    "value_len_bytes": sem_value_len,
                                }
                            }));
                            state = 2;
                            break;
                        }

                        // Composition should be cleared after a pointer-driven selection change.
                        if sem_comp.is_none() {
                            after = Some(serde_json::json!({
                                "window": window_id,
                                "frame_id": frame_id,
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "selection_anchor": selection_anchor,
                                "selection_caret": selection_caret,
                                "text_selection": sem_sel.map(|(a,b)| if a <= b { [a,b] } else { [b,a] }),
                                "text_composition": null,
                                "value_len_bytes": sem_value_len,
                            }));
                            state = 3;
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
        if state >= 2 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_torture_composed_preedit_cancels_on_drag_selection",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "state": state,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_preedit_utf8": std::str::from_utf8(EXPECTED_PREEDIT).unwrap_or(""),
        "baseline": baseline,
        "after": after,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed preedit drag-select gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor composed preedit drag-select gate requires soft_wrap_cols != null and torture.allow_decorations_under_inline_preedit=true and torture.compose_inline_preedit=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor composed preedit drag-select gate failed (expected: observe preedit composition once, then a pointer-driven drag selection cancels preedit without mutating buffer)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_word_boundary(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_word_boundary_json(&bundle, bundle_path, warmup_frames)
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_word_boundary_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_IDS: [&str; 2] = [
        "ui-gallery-code-editor-word-gate-viewport",
        "ui-gallery-code-editor-word-gate-soft-wrap-viewport",
    ];

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for caret=0 in Identifier mode
    // 1: waiting for caret=3 (Identifier splits `can't` around the apostrophe, `can|'t`)
    // 2: waiting for caret=0 in UnicodeWord mode
    // 3: waiting for caret=5 (UnicodeWord treats `can't` as a single word)
    // 4: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let nodes = semantics.nodes(s).unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(&semantics, s);
            for viewport_test_id in VIEWPORT_TEST_IDS {
                let Some(viewport_node_id) =
                    semantics_node_id_for_test_id(&semantics, s, viewport_test_id)
                else {
                    continue;
                };
                matched_snapshots = matched_snapshots.saturating_add(1);

                let mut cur = viewport_node_id;
                let mut text_field: Option<&serde_json::Value> = None;
                for _ in 0..128 {
                    let node = nodes
                        .iter()
                        .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                    let Some(node) = node else {
                        break;
                    };
                    if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                        text_field = Some(node);
                        break;
                    }
                    let Some(parent) = parents.get(&cur).copied() else {
                        break;
                    };
                    cur = parent;
                }

                let Some(text_field) = text_field else {
                    continue;
                };

                let text_selection = text_field.get("text_selection");
                let selection = text_selection.and_then(|v| {
                    if let Some(arr) = v.as_array()
                        && arr.len() == 2
                    {
                        let a = arr[0].as_u64()?;
                        let b = arr[1].as_u64()?;
                        return Some((a, b));
                    }
                    if let Some(obj) = v.as_object() {
                        let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                        let b = obj.get("focus").and_then(|v| v.as_u64())?;
                        return Some((a, b));
                    }
                    None
                });

                let focused = text_field
                    .get("flags")
                    .and_then(|v| v.get("focused"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
                last_observed = Some(serde_json::json!({
                    "window": window_id,
                    "tick_id": tick_id,
                    "frame_id": frame_id,
                    "viewport_test_id": viewport_test_id,
                    "viewport_node": viewport_node_id,
                    "text_field_node": cur,
                    "focused": focused,
                    "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                    "state": state,
                }));

                let Some((mut a, mut b)) = selection else {
                    continue;
                };
                if !focused {
                    continue;
                }
                if a > b {
                    std::mem::swap(&mut a, &mut b);
                }

                match state {
                    0 => {
                        if a == 0 && b == 0 {
                            state = 1;
                        }
                    }
                    1 => {
                        if (a == 0 || a == 3) && b == 3 || (a == 4 && b == 5) {
                            state = 2;
                        }
                    }
                    2 => {
                        if a == 0 && b == 0 {
                            state = 3;
                        }
                    }
                    3 => {
                        if (a == 0 || a == 5) && b == 5 {
                            state = 4;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_code_editor_word_boundary.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_word_boundary",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_ids": VIEWPORT_TEST_IDS,
        "expected_sequence": [
            {"selection":[0,0]},
            {"selection_any_of":[[3,3],[0,3],[4,5]]},
            {"selection":[0,0]},
            {"selection_any_of":[[5,5],[0,5]]}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor word-boundary gate requires semantics snapshots with viewport test_ids={:?} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            VIEWPORT_TEST_IDS,
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 4 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor word-boundary gate failed (expected selection sequence [0,0] -> [3,3]/[0,3]/[4,5] -> [0,0] -> [5,5]/[0,5] for can't)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_selection(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_selection_json(&bundle, bundle_path, warmup_frames)
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_selection_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-selection-gate-viewport";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine for "hello world":
    //
    // 0: waiting for caret=0 (collapsed)
    // 1: waiting for selection=0..5 ("hello", normalized)
    // 2: waiting for caret=11 (collapsed, end of string)
    // 3: waiting for selection=0..11 (select all, normalized)
    // 4: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(&semantics, s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = semantics.nodes(s).unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(&semantics, s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (lo, hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            match state {
                0 => {
                    if lo == 0 && hi == 0 {
                        state = 1;
                    }
                }
                1 => {
                    if lo == 0 && hi == 5 {
                        state = 2;
                    }
                }
                2 => {
                    if lo == 11 && hi == 11 {
                        state = 3;
                    }
                }
                3 => {
                    if lo == 0 && hi == 11 {
                        state = 4;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_code_editor_a11y_selection.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_selection",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_sequence_normalized": [
            {"min":0,"max":0},
            {"min":0,"max":5},
            {"min":11,"max":11},
            {"min":0,"max":11}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-selection gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 4 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-selection gate failed (expected selection sequence for \"hello world\": 0..0 -> 0..5 -> 11..11 -> 0..11)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_composition(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_composition_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-composition-gate-viewport";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for caret=2 (collapsed), no composition
    // 1: waiting for composition=2..4 and caret=4 (collapsed)
    // 2: waiting for caret=2 (collapsed), no composition
    // 3: waiting for selection=0..5 (no composition) OR selection=2..2 + composition=0..2
    // 4: waiting for selection=2..2 + composition=0..2
    // 5: waiting for selection=0..5 (no composition)
    // 6: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(&semantics, s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = semantics.nodes(s).unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(&semantics, s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    if let Some((a, b)) = obj
                        .get("anchor")
                        .and_then(|a| Some((a.as_u64()?, obj.get("focus")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                    if let Some((a, b)) = obj
                        .get("start")
                        .and_then(|a| Some((a.as_u64()?, obj.get("end")?.as_u64()?)))
                    {
                        return Some((a, b));
                    }
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": composition.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            match state {
                0 => {
                    if sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 1;
                    }
                }
                1 => {
                    if sel_lo == 4 && sel_hi == 4 && comp_norm == Some((2, 4)) {
                        state = 2;
                    }
                }
                2 => {
                    if sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 3;
                    }
                }
                3 => {
                    // The platform-style button sets selection 0..5 and immediately begins a
                    // selection-replacing composition. Depending on snapshot timing we may see
                    // either intermediate selection state or the composed view directly.
                    if sel_lo == 0 && sel_hi == 5 && comp_norm.is_none() {
                        state = 4;
                    } else if sel_lo == 2 && sel_hi == 2 && comp_norm == Some((0, 2)) {
                        state = 5;
                    }
                }
                4 => {
                    if sel_lo == 2 && sel_hi == 2 && comp_norm == Some((0, 2)) {
                        state = 5;
                    }
                }
                5 => {
                    if sel_lo == 0 && sel_hi == 5 && comp_norm.is_none() {
                        state = 6;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_code_editor_a11y_composition.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_composition",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_sequence_normalized": [
            {"text_selection":[2,2],"text_composition":null},
            {"text_selection":[4,4],"text_composition":[2,4]},
            {"text_selection":[2,2],"text_composition":null},
            {"text_selection":[2,2],"text_composition":[0,2]},
            {"text_selection":[0,5],"text_composition":null}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-composition gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 6 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-composition gate failed (expected selection/composition sequence: caret 2..2 (no composition) -> caret 4..4 (composition 2..4) -> caret 2..2 (no composition) -> selection-replacing composition -> cancel restores selection)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_selection_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-selection-wrap-gate-viewport";
    const WRAP_COLS: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    let mut expected_len_bytes: Option<u64> = None;

    // State machine (single long line, wrap at 80 cols):
    //
    // 0: waiting for caret=0
    // 1: waiting for caret=80 (End over visual row)
    // 2: waiting for caret=len (Ctrl+End clamps to document bounds)
    // 3: waiting for selection=0..len (Ctrl+A)
    // 4: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(&semantics, s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = semantics.nodes(s).unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(&semantics, s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let len_bytes = text_field
                .get("value")
                .and_then(|v| v.as_str())
                .and_then(|s| {
                    parse_redacted_len_bytes(s).or_else(|| {
                        let trimmed = s.trim_start();
                        if trimmed.starts_with("<redacted") {
                            return None;
                        }
                        Some(s.len() as u64)
                    })
                });
            if let Some(len_bytes) = len_bytes
                && expected_len_bytes.is_none()
            {
                expected_len_bytes = Some(len_bytes);
            }
            let Some(len_bytes) = expected_len_bytes else {
                continue;
            };
            if len_bytes <= WRAP_COLS {
                continue;
            }

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "len_bytes": len_bytes,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (lo, hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            match state {
                0 => {
                    if lo == 0 && hi == 0 {
                        state = 1;
                    }
                }
                1 => {
                    if lo == WRAP_COLS && hi == WRAP_COLS {
                        state = 2;
                    }
                }
                2 => {
                    if lo == len_bytes && hi == len_bytes {
                        state = 3;
                    }
                }
                3 => {
                    if lo == 0 && hi == len_bytes {
                        state = 4;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_code_editor_a11y_selection_wrap.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_selection_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "wrap_cols": WRAP_COLS,
        "expected_len_bytes": expected_len_bytes,
        "expected_sequence_template": [
            {"min":0,"max":0},
            {"min":WRAP_COLS,"max":WRAP_COLS},
            {"min":"len_bytes","max":"len_bytes"},
            {"min":0,"max":"len_bytes"}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-selection-wrap gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if expected_len_bytes.is_none() {
        return Err(format!(
            "ui-gallery code-editor a11y-selection-wrap gate requires a text_field semantics node with a value/len, but none was observed\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if expected_len_bytes.unwrap_or(0) <= WRAP_COLS {
        return Err(format!(
            "ui-gallery code-editor a11y-selection-wrap gate requires len_bytes > {WRAP_COLS}, but observed len_bytes={:?}\n  bundle: {}\n  evidence: {}",
            expected_len_bytes,
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 4 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-selection-wrap gate failed (expected: caret 0..0 -> caret {WRAP_COLS}..{WRAP_COLS} (End) -> caret len..len (Ctrl+End) -> selection 0..len (Ctrl+A))\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-composition-wrap-gate-viewport";
    const PREEDIT_START: u64 = 78;
    const PREEDIT_END: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    //
    // 0: waiting for caret=78 (collapsed), no composition
    // 1: waiting for caret=80 (collapsed), composition=78..80
    // 2: waiting for caret=78 (collapsed), no composition
    // 3: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(&semantics, s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = semantics.nodes(s).unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(&semantics, s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("start").and_then(|v| v.as_u64())?;
                    let b = obj.get("end").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": composition.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };
            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            match state {
                0 => {
                    if sel_lo == PREEDIT_START && sel_hi == PREEDIT_START && comp_norm.is_none() {
                        state = 1;
                    }
                }
                1 => {
                    if sel_lo == PREEDIT_END
                        && sel_hi == PREEDIT_END
                        && comp_norm == Some((PREEDIT_START, PREEDIT_END))
                    {
                        state = 2;
                    }
                }
                2 => {
                    if sel_lo == PREEDIT_START && sel_hi == PREEDIT_START && comp_norm.is_none() {
                        state = 3;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_a11y_composition_wrap.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_composition_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_sequence_normalized": [
            {"text_selection":[PREEDIT_START,PREEDIT_START],"text_composition":null},
            {"text_selection":[PREEDIT_END,PREEDIT_END],"text_composition":[PREEDIT_START,PREEDIT_END]},
            {"text_selection":[PREEDIT_START,PREEDIT_START],"text_composition":null}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-composition-wrap gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-composition-wrap gate failed (expected selection/composition sequence: caret 78..78 (no composition) -> caret 80..80 (composition 78..80) -> caret 78..78 (no composition))\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_scroll(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_scroll_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_wrap_scroll_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-composition-wrap-gate-viewport";
    const PREEDIT_START: u64 = 78;
    const PREEDIT_END: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    let mut preedit_observed_frame: Option<u64> = None;
    let mut scroll_after_preedit_frame: Option<u64> = None;
    let mut preedit_observed_after_scroll_frame: Option<u64> = None;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(&semantics, s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = semantics.nodes(s).unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(&semantics, s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("start").and_then(|v| v.as_u64())?;
                    let b = obj.get("end").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let scroll_offset_changed = s
                .get("debug")
                .and_then(|v| v.get("scroll_handle_changes"))
                .and_then(|v| v.as_array())
                .is_some_and(|changes| {
                    changes.iter().any(|c| {
                        c.get("offset_changed")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                    })
                });

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "scroll_offset_changed": scroll_offset_changed,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": composition.map(|(a,b)| serde_json::json!([a,b])),
                "preedit_observed_frame": preedit_observed_frame,
                "scroll_after_preedit_frame": scroll_after_preedit_frame,
                "preedit_observed_after_scroll_frame": preedit_observed_after_scroll_frame,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };
            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            let is_expected_preedit = sel_lo == PREEDIT_END
                && sel_hi == PREEDIT_END
                && comp_norm == Some((PREEDIT_START, PREEDIT_END));

            if preedit_observed_frame.is_none() && is_expected_preedit {
                preedit_observed_frame = Some(frame_id);
            }

            if scroll_after_preedit_frame.is_none()
                && preedit_observed_frame.is_some()
                && scroll_offset_changed
            {
                scroll_after_preedit_frame = Some(frame_id);
            }

            if preedit_observed_after_scroll_frame.is_none()
                && scroll_after_preedit_frame.is_some()
                && is_expected_preedit
            {
                preedit_observed_after_scroll_frame = Some(frame_id);
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_a11y_composition_wrap_scroll.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_composition_wrap_scroll",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "preedit_observed_frame": preedit_observed_frame,
        "scroll_after_preedit_frame": scroll_after_preedit_frame,
        "preedit_observed_after_scroll_frame": preedit_observed_after_scroll_frame,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_preedit": {"text_selection":[PREEDIT_END,PREEDIT_END],"text_composition":[PREEDIT_START,PREEDIT_END]},
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-composition-wrap-scroll gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if preedit_observed_frame.is_none() {
        return Err(format!(
            "ui-gallery code-editor a11y-composition-wrap-scroll gate requires observing an inline preedit (selection 80..80, composition 78..80), but none was observed\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if scroll_after_preedit_frame.is_none() {
        return Err(format!(
            "ui-gallery code-editor a11y-composition-wrap-scroll gate requires observing a scroll offset change after preedit is active, but none was observed\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if preedit_observed_after_scroll_frame.is_some() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-composition-wrap-scroll gate failed (expected preedit to remain active after scroll while composing)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_drag(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_code_editor_a11y_composition_drag_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_code_editor_a11y_composition_drag_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-code-editor-a11y-composition-drag-gate-viewport";
    const PREEDIT_START: u64 = 78;
    const PREEDIT_END: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle artifact: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    //
    // 0: waiting for caret=78 (collapsed), no composition
    // 1: waiting for caret=80 (collapsed), composition=78..80
    // 2: waiting for a non-collapsed selection, no composition (drag selection clears preedit deterministically)
    // 3: success
    let mut state: u8 = 0;

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let snaps = w
            .get("snapshots")
            .and_then(|v| v.as_array())
            .map_or(&[][..], |v| v);
        for s in snaps {
            let frame_id = s.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
            if frame_id < warmup_frames {
                continue;
            }
            examined_snapshots = examined_snapshots.saturating_add(1);

            let viewport_node_id = semantics_node_id_for_test_id(&semantics, s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes = semantics.nodes(s).unwrap_or(&[]);
            if nodes.is_empty() {
                continue;
            }

            let parents = semantics_parent_map(&semantics, s);

            let mut cur = viewport_node_id;
            let mut text_field: Option<&serde_json::Value> = None;
            for _ in 0..128 {
                let node = nodes
                    .iter()
                    .find(|n| n.get("id").and_then(|v| v.as_u64()) == Some(cur));
                let Some(node) = node else {
                    break;
                };
                if node.get("role").and_then(|v| v.as_str()) == Some("text_field") {
                    text_field = Some(node);
                    break;
                }
                let Some(parent) = parents.get(&cur).copied() else {
                    break;
                };
                cur = parent;
            }

            let Some(text_field) = text_field else {
                continue;
            };

            let text_selection = text_field.get("text_selection");
            let selection = text_selection.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("anchor").and_then(|v| v.as_u64())?;
                    let b = obj.get("focus").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let text_composition = text_field.get("text_composition");
            let composition = text_composition.and_then(|v| {
                if let Some(arr) = v.as_array()
                    && arr.len() == 2
                {
                    let a = arr[0].as_u64()?;
                    let b = arr[1].as_u64()?;
                    return Some((a, b));
                }
                if let Some(obj) = v.as_object() {
                    let a = obj.get("start").and_then(|v| v.as_u64())?;
                    let b = obj.get("end").and_then(|v| v.as_u64())?;
                    return Some((a, b));
                }
                None
            });

            let focused = text_field
                .get("flags")
                .and_then(|v| v.get("focused"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let tick_id = s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0);
            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "viewport_node": viewport_node_id,
                "text_field_node": cur,
                "focused": focused,
                "text_selection": selection.map(|(a,b)| serde_json::json!([a,b])),
                "text_composition": composition.map(|(a,b)| serde_json::json!([a,b])),
                "state": state,
            }));

            let Some((anchor, focus)) = selection else {
                continue;
            };
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };
            let comp_norm = composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

            match state {
                0 => {
                    if sel_lo == PREEDIT_START && sel_hi == PREEDIT_START && comp_norm.is_none() {
                        state = 1;
                    }
                }
                1 => {
                    if sel_lo == PREEDIT_END
                        && sel_hi == PREEDIT_END
                        && comp_norm == Some((PREEDIT_START, PREEDIT_END))
                    {
                        state = 2;
                    }
                }
                2 => {
                    if sel_lo != sel_hi && comp_norm.is_none() {
                        state = 3;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_code_editor_a11y_composition_drag.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_code_editor_a11y_composition_drag",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "preedit": {"start": PREEDIT_START, "end": PREEDIT_END},
        "expected_sequence": [
            {"text_selection":[PREEDIT_START,PREEDIT_START],"text_composition":null},
            {"text_selection":[PREEDIT_END,PREEDIT_END],"text_composition":[PREEDIT_START,PREEDIT_END]},
            {"text_selection":"non-collapsed","text_composition":null}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery code-editor a11y-composition-drag gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery code-editor a11y-composition-drag gate failed (expected: caret 78..78 (no composition) -> caret 80..80 (composition 78..80) -> non-collapsed selection (no composition))\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}
