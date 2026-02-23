use std::path::Path;

use crate::util::{now_unix_ms, write_json_value};

pub(super) fn check_bundle_for_gc_sweep_liveness(
    bundle_path: &Path,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;

    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut offenders: Vec<String> = Vec::new();
    let mut offender_samples: Vec<serde_json::Value> = Vec::new();
    let mut offender_taxonomy_counts: std::collections::BTreeMap<String, u64> =
        std::collections::BTreeMap::new();
    let mut examined_snapshots: u64 = 0;
    let mut removed_subtrees_total: u64 = 0;
    let mut removed_subtrees_offenders: u64 = 0;

    let mut element_runtime_node_entry_root_overwrites_total: u64 = 0;
    let mut element_runtime_view_cache_reuse_root_element_samples_total: u64 = 0;
    let mut element_runtime_retained_keep_alive_roots_total: u64 = 0;

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

            let mut snapshot_node_entry_root_overwrites_len: u64 = 0;
            let mut snapshot_view_cache_reuse_root_element_samples_len: u64 = 0;
            let mut snapshot_retained_keep_alive_roots_len: u64 = 0;

            let element_runtime = s
                .get("debug")
                .and_then(|v| v.get("element_runtime"))
                .and_then(|v| v.as_object());
            if let Some(element_runtime) = element_runtime {
                snapshot_node_entry_root_overwrites_len = element_runtime
                    .get("node_entry_root_overwrites")
                    .and_then(|v| v.as_array())
                    .map(|v| v.len() as u64)
                    .unwrap_or(0);
                snapshot_view_cache_reuse_root_element_samples_len = element_runtime
                    .get("view_cache_reuse_root_element_samples")
                    .and_then(|v| v.as_array())
                    .map(|v| v.len() as u64)
                    .unwrap_or(0);
                snapshot_retained_keep_alive_roots_len = element_runtime
                    .get("retained_keep_alive_roots")
                    .and_then(|v| v.as_array())
                    .map(|v| v.len() as u64)
                    .unwrap_or(0);

                element_runtime_node_entry_root_overwrites_total =
                    element_runtime_node_entry_root_overwrites_total
                        .saturating_add(snapshot_node_entry_root_overwrites_len);
                element_runtime_view_cache_reuse_root_element_samples_total =
                    element_runtime_view_cache_reuse_root_element_samples_total
                        .saturating_add(snapshot_view_cache_reuse_root_element_samples_len);
                element_runtime_retained_keep_alive_roots_total =
                    element_runtime_retained_keep_alive_roots_total
                        .saturating_add(snapshot_retained_keep_alive_roots_len);
            }

            let Some(removed) = s
                .get("debug")
                .and_then(|v| v.get("removed_subtrees"))
                .and_then(|v| v.as_array())
            else {
                continue;
            };

            for r in removed {
                removed_subtrees_total = removed_subtrees_total.saturating_add(1);
                let unreachable = r
                    .get("unreachable_from_liveness_roots")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let reachable_from_layer_roots = r
                    .get("reachable_from_layer_roots")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let reachable_from_view_cache_roots = r
                    .get("reachable_from_view_cache_roots")
                    .and_then(|v| v.as_bool());
                let root_layer_visible = r.get("root_layer_visible").and_then(|v| v.as_bool());
                let reuse_roots_len = r
                    .get("view_cache_reuse_roots_len")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let under_reuse = reuse_roots_len > 0;
                let reuse_root_nodes_len = r
                    .get("view_cache_reuse_root_nodes_len")
                    .and_then(|v| v.as_u64());
                let trigger_in_keep_alive = r
                    .get("trigger_element_in_view_cache_keep_alive")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let trigger_listed_under_reuse_root = r
                    .get("trigger_element_listed_under_reuse_root")
                    .and_then(|v| v.as_u64())
                    .is_some();

                let taxonomy_flags: Vec<&'static str> = {
                    let mut flags: Vec<&'static str> = Vec::new();
                    if snapshot_node_entry_root_overwrites_len > 0 {
                        flags.push("ownership_drift_suspected");
                    }
                    if under_reuse && snapshot_view_cache_reuse_root_element_samples_len == 0 {
                        flags.push("missing_reuse_root_membership_samples");
                    }
                    if trigger_in_keep_alive {
                        flags.push("trigger_in_keep_alive");
                    }
                    if under_reuse && trigger_listed_under_reuse_root {
                        flags.push("trigger_listed_under_reuse_root");
                    }
                    if under_reuse && reachable_from_view_cache_roots.is_none() {
                        flags.push("missing_view_cache_reachability_evidence");
                    }
                    if under_reuse && reuse_root_nodes_len == Some(0) {
                        flags.push("reuse_roots_unmapped");
                    }
                    flags
                };

                let taxonomy = if !unreachable
                    || reachable_from_layer_roots
                    || reachable_from_view_cache_roots == Some(true)
                    || root_layer_visible == Some(true)
                {
                    Some("swept_while_reachable")
                } else if under_reuse && reachable_from_view_cache_roots.is_none() {
                    // Under reuse we expect reachability from reuse roots to be recorded; otherwise
                    // the cache-005 harness won't be actionable from a single bundle.
                    Some("missing_view_cache_reachability_evidence")
                } else if under_reuse && reuse_root_nodes_len == Some(0) {
                    // If we know reuse roots exist but cannot map any to nodes, the window's
                    // identity bookkeeping is inconsistent, so "reachable from reuse roots" is
                    // meaningless for that frame.
                    Some("reuse_roots_unmapped")
                } else if trigger_in_keep_alive {
                    // Keep-alive roots are part of the liveness root set. If the record still
                    // indicates a trigger element is in a keep-alive bucket while being swept as
                    // unreachable, that's almost certainly bookkeeping drift.
                    Some("keep_alive_liveness_mismatch")
                } else if under_reuse && snapshot_view_cache_reuse_root_element_samples_len == 0 {
                    // When reuse roots exist, we expect membership list samples to be exported so
                    // cache-005 failures remain actionable from a single bundle.
                    Some("missing_reuse_root_membership_samples")
                } else if under_reuse && trigger_listed_under_reuse_root {
                    // If the trigger element is recorded as being listed under some reuse root,
                    // but the removal happens as unreachable from reuse roots, the membership/touch
                    // logic is likely stale or incomplete.
                    Some("reuse_membership_mismatch")
                } else {
                    None
                };

                if let Some(taxonomy) = taxonomy {
                    removed_subtrees_offenders = removed_subtrees_offenders.saturating_add(1);
                    *offender_taxonomy_counts
                        .entry(taxonomy.to_string())
                        .or_insert(0) += 1;
                    let root = r.get("root").and_then(|v| v.as_u64()).unwrap_or(0);
                    let root_element_path = r
                        .get("root_element_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>");
                    let trigger_path = r
                        .get("trigger_element_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<none>");
                    let mut violations: Vec<&'static str> = Vec::new();
                    if taxonomy == "swept_while_reachable" && !unreachable {
                        violations.push("reachable_from_liveness_roots");
                    }
                    if taxonomy == "swept_while_reachable" && reachable_from_layer_roots {
                        violations.push("reachable_from_layer_roots");
                    }
                    if taxonomy == "swept_while_reachable"
                        && reachable_from_view_cache_roots == Some(true)
                    {
                        violations.push("reachable_from_view_cache_roots");
                    }
                    if taxonomy == "swept_while_reachable" && root_layer_visible == Some(true) {
                        violations.push("root_layer_visible");
                    }
                    offenders.push(format!(
                        "window={window_id} frame_id={frame_id} taxonomy={taxonomy} root={root} unreachable_from_liveness_roots={unreachable} reachable_from_layer_roots={reachable_from_layer_roots} reachable_from_view_cache_roots={reachable_from_view_cache_roots:?} root_layer_visible={root_layer_visible:?} reuse_roots_len={reuse_roots_len} reuse_root_nodes_len={reuse_root_nodes_len:?} trigger_in_keep_alive={trigger_in_keep_alive} trigger_listed_under_reuse_root={trigger_listed_under_reuse_root} root_element_path={root_element_path} trigger_element_path={trigger_path}"
                    ));

                    const MAX_SAMPLES: usize = 128;
                    if offender_samples.len() < MAX_SAMPLES {
                        offender_samples.push(serde_json::json!({
                            "window": window_id,
                            "frame_id": frame_id,
                            "tick_id": s.get("tick_id").and_then(|v| v.as_u64()).unwrap_or(0),
                            "taxonomy": taxonomy,
                            "taxonomy_flags": taxonomy_flags,
                            "root": r.get("root").and_then(|v| v.as_u64()).unwrap_or(0),
                            "root_root": r.get("root_root").and_then(|v| v.as_u64()),
                            "root_layer": r.get("root_layer").and_then(|v| v.as_u64()),
                            "root_layer_visible": root_layer_visible,
                            "reachable_from_layer_roots": reachable_from_layer_roots,
                            "reachable_from_view_cache_roots": reachable_from_view_cache_roots,
                            "unreachable_from_liveness_roots": unreachable,
                            "violations": violations,
                            "reuse_roots_len": reuse_roots_len,
                            "reuse_root_nodes_len": reuse_root_nodes_len,
                            "trigger_in_keep_alive": trigger_in_keep_alive,
                            "trigger_listed_under_reuse_root": trigger_listed_under_reuse_root,
                            "liveness_layer_roots_len": r.get("liveness_layer_roots_len").and_then(|v| v.as_u64()),
                            "view_cache_reuse_roots_len": r.get("view_cache_reuse_roots_len").and_then(|v| v.as_u64()),
                            "view_cache_reuse_root_nodes_len": r.get("view_cache_reuse_root_nodes_len").and_then(|v| v.as_u64()),
                            "snapshot_node_entry_root_overwrites_len": snapshot_node_entry_root_overwrites_len,
                            "snapshot_view_cache_reuse_root_element_samples_len": snapshot_view_cache_reuse_root_element_samples_len,
                            "snapshot_retained_keep_alive_roots_len": snapshot_retained_keep_alive_roots_len,
                            "root_element": r.get("root_element").and_then(|v| v.as_u64()),
                            "root_element_path": r.get("root_element_path").and_then(|v| v.as_str()),
                            "trigger_element": r.get("trigger_element").and_then(|v| v.as_u64()),
                            "trigger_element_path": r.get("trigger_element_path").and_then(|v| v.as_str()),
                            "trigger_element_in_view_cache_keep_alive": r.get("trigger_element_in_view_cache_keep_alive").and_then(|v| v.as_bool()),
                            "trigger_element_listed_under_reuse_root": r.get("trigger_element_listed_under_reuse_root").and_then(|v| v.as_u64()),
                            "root_root_parent_sever_parent": r.get("root_root_parent_sever_parent").and_then(|v| v.as_u64()),
                            "root_root_parent_sever_location": r.get("root_root_parent_sever_location").and_then(|v| v.as_str()),
                            "root_root_parent_sever_frame_id": r.get("root_root_parent_sever_frame_id").and_then(|v| v.as_u64()),
                        }));
                    }
                }
            }
        }
    }

    // Always write evidence so debugging doesn't require re-running the harness.
    let evidence_dir = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let evidence_path = evidence_dir.join("check.gc_sweep_liveness.json");
    let payload = serde_json::json!({
        "schema_version": 1,
        "generated_unix_ms": now_unix_ms(),
        "kind": "gc_sweep_liveness",
        "bundle_json": bundle_path.display().to_string(),
        "evidence_dir": evidence_dir.display().to_string(),
        "evidence_path": evidence_path.display().to_string(),
        "warmup_frames": warmup_frames,
        "examined_snapshots": examined_snapshots,
        "removed_subtrees_total": removed_subtrees_total,
        "removed_subtrees_offenders": removed_subtrees_offenders,
        "offender_taxonomy_counts": offender_taxonomy_counts,
        "offender_samples": offender_samples,
        "debug_summary": {
            "element_runtime_node_entry_root_overwrites_total": element_runtime_node_entry_root_overwrites_total,
            "element_runtime_view_cache_reuse_root_element_samples_total": element_runtime_view_cache_reuse_root_element_samples_total,
            "element_runtime_retained_keep_alive_roots_total": element_runtime_retained_keep_alive_roots_total,
        },
    });
    write_json_value(&evidence_path, &payload)?;

    if offenders.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("GC sweep liveness violation: removed_subtrees contains entries that appear live or inconsistent with keep-alive/reuse bookkeeping\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
    ));
    msg.push_str(&format!("evidence: {}\n", evidence_path.display()));
    for line in offenders.into_iter().take(10) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}
