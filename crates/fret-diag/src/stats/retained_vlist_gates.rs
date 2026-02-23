use std::path::Path;

pub(super) fn check_bundle_for_retained_vlist_reconcile_no_notify_min(
    bundle_path: &Path,
    min_reconcile_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
        &bundle,
        bundle_path,
        min_reconcile_events,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_retained_vlist_reconcile_no_notify_min_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    min_reconcile_events: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reconcile_events: u64 = 0;
    let mut reconcile_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut notify_offenders: Vec<String> = Vec::new();

    for w in windows {
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

            let list_count = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let stats_count = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let count = list_count.max(stats_count);
            if count == 0 {
                continue;
            }

            reconcile_frames = reconcile_frames.saturating_add(1);
            reconcile_events = reconcile_events.saturating_add(count);

            let dirty_views = s
                .get("debug")
                .and_then(|v| v.get("dirty_views"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);

            for dv in dirty_views {
                let source = dv
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let detail = dv
                    .get("detail")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if source == "notify" || detail.contains("notify") {
                    let root_node = dv.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0);
                    notify_offenders.push(format!(
                        "frame_id={frame_id} dirty_view_root_node={root_node} source={source} detail={detail}"
                    ));
                    break;
                }
            }
        }
    }

    if !notify_offenders.is_empty() {
        let mut msg = String::new();
        msg.push_str(
            "retained virtual-list reconcile should not require notify-based dirty views\n",
        );
        msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
        msg.push_str(&format!(
            "min_reconcile_events={min_reconcile_events} reconcile_events={reconcile_events} reconcile_frames={reconcile_frames} warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
        ));
        for line in notify_offenders.into_iter().take(10) {
            msg.push_str("  ");
            msg.push_str(&line);
            msg.push('\n');
        }
        return Err(msg);
    }

    if reconcile_events < min_reconcile_events {
        return Err(format!(
            "expected at least {min_reconcile_events} retained virtual-list reconcile events, got {reconcile_events} \
(reconcile_frames={reconcile_frames}, warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) \
bundle: {}",
            bundle_path.display()
        ));
    }

    Ok(())
}

pub(super) fn check_bundle_for_retained_vlist_attach_detach_max(
    bundle_path: &Path,
    max_delta: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let bytes = std::fs::read(bundle_path).map_err(|e| e.to_string())?;
    let bundle: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    check_bundle_for_retained_vlist_attach_detach_max_json(
        &bundle,
        bundle_path,
        max_delta,
        warmup_frames,
    )
}

pub(super) fn check_bundle_for_retained_vlist_attach_detach_max_json(
    bundle: &serde_json::Value,
    bundle_path: &Path,
    max_delta: u64,
    warmup_frames: u64,
) -> Result<(), String> {
    let windows = bundle
        .get("windows")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "invalid bundle.json: missing windows".to_string())?;
    if windows.is_empty() {
        return Ok(());
    }

    let mut reconcile_events: u64 = 0;
    let mut reconcile_frames: u64 = 0;
    let mut examined_snapshots: u64 = 0;
    let mut offenders: Vec<String> = Vec::new();

    for w in windows {
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

            let list_count = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map(|v| v.len() as u64)
                .unwrap_or(0);
            let stats_count = s
                .get("debug")
                .and_then(|v| v.get("stats"))
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let count = list_count.max(stats_count);
            if count == 0 {
                continue;
            }

            reconcile_frames = reconcile_frames.saturating_add(1);
            reconcile_events = reconcile_events.saturating_add(count);

            let records = s
                .get("debug")
                .and_then(|v| v.get("retained_virtual_list_reconciles"))
                .and_then(|v| v.as_array())
                .map_or(&[][..], |v| v);
            let (attached, detached) = if records.is_empty() {
                let stats = s
                    .get("debug")
                    .and_then(|v| v.get("stats"))
                    .and_then(|v| v.as_object());
                let attached = stats
                    .and_then(|v| v.get("retained_virtual_list_attached_items"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let detached = stats
                    .and_then(|v| v.get("retained_virtual_list_detached_items"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                (attached, detached)
            } else {
                let attached = records
                    .iter()
                    .map(|r| {
                        r.get("attached_items")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                    })
                    .sum::<u64>();
                let detached = records
                    .iter()
                    .map(|r| {
                        r.get("detached_items")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                    })
                    .sum::<u64>();
                (attached, detached)
            };

            let delta = attached.saturating_add(detached);
            if delta > max_delta {
                offenders.push(format!(
                    "frame_id={frame_id} attached={attached} detached={detached} delta={delta} max={max_delta}"
                ));
            }
        }
    }

    if reconcile_events == 0 {
        return Err(format!(
            "expected at least 1 retained virtual-list reconcile event (required for attach/detach max check), got 0 \
(warmup_frames={warmup_frames}, examined_snapshots={examined_snapshots}) bundle: {}",
            bundle_path.display()
        ));
    }

    if offenders.is_empty() {
        return Ok(());
    }

    let mut msg = String::new();
    msg.push_str("retained virtual-list attach/detach delta exceeded the configured maximum\n");
    msg.push_str(&format!("bundle: {}\n", bundle_path.display()));
    msg.push_str(&format!(
        "max_delta={max_delta} reconcile_events={reconcile_events} reconcile_frames={reconcile_frames} warmup_frames={warmup_frames} examined_snapshots={examined_snapshots}\n"
    ));
    for line in offenders.into_iter().take(10) {
        msg.push_str("  ");
        msg.push_str(&line);
        msg.push('\n');
    }
    Err(msg)
}
