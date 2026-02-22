use std::path::Path;

use crate::util::{now_unix_ms, write_json_value};

use super::parse_redacted_len_bytes;
use super::{semantics_node_id_for_test_id, semantics_parent_map};

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
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
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let editable = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("editable"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
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
        evidence_dir.join("check.ui_gallery_markdown_editor_source_read_only_blocks_edits.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_read_only_blocks_edits",
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
            "ui-gallery markdown editor read-only gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if let Some(violation) = violation {
        return Err(format!(
            "ui-gallery markdown editor read-only gate failed (buffer mutated while interaction.editable=false)\n  bundle: {}\n  evidence: {}\n  violation: {}",
            bundle_path.display(),
            evidence_path.display(),
            violation
        ));
    }

    if state == 4 && edit_after_rev > edit_before_rev && ro_samples >= 2 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor read-only gate failed (expected: edit applies, then read-only holds revision stable)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }
    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;
    let mut disabled_semantics_matched: u64 = 0;
    let mut disabled_semantics_checked: u64 = 0;
    let mut disabled_focus_violation: Option<serde_json::Value> = None;
    let mut disabled_composition_violation: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for editable baseline snapshot
    // 1: waiting for a disabled snapshot
    // 2: ensure disabled does not mutate (rev/len/caret stable for >=2 snapshots)
    // 3: success
    let mut state: u8 = 0;

    let mut edit_before_rev: u64 = 0;
    let mut edit_before_len: u64 = 0;
    let mut edit_before_caret: u64 = 0;

    let mut disabled_rev: u64 = 0;
    let mut disabled_len: u64 = 0;
    let mut disabled_caret: u64 = 0;
    let mut disabled_samples: u64 = 0;

    let mut violation: Option<serde_json::Value> = None;
    let mut failed: bool = false;

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
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let editable = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("editable"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            if !enabled {
                disabled_semantics_checked = disabled_semantics_checked.saturating_add(1);

                let viewport_node_id =
                    semantics_node_id_for_test_id(&semantics, s, VIEWPORT_TEST_ID);
                if let Some(viewport_node_id) = viewport_node_id {
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
                            disabled_semantics_matched =
                                disabled_semantics_matched.saturating_add(1);

                            let focused = text_field
                                .get("flags")
                                .and_then(|v| v.get("focused"))
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);

                            if focused && disabled_focus_violation.is_none() {
                                disabled_focus_violation = Some(serde_json::json!({
                                    "window": window_id,
                                    "tick_id": tick_id,
                                    "frame_id": frame_id,
                                    "viewport_test_id": VIEWPORT_TEST_ID,
                                    "viewport_node": viewport_node_id,
                                    "text_field_node": cur,
                                    "focused": focused,
                                }));
                                failed = true;
                            }

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
                            let comp_norm =
                                composition.map(|(a, b)| if a <= b { (a, b) } else { (b, a) });

                            if comp_norm.is_some() && disabled_composition_violation.is_none() {
                                disabled_composition_violation = Some(serde_json::json!({
                                    "window": window_id,
                                    "tick_id": tick_id,
                                    "frame_id": frame_id,
                                    "viewport_test_id": VIEWPORT_TEST_ID,
                                    "viewport_node": viewport_node_id,
                                    "text_field_node": cur,
                                    "text_composition": comp_norm.map(|(a,b)| [a,b]),
                                }));
                                failed = true;
                            }
                        }
                    }
                }
            }

            match state {
                0 if enabled && editable => {
                    edit_before_rev = rev;
                    edit_before_len = len;
                    edit_before_caret = caret;
                    state = 1;
                }
                1 if !enabled => {
                    disabled_rev = rev;
                    disabled_len = len;
                    disabled_caret = caret;
                    disabled_samples = 0;
                    state = 2;
                }
                2 if !enabled => {
                    disabled_samples = disabled_samples.saturating_add(1);
                    if rev != disabled_rev || len != disabled_len || caret != disabled_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "expected": {
                                "buffer_revision": disabled_rev,
                                "text_len_bytes": disabled_len,
                                "selection_caret": disabled_caret
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "selection_caret": caret
                            },
                        }));
                        state = 3;
                        break;
                    }
                    if disabled_samples >= 2 {
                        state = 3;
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
                "selection_caret": caret,
                "disabled_semantics_matched": disabled_semantics_matched,
                "state": state,
            }));
        }
        if state == 3 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_disabled_blocks_edits.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_disabled_blocks_edits",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "editable_baseline": {
            "buffer_revision": edit_before_rev,
            "text_len_bytes": edit_before_len,
            "selection_caret": edit_before_caret
        },
        "disabled_baseline": {
            "buffer_revision": disabled_rev,
            "text_len_bytes": disabled_len,
            "selection_caret": disabled_caret,
            "samples": disabled_samples
        },
        "disabled_semantics_checked": disabled_semantics_checked,
        "disabled_semantics_matched": disabled_semantics_matched,
        "disabled_focus_violation": disabled_focus_violation,
        "disabled_composition_violation": disabled_composition_violation,
        "violation": violation,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor disabled gate requires fret_ui_gallery app snapshots after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if violation.is_some() {
        return Err(format!(
            "ui-gallery markdown editor disabled gate observed mutation while disabled\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if failed {
        return Err(format!(
            "ui-gallery markdown editor disabled gate observed focus/composition while disabled\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 && disabled_samples >= 2 && disabled_semantics_matched > 0 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor disabled gate failed (expected: disabled holds revision/len/caret stable, and is not focused with no composition)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

// --- Extracted from `stats.rs` to keep the main module smaller. ---

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline snapshot (wrap A)
    // 1: waiting for wrap to toggle to B (wrap != A)
    // 2: waiting for wrap to return to A
    // 3: success
    let mut state: u8 = 0;

    let mut baseline_wrap_cols: Option<u64> = None;
    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;
    let mut baseline_caret: u64 = 0;

    let mut toggled_wrap_cols: Option<u64> = None;

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
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !enabled {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            match state {
                0 => {
                    baseline_wrap_cols = wrap_cols;
                    baseline_rev = rev;
                    baseline_len = len;
                    baseline_caret = caret;
                    state = 1;
                }
                1 if wrap_cols != baseline_wrap_cols => {
                    toggled_wrap_cols = wrap_cols;
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "toggled",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "wrap": { "baseline_soft_wrap_cols": baseline_wrap_cols, "observed_soft_wrap_cols": wrap_cols },
                        }));
                        state = 3;
                        break;
                    }
                    state = 2;
                }
                2 if wrap_cols == baseline_wrap_cols => {
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "returned",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "wrap": { "baseline_soft_wrap_cols": baseline_wrap_cols, "observed_soft_wrap_cols": wrap_cols },
                        }));
                        state = 3;
                        break;
                    }
                    state = 3;
                    break;
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": wrap_cols,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "caret": caret,
                "state": state,
            }));
        }
        if state == 3 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_soft_wrap_toggle_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_soft_wrap_toggle_stable",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "soft_wrap_cols": baseline_wrap_cols,
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "caret": baseline_caret,
        },
        "toggled_soft_wrap_cols": toggled_wrap_cols,
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor soft-wrap toggle gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if let Some(violation) = violation {
        return Err(format!(
            "ui-gallery markdown editor soft-wrap toggle gate failed (caret/rev/len changed across wrap toggles)\n  bundle: {}\n  evidence: {}\n  violation: {}",
            bundle_path.display(),
            evidence_path.display(),
            violation
        ));
    }

    if state == 3 && toggled_wrap_cols != baseline_wrap_cols {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor soft-wrap toggle gate failed (expected: wrap toggles twice, and caret/rev/len remain stable)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_word_boundary(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_word_boundary_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_word_boundary_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }
    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for caret=0 (collapsed)
    // 1: waiting for caret=5 (collapsed) (UnicodeWord treats `can't` as a single word)
    // 2: success
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
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            match state {
                0 => {
                    if focused && sel_lo == 0 && sel_hi == 0 {
                        state = 1;
                    }
                }
                1 => {
                    if focused && (sel_lo == 0 || sel_lo == 5) && sel_hi == 5 {
                        state = 2;
                        break;
                    }
                }
                _ => {}
            }
        }
        if state == 2 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_markdown_editor_word_boundary.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_word_boundary",
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
            {"text_selection":[0,0]},
            {"text_selection_any_of":[[5,5],[0,5]]}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor word-boundary gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 2 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor word-boundary gate failed (expected caret to move 0 -> 5 for can't)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_web_ime_bridge_enabled(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_web_ime_bridge_enabled_json(&bundle, bundle_path, warmup_frames)
}

pub(crate) fn check_bundle_for_ui_gallery_web_ime_bridge_enabled_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut satisfied: bool = false;
    let mut observed_focus_true: bool = false;
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

            let selected_page = app_snapshot
                .and_then(|v| v.get("selected_page"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let web_ime = s
                .get("debug")
                .and_then(|v| v.get("web_ime_bridge"))
                .and_then(|v| v.as_object());
            let Some(web_ime) = web_ime else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let enabled = web_ime
                .get("enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let mount_kind = web_ime.get("mount_kind").and_then(|v| v.as_str());
            let position_mode = web_ime.get("position_mode").and_then(|v| v.as_str());
            let textarea_has_focus = web_ime.get("textarea_has_focus").and_then(|v| v.as_bool());
            let cursor_area_set_seen = web_ime
                .get("cursor_area_set_seen")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let last_cursor_area = web_ime.get("last_cursor_area").cloned();

            observed_focus_true |= textarea_has_focus == Some(true);

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "selected_page": selected_page,
                "enabled": enabled,
                "textarea_has_focus": textarea_has_focus,
                "mount_kind": mount_kind,
                "position_mode": position_mode,
                "cursor_area_set_seen": cursor_area_set_seen,
                "last_cursor_area": last_cursor_area,
            }));

            if enabled
                && mount_kind.is_some()
                && position_mode.is_some()
                && textarea_has_focus.is_some()
                && cursor_area_set_seen > 0
            {
                satisfied = true;
                break;
            }
        }
        if satisfied {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.ui_gallery_web_ime_bridge_enabled.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_web_ime_bridge_enabled",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matched_snapshots": matched_snapshots,
        "satisfied": satisfied,
        "observed_focus_true": observed_focus_true,
        "last_observed": last_observed,
        "expected": {
            "selected_page": "markdown_editor_source",
            "web_ime_bridge": {
                "enabled": true,
                "mount_kind": "non_null",
                "position_mode": "non_null",
                "textarea_has_focus": "some(true_or_false)",
                "cursor_area_set_seen_gt": 0
            }
        }
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery web-ime bridge gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery web-ime bridge gate requires debug.web_ime_bridge snapshots on selected_page=markdown_editor_source after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}, ui_gallery_snapshots={ui_gallery_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if satisfied {
        return Ok(());
    }

    Err(format!(
        "ui-gallery web-ime bridge gate failed (expected bridge to be enabled with mount/position metadata and cursor area updates; focus may be best-effort)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

// ADR 0179: triple-click should select the logical line.
pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";
    const EXPECTED_LINE_END_ANY_OF: [u64; 2] = [6, 7]; // "hello\n" (LF) or "hello\r\n" (CRLF)

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for caret=0 (collapsed)
    // 1: waiting for selection=0..line_end (including the trailing newline when present)
    // 2: success
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
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };

            match state {
                0 => {
                    if focused && sel_lo == 0 && sel_hi == 0 {
                        state = 1;
                    }
                }
                1 => {
                    if focused && sel_lo == 0 && EXPECTED_LINE_END_ANY_OF.contains(&sel_hi) {
                        state = 2;
                        break;
                    }
                }
                _ => {}
            }
        }
        if state == 2 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_line_boundary_triple_click.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_line_boundary_triple_click",
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
            {"text_selection":[0,0]},
            {"text_selection_any_of":[[0,6],[0,7]]}
        ],
        "expected_line_end_any_of": EXPECTED_LINE_END_ANY_OF,
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor line-boundary (triple-click) gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 2 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor line-boundary (triple-click) gate failed (expected selection to expand 0..line_end including trailing newline)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
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
                    if focused && sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 1;
                    }
                }
                1 => {
                    if focused && sel_lo == 4 && sel_hi == 4 && comp_norm == Some((2, 4)) {
                        state = 2;
                    }
                }
                2 => {
                    if focused && sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 3;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_a11y_composition.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_a11y_composition",
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
            {"text_selection":[2,2],"text_composition":null}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor a11y-composition gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor a11y-composition gate failed (expected: caret 2, then composition 2..4 with caret 4, then clear back to caret 2)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";
    const EXPECTED_SOFT_WRAP_COLS: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    let mut saw_soft_wrap: bool = false;
    let mut last_soft_wrap_cols: Option<u64> = None;

    // State machine:
    // 0: waiting for caret=2 (collapsed), no composition
    // 1: waiting for composition=2..4 and caret=4 (collapsed)
    // 2: waiting for caret=2 (collapsed), no composition
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

            if let Some(app_snapshot) = s.get("app_snapshot") {
                let kind = app_snapshot
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let selected_page = app_snapshot
                    .get("selected_page")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if kind == "fret_ui_gallery" && selected_page == "markdown_editor_source" {
                    let cols = app_snapshot
                        .get("code_editor")
                        .and_then(|v| v.get("soft_wrap_cols"))
                        .and_then(|v| v.as_u64());
                    last_soft_wrap_cols = cols;
                    if cols == Some(EXPECTED_SOFT_WRAP_COLS) {
                        saw_soft_wrap = true;
                    }
                }
            }

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
                "soft_wrap_cols": last_soft_wrap_cols,
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
                    if focused && sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 1;
                    }
                }
                1 => {
                    if focused && sel_lo == 4 && sel_hi == 4 && comp_norm == Some((2, 4)) {
                        state = 2;
                    }
                }
                2 => {
                    if focused && sel_lo == 2 && sel_hi == 2 && comp_norm.is_none() {
                        state = 3;
                    }
                }
                _ => {}
            }
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_markdown_editor_source_a11y_composition_soft_wrap.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_a11y_composition_soft_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_soft_wrap_cols": EXPECTED_SOFT_WRAP_COLS,
        "saw_soft_wrap": saw_soft_wrap,
        "expected_sequence_normalized": [
            {"text_selection":[2,2],"text_composition":null},
            {"text_selection":[4,4],"text_composition":[2,4]},
            {"text_selection":[2,2],"text_composition":null}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor a11y-composition (soft-wrap) gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if !saw_soft_wrap {
        return Err(format!(
            "ui-gallery markdown editor a11y-composition (soft-wrap) gate requires observing soft_wrap_cols={EXPECTED_SOFT_WRAP_COLS} in app snapshots, but none were observed\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor a11y-composition (soft-wrap) gate failed (expected: caret 2, then composition 2..4 with caret 4, then clear back to caret 2)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    const VIEWPORT_TEST_ID: &str = "ui-gallery-markdown-editor-viewport";
    const WRAP_COLS: u64 = 80;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let semantics = crate::json_bundle::SemanticsResolver::new(bundle);

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut matched_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for wrap=80 and caret=0 (collapsed)
    // 1: waiting for caret=80 (End over visual row)
    // 2: waiting for caret=81 and len to increase by 1 (typed a single byte)
    // 3: waiting for caret=0 (Ctrl+Home)
    // 4: waiting for caret=80 again (End over visual row) with edited len
    // 5: success
    let mut state: u8 = 0;

    let mut baseline_len_bytes: Option<u64> = None;
    let mut edited_len_bytes: Option<u64> = None;

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
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            if wrap_cols != WRAP_COLS {
                continue;
            }

            let viewport_node_id = semantics_node_id_for_test_id(&semantics, s, VIEWPORT_TEST_ID);
            let Some(viewport_node_id) = viewport_node_id else {
                continue;
            };
            matched_snapshots = matched_snapshots.saturating_add(1);

            let nodes: &[serde_json::Value] = semantics.nodes(s).unwrap_or(&[]);
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

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": wrap_cols,
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
            if !focused {
                continue;
            }
            let (sel_lo, sel_hi) = if anchor <= focus {
                (anchor, focus)
            } else {
                (focus, anchor)
            };
            if sel_lo != sel_hi {
                continue;
            }
            let caret = sel_lo;

            let Some(len_bytes) = len_bytes else {
                continue;
            };
            if len_bytes <= WRAP_COLS {
                continue;
            }

            match state {
                0 => {
                    if caret == 0 {
                        baseline_len_bytes = Some(len_bytes);
                        state = 1;
                    }
                }
                1 => {
                    if caret == WRAP_COLS {
                        state = 2;
                    }
                }
                2 => {
                    if caret == WRAP_COLS + 1
                        && let Some(base) = baseline_len_bytes
                        && len_bytes == base.saturating_add(1)
                    {
                        edited_len_bytes = Some(len_bytes);
                        state = 3;
                    }
                }
                3 => {
                    if caret == 0 {
                        state = 4;
                    }
                }
                4 => {
                    if caret == WRAP_COLS && edited_len_bytes == Some(len_bytes) {
                        state = 5;
                        break;
                    }
                }
                _ => {}
            }
        }
        if state == 5 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join(
        "check.ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matched_snapshots": matched_snapshots,
        "state": state,
        "baseline_len_bytes": baseline_len_bytes,
        "edited_len_bytes": edited_len_bytes,
        "last_observed": last_observed,
        "viewport_test_id": VIEWPORT_TEST_ID,
        "expected_sequence": [
            {"text_selection":[0,0]},
            {"text_selection":[WRAP_COLS,WRAP_COLS]},
            {"text_selection":[WRAP_COLS+1,WRAP_COLS+1], "len_bytes":"baseline+1"},
            {"text_selection":[0,0]},
            {"text_selection":[WRAP_COLS,WRAP_COLS], "len_bytes":"baseline+1"}
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor soft-wrap editing gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matched_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor soft-wrap editing gate requires semantics snapshots with viewport test_id={VIEWPORT_TEST_ID} and soft_wrap_cols=80 after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 5 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor soft-wrap editing gate failed (expected: caret 0 -> 80 -> 81 (len+1) -> 0 -> 80 under soft_wrap_cols=80)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_json(
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
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
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
        evidence_dir.join("check.ui_gallery_markdown_editor_source_folds_placeholder_present.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_folds_placeholder_present",
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
            "ui-gallery markdown editor folds gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor folds gate failed (expected fold placeholder to be observed at least once)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap_json(
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
            if selected_page != "markdown_editor_source" {
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
                .and_then(|v| v.get("markdown_editor_source"))
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
        "check.ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_folds_placeholder_present_under_soft_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "placeholder_present_observed": placeholder_present_under_wrap_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds-under-wrap gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds-under-wrap gate requires soft_wrap_cols != null and folds_fixture=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_under_wrap_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor folds-under-wrap gate failed (expected fold placeholder to be observed at least once under soft wrap)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit_json(
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
            if selected_page != "markdown_editor_source" {
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
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
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
        "check.ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit.json",
    );
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_folds_placeholder_absent_under_inline_preedit",
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
            "ui-gallery markdown editor folds-under-inline-preedit gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds-under-inline-preedit gate requires soft_wrap_cols != null and folds_fixture=true and markdown_editor_source.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if placeholder_present_observed {
        return Err(format!(
            "ui-gallery markdown editor folds-under-inline-preedit gate failed (expected fold placeholder to be absent while inline preedit is active)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_present(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_json(
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
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_present"))
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
        evidence_dir.join("check.ui_gallery_markdown_editor_source_inlays_present.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_inlays_present",
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
            "ui-gallery markdown editor inlays gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor inlays gate failed (expected inlay to be observed at least once)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap_json(
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
            if selected_page != "markdown_editor_source" {
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
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_present"))
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
        .join("check.ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_inlays_present_under_soft_wrap",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "matching_snapshots": matching_snapshots,
        "inlay_present_observed": inlay_present_under_wrap_observed,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays-under-wrap gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays-under-wrap gate requires soft_wrap_cols != null and inlays_fixture=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_under_wrap_observed {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor inlays-under-wrap gate failed (expected inlay to be observed at least once under soft wrap)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit_json(
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
            if selected_page != "markdown_editor_source" {
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
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !preedit_active {
                continue;
            }

            matching_snapshots = matching_snapshots.saturating_add(1);

            let present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_present"))
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
        .join("check.ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_inlays_absent_under_inline_preedit",
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
            "ui-gallery markdown editor inlays-under-inline-preedit gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if matching_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays-under-inline-preedit gate requires soft_wrap_cols != null and inlays_fixture=true and markdown_editor_source.preedit_active=true after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if inlay_present_observed {
        return Err(format!(
            "ui-gallery markdown editor inlays-under-inline-preedit gate failed (expected inlays to be absent while inline preedit is active)\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    Ok(())
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_toggle_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline snapshot (folds A)
    // 1: waiting for folds to toggle to B (folds != A)
    // 2: waiting for folds to return to A
    // 3: success
    let mut state: u8 = 0;

    let mut baseline_folds: Option<bool> = None;
    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;
    let mut baseline_caret: u64 = 0;

    let mut toggled_folds: Option<bool> = None;
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
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !enabled {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if wrap_cols.is_some() {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            match state {
                0 => {
                    baseline_folds = Some(folds_fixture);
                    baseline_rev = rev;
                    baseline_len = len;
                    baseline_caret = caret;
                    state = 1;
                }
                1 if Some(folds_fixture) != baseline_folds => {
                    toggled_folds = Some(folds_fixture);
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "toggled",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "folds_fixture": { "baseline": baseline_folds, "observed": folds_fixture },
                        }));
                        state = 3;
                        break;
                    }
                    state = 2;
                }
                2 if Some(folds_fixture) == baseline_folds => {
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "returned",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "folds_fixture": { "baseline": baseline_folds, "observed": folds_fixture },
                        }));
                        state = 3;
                        break;
                    }
                    state = 3;
                    break;
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": wrap_cols,
                "folds_fixture": folds_fixture,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "caret": caret,
                "state": state,
            }));
        }
        if state == 3 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_folds_toggle_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_folds_toggle_stable",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "folds_fixture": baseline_folds,
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "caret": baseline_caret,
        },
        "toggled_folds_fixture": toggled_folds,
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds toggle gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 && violation.is_none() && toggled_folds.is_some() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor folds toggle gate failed (expected: folds_fixture toggles and returns without changing buffer_revision/text_len_bytes/caret)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline snapshot (folds off; caret inside fold span)
    // 1: waiting for folds on (caret clamped to fold start; buffer unchanged)
    // 2: success
    let mut state: u8 = 0;

    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;
    let mut baseline_caret: u64 = 0;

    let mut fold_span_start: u64 = 0;
    let mut fold_span_end: u64 = 0;

    let mut clamp_observed: Option<serde_json::Value> = None;
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
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !enabled {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if wrap_cols.is_some() {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if preedit_active {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let fold_span = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("fixture_span_line0"));
            let placeholder_present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("folds"))
                .and_then(|v| v.get("line0_placeholder_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let span_start = fold_span
                .and_then(|v| v.get("start"))
                .and_then(|v| v.as_u64());
            let span_end = fold_span
                .and_then(|v| v.get("end"))
                .and_then(|v| v.as_u64());

            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let inside_fold = match (span_start, span_end) {
                (Some(start), Some(end)) if start < end => caret > start && caret < end,
                _ => false,
            };

            match state {
                0 => {
                    if folds_fixture {
                        continue;
                    }
                    let Some(start) = span_start else {
                        continue;
                    };
                    let Some(end) = span_end else {
                        continue;
                    };
                    if start >= end || !inside_fold {
                        continue;
                    }

                    fold_span_start = start;
                    fold_span_end = end;
                    baseline_rev = rev;
                    baseline_len = len;
                    baseline_caret = caret;
                    state = 1;
                }
                1 if folds_fixture => {
                    // The UI Gallery model toggle (`folds_fixture`) may be observed before the view
                    // updates have propagated to the decorated line text. Gate only once the
                    // placeholder is visible, which implies decorations are applied.
                    if !placeholder_present {
                        continue;
                    }

                    if rev != baseline_rev || len != baseline_len {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "folds_on",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                            },
                            "caret": caret,
                            "fold_span_line0": { "start": fold_span_start, "end": fold_span_end },
                        }));
                        state = 2;
                        break;
                    }

                    if caret == fold_span_start
                        && !(caret > fold_span_start && caret < fold_span_end)
                    {
                        clamp_observed = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "buffer_revision": rev,
                            "text_len_bytes": len,
                            "caret": caret,
                            "fold_span_line0": { "start": fold_span_start, "end": fold_span_end },
                        }));
                        state = 2;
                        break;
                    }

                    if caret > fold_span_start && caret < fold_span_end {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "folds_on",
                            "expected": {
                                "clamped_caret": fold_span_start,
                                "caret_not_inside_fold_span": true,
                            },
                            "observed": {
                                "caret": caret,
                                "caret_inside_fold_span": true,
                            },
                            "fold_span_line0": { "start": fold_span_start, "end": fold_span_end },
                        }));
                        state = 2;
                        break;
                    }
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": wrap_cols,
                "folds_fixture": folds_fixture,
                "preedit_active": preedit_active,
                "line0_placeholder_present": placeholder_present,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "caret": caret,
                "fold_span_line0": {
                    "start": span_start,
                    "end": span_end,
                },
                "state": state,
            }));
        }
        if state == 2 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "caret": baseline_caret,
            "fold_span_line0": { "start": fold_span_start, "end": fold_span_end },
        },
        "clamp_observed": clamp_observed,
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor folds clamp-selection gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 2 && clamp_observed.is_some() && violation.is_none() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor folds clamp-selection gate failed (expected: with folds_fixture=false, caret inside fold span; then when folds_fixture=true caret clamps to fold start without mutating buffer)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_toggle_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline snapshot (inlays A)
    // 1: waiting for inlays to toggle to B (inlays != A)
    // 2: waiting for inlays to return to A
    // 3: success
    let mut state: u8 = 0;

    let mut baseline_inlays: Option<bool> = None;
    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;
    let mut baseline_caret: u64 = 0;

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
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !enabled {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if wrap_cols.is_some() {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            match state {
                0 => {
                    baseline_inlays = Some(inlays_fixture);
                    baseline_rev = rev;
                    baseline_len = len;
                    baseline_caret = caret;
                    state = 1;
                }
                1 if Some(inlays_fixture) != baseline_inlays => {
                    toggled_inlays = Some(inlays_fixture);
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "toggled",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "inlays_fixture": { "baseline": baseline_inlays, "observed": inlays_fixture },
                        }));
                        state = 3;
                        break;
                    }
                    state = 2;
                }
                2 if Some(inlays_fixture) == baseline_inlays => {
                    if rev != baseline_rev || len != baseline_len || caret != baseline_caret {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "returned",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                                "caret": baseline_caret,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                                "caret": caret,
                            },
                            "inlays_fixture": { "baseline": baseline_inlays, "observed": inlays_fixture },
                        }));
                        state = 3;
                        break;
                    }
                    state = 3;
                    break;
                }
                _ => {}
            }

            last_observed = Some(serde_json::json!({
                "window": window_id,
                "tick_id": tick_id,
                "frame_id": frame_id,
                "soft_wrap_cols": wrap_cols,
                "inlays_fixture": inlays_fixture,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "caret": caret,
                "state": state,
            }));
        }
        if state == 3 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path =
        evidence_dir.join("check.ui_gallery_markdown_editor_source_inlays_toggle_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_inlays_toggle_stable",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "inlays_fixture": baseline_inlays,
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "caret": baseline_caret,
        },
        "toggled_inlays_fixture": toggled_inlays,
        "violation": violation,
        "last_observed": last_observed,
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays toggle gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if state == 3 && violation.is_none() && toggled_inlays.is_some() {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor inlays toggle gate failed (expected: inlays_fixture toggles and returns without changing buffer_revision/text_len_bytes/caret)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable_json(
        &bundle,
        bundle_path,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut examined_snapshots: u64 = 0;
    let mut ui_gallery_snapshots: u64 = 0;
    let mut last_observed: Option<serde_json::Value> = None;

    // State machine:
    // 0: waiting for baseline (inlays off, caret=2)
    // 1: waiting for inlays applied (fixture=true, line0_present=true, caret=2)
    // 2: waiting for caret to move right across the inlay (caret=3)
    // 3: waiting for caret to move left back to baseline (caret=2)
    // 4: success
    let mut state: u8 = 0;

    let mut baseline_rev: u64 = 0;
    let mut baseline_len: u64 = 0;

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
            if selected_page != "markdown_editor_source" {
                continue;
            }

            let enabled = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("interaction"))
                .and_then(|v| v.get("enabled"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !enabled {
                continue;
            }

            let wrap_cols = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("soft_wrap_cols"))
                .and_then(|v| v.as_u64());
            if wrap_cols.is_some() {
                continue;
            }

            let preedit_active = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("preedit_active"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if preedit_active {
                continue;
            }

            let folds_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("folds_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if folds_fixture {
                continue;
            }

            let inlays_fixture = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("inlays_fixture"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let inlay_present = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("inlays"))
                .and_then(|v| v.get("line0_present"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let rev = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("buffer_revision"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let len = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("text_len_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let anchor = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("anchor"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let caret = app_snapshot
                .and_then(|v| v.get("code_editor"))
                .and_then(|v| v.get("markdown_editor_source"))
                .and_then(|v| v.get("selection"))
                .and_then(|v| v.get("caret"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            let collapsed = anchor == caret;

            match state {
                0 => {
                    if inlays_fixture || inlay_present || !collapsed || caret != 2 || len != 5 {
                        // Keep scanning until we observe the baseline caret position with inlays off.
                    } else {
                        baseline_rev = rev;
                        baseline_len = len;
                        state = 1;
                    }
                }
                1 => {
                    if rev != baseline_rev || len != baseline_len {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "inlays_applied",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                            },
                        }));
                        state = 4;
                        break;
                    }

                    if inlays_fixture && inlay_present && collapsed && caret == 2 {
                        state = 2;
                    }
                }
                2 => {
                    if !(inlays_fixture && inlay_present) {
                        // Wait until the inlay is applied.
                    } else if rev != baseline_rev || len != baseline_len {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "move_right",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                            },
                        }));
                        state = 4;
                        break;
                    } else if collapsed && caret == 3 {
                        state = 3;
                    }
                }
                3 => {
                    if !(inlays_fixture && inlay_present) {
                        // Wait until the inlay is applied.
                    } else if rev != baseline_rev || len != baseline_len {
                        violation = Some(serde_json::json!({
                            "window": window_id,
                            "tick_id": tick_id,
                            "frame_id": frame_id,
                            "phase": "move_left",
                            "expected": {
                                "buffer_revision": baseline_rev,
                                "text_len_bytes": baseline_len,
                            },
                            "observed": {
                                "buffer_revision": rev,
                                "text_len_bytes": len,
                            },
                        }));
                        state = 4;
                        break;
                    } else if inlays_fixture && inlay_present && collapsed && caret == 2 {
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
                "soft_wrap_cols": wrap_cols,
                "folds_fixture": folds_fixture,
                "inlays_fixture": inlays_fixture,
                "line0_inlay_present": inlay_present,
                "buffer_revision": rev,
                "text_len_bytes": len,
                "selection": { "anchor": anchor, "caret": caret },
                "state": state,
            }));
        }
        if state == 4 {
            break;
        }
    }

    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir
        .join("check.ui_gallery_markdown_editor_source_inlays_caret_navigation_stable.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "ui_gallery_markdown_editor_source_inlays_caret_navigation_stable",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "ui_gallery_snapshots": ui_gallery_snapshots,
        "state": state,
        "baseline": {
            "buffer_revision": baseline_rev,
            "text_len_bytes": baseline_len,
            "expected_caret": 2,
        },
        "violation": violation,
        "last_observed": last_observed,
        "expected_sequence": [
            { "inlays_fixture": false, "line0_inlay_present": false, "caret": 2 },
            { "inlays_fixture": true, "line0_inlay_present": true, "caret": 2 },
            { "inlays_fixture": true, "line0_inlay_present": true, "caret": 3 },
            { "inlays_fixture": true, "line0_inlay_present": true, "caret": 2 }
        ],
    });
    write_json_value(&evidence_path, &payload)?;

    if ui_gallery_snapshots == 0 {
        return Err(format!(
            "ui-gallery markdown editor inlays caret-navigation gate requires app_snapshot.kind=fret_ui_gallery after warmup, but none were observed (warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots})\n  bundle: {}\n  evidence: {}",
            bundle_path.display(),
            evidence_path.display()
        ));
    }

    if let Some(violation) = violation {
        return Err(format!(
            "ui-gallery markdown editor inlays caret-navigation gate failed (buffer mutated)\n  bundle: {}\n  evidence: {}\n  violation: {}",
            bundle_path.display(),
            evidence_path.display(),
            violation
        ));
    }

    if state == 4 {
        return Ok(());
    }

    Err(format!(
        "ui-gallery markdown editor inlays caret-navigation gate failed (expected caret to move across the inlay without mutating buffer)\n  bundle: {}\n  evidence: {}",
        bundle_path.display(),
        evidence_path.display()
    ))
}
