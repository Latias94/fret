use std::path::{Path, PathBuf};

use serde_json::Value;

use super::resolve;

pub(crate) fn cmd_dock_graph(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let Some(src) = rest.first().cloned() else {
        return Err(
            "missing bundle artifact path (try: fretboard-dev diag dock-graph <bundle_dir|bundle.json|bundle.schema2.json>)"
                .to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }

    let src = crate::resolve_path(workspace_root, PathBuf::from(src));
    let resolved = resolve::resolve_bundle_ref(&src)?;
    let bundle_path = resolved.bundle_artifact;

    let bytes = std::fs::read(&bundle_path).map_err(|e| {
        format!(
            "failed to read bundle artifact (bundle.json or bundle.schema2.json): {}\n  {}",
            bundle_path.display(),
            e
        )
    })?;
    let bundle: Value = serde_json::from_slice(&bytes).map_err(|e| {
        format!(
            "failed to parse bundle JSON (bundle.json or bundle.schema2.json): {}\n  {}",
            bundle_path.display(),
            e
        )
    })?;

    let report = extract_dock_graph_report(&bundle, &bundle_path)?;
    if stats_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    print_dock_graph_report(&report);
    Ok(())
}

fn extract_dock_graph_report(bundle: &Value, bundle_path: &Path) -> Result<Value, String> {
    let Some(windows) = bundle.get("windows").and_then(|v| v.as_array()) else {
        return Err(format!(
            "bundle is missing `windows[]` (unexpected bundle schema)\n  bundle: {}",
            bundle_path.display()
        ));
    };

    let mut out_windows: Vec<Value> = Vec::new();

    for w in windows {
        let window_id = w.get("window").and_then(|v| v.as_u64());
        let Some(snapshots) = w.get("snapshots").and_then(|v| v.as_array()) else {
            continue;
        };
        let Some(last) = snapshots.last() else {
            continue;
        };

        let frame_id = last.get("frame_id").and_then(|v| v.as_u64());
        let tick_id = last.get("tick_id").and_then(|v| v.as_u64());
        let window_snapshot_seq = last.get("window_snapshot_seq").and_then(|v| v.as_u64());
        let docking = last.get("debug").and_then(|v| v.get("docking_interaction"));
        let signature = docking
            .and_then(|v| v.get("dock_graph_signature"))
            .and_then(|v| v.get("signature"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let fingerprint64 = docking
            .and_then(|v| v.get("dock_graph_signature"))
            .and_then(|v| v.get("fingerprint64"))
            .and_then(|v| v.as_u64());

        let stats = docking.and_then(|v| v.get("dock_graph_stats"));
        let canonical_ok = stats
            .and_then(|v| v.get("canonical_ok"))
            .and_then(|v| v.as_bool());
        let node_count = stats
            .and_then(|v| v.get("node_count"))
            .and_then(|v| v.as_u64());
        let tabs_count = stats
            .and_then(|v| v.get("tabs_count"))
            .and_then(|v| v.as_u64());
        let split_count = stats
            .and_then(|v| v.get("split_count"))
            .and_then(|v| v.as_u64());
        let floating_count = stats
            .and_then(|v| v.get("floating_count"))
            .and_then(|v| v.as_u64());
        let max_depth = stats
            .and_then(|v| v.get("max_depth"))
            .and_then(|v| v.as_u64());

        out_windows.push(serde_json::json!({
            "window": window_id,
            "frame_id": frame_id,
            "tick_id": tick_id,
            "window_snapshot_seq": window_snapshot_seq,
            "dock_graph": {
                "signature": signature,
                "fingerprint64": fingerprint64,
                "canonical_ok": canonical_ok,
                "node_count": node_count,
                "tabs_count": tabs_count,
                "split_count": split_count,
                "floating_count": floating_count,
                "max_depth": max_depth,
            }
        }));
    }

    Ok(serde_json::json!({
        "schema_version": 1,
        "kind": "dock_graph",
        "bundle": bundle_path.display().to_string(),
        "windows_total": out_windows.len(),
        "windows": out_windows,
    }))
}

fn print_dock_graph_report(report: &Value) {
    fn str_field<'a>(v: &'a Value, key: &str) -> &'a str {
        v.get(key).and_then(|v| v.as_str()).unwrap_or("")
    }
    fn u64_field(v: &Value, key: &str) -> Option<u64> {
        v.get(key).and_then(|v| v.as_u64())
    }

    println!("dock_graph:");
    println!("  bundle: {}", str_field(report, "bundle"));
    println!(
        "  windows_total: {}",
        u64_field(report, "windows_total").unwrap_or(0)
    );

    let Some(windows) = report.get("windows").and_then(|v| v.as_array()) else {
        return;
    };

    for w in windows {
        let win = w.get("window").and_then(|v| v.as_u64()).unwrap_or(0);
        let frame_id = w.get("frame_id").and_then(|v| v.as_u64()).unwrap_or(0);
        let graph = w.get("dock_graph").unwrap_or(&Value::Null);
        let fingerprint64 = graph
            .get("fingerprint64")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let canonical_ok = graph
            .get("canonical_ok")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        println!(
            "  - window={} frame={} fingerprint64={} canonical_ok={}",
            win, frame_id, fingerprint64, canonical_ok
        );
        if let Some(sig) = graph.get("signature").and_then(|v| v.as_str()) {
            println!("    signature: {}", sig);
        }
        let node_count = graph
            .get("node_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let tabs_count = graph
            .get("tabs_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let split_count = graph
            .get("split_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let floating_count = graph
            .get("floating_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let max_depth = graph.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(0);
        println!(
            "    stats: nodes={} tabs={} splits={} floatings={} max_depth={}",
            node_count, tabs_count, split_count, floating_count, max_depth
        );
    }
}
