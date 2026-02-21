use std::path::Path;

use crate::util::{now_unix_ms, write_json_value};

use super::{semantics_node_id_for_test_id, semantics_parent_map};

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits(
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

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits_json(
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

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits(
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

pub(super) fn check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits_json(
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
