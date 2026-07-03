use std::path::Path;

pub(crate) fn check_bundle_for_retained_identity_liveness_pressure(
    bundle_path: &Path,
    parent_pointer_would_repair_max: u64,
    gc_stale_liveness_offenders_max: u64,
    retained_subtree_membership_scan_nodes_max: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_identity_liveness_pressure_json(
        &bundle,
        bundle_path,
        parent_pointer_would_repair_max,
        gc_stale_liveness_offenders_max,
        retained_subtree_membership_scan_nodes_max,
        warmup_frames,
    )
}

pub(crate) fn check_bundle_for_retained_identity_liveness_pressure_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    parent_pointer_would_repair_max: u64,
    gc_stale_liveness_offenders_max: u64,
    retained_subtree_membership_scan_nodes_max: u64,
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
    let mut parent_pointer_would_repair_peak: u64 = 0;
    let mut gc_stale_liveness_offenders_peak: u64 = 0;
    let mut retained_subtree_membership_scan_nodes_peak: u64 = 0;
    let mut failures: Vec<String> = Vec::new();

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

            let stats = s.get("debug").and_then(|v| v.get("stats"));
            let parent_pointer_would_repair_nodes = stats
                .and_then(|v| v.get("parent_pointer_would_repair_nodes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let gc_stale_liveness_offenders = stats
                .and_then(|v| v.get("gc_stale_liveness_offenders"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let retained_subtree_membership_scan_nodes = stats
                .and_then(|v| v.get("retained_subtree_membership_scan_nodes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            parent_pointer_would_repair_peak =
                parent_pointer_would_repair_peak.max(parent_pointer_would_repair_nodes);
            gc_stale_liveness_offenders_peak =
                gc_stale_liveness_offenders_peak.max(gc_stale_liveness_offenders);
            retained_subtree_membership_scan_nodes_peak =
                retained_subtree_membership_scan_nodes_peak
                    .max(retained_subtree_membership_scan_nodes);

            if parent_pointer_would_repair_nodes > parent_pointer_would_repair_max {
                failures.push(format!(
                    "window={window_id} frame_id={frame_id} parent_pointer_would_repair_nodes={parent_pointer_would_repair_nodes} max={parent_pointer_would_repair_max}"
                ));
            }
            if gc_stale_liveness_offenders > gc_stale_liveness_offenders_max {
                failures.push(format!(
                    "window={window_id} frame_id={frame_id} gc_stale_liveness_offenders={gc_stale_liveness_offenders} max={gc_stale_liveness_offenders_max}"
                ));
            }
            if retained_subtree_membership_scan_nodes > retained_subtree_membership_scan_nodes_max {
                failures.push(format!(
                    "window={window_id} frame_id={frame_id} retained_subtree_membership_scan_nodes={retained_subtree_membership_scan_nodes} max={retained_subtree_membership_scan_nodes_max}"
                ));
            }
        }
    }

    if failures.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("retained identity/liveness pressure gate failed\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "warmup_frames={warmup_frames} examined_snapshots={examined_snapshots} parent_pointer_would_repair_peak={parent_pointer_would_repair_peak} gc_stale_liveness_offenders_peak={gc_stale_liveness_offenders_peak} retained_subtree_membership_scan_nodes_peak={retained_subtree_membership_scan_nodes_peak}\n"
    ));
    for line in failures.into_iter().take(10) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}
