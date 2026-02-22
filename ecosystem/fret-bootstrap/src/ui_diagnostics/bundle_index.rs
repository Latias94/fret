use std::collections::{HashMap, HashSet};

use serde_json::{Value, json};

use super::{UiDiagnosticsSnapshotV1, UiDiagnosticsWindowBundleV1, UiSemanticsSnapshotV1, bundle};

const DEFAULT_WARMUP_FRAMES: u64 = 0;

fn semantics_table_map<'a>(
    table: Option<&'a bundle::UiBundleSemanticsTableV1>,
) -> HashMap<(u64, u64), &'a UiSemanticsSnapshotV1> {
    let mut out: HashMap<(u64, u64), &'a UiSemanticsSnapshotV1> = HashMap::new();
    let Some(table) = table else {
        return out;
    };
    for e in &table.entries {
        out.entry((e.window, e.semantics_fingerprint))
            .or_insert(&e.semantics);
    }
    out
}

fn resolve_semantics_snapshot<'a>(
    s: &'a UiDiagnosticsSnapshotV1,
    semantics_table: &'a HashMap<(u64, u64), &'a UiSemanticsSnapshotV1>,
) -> Option<&'a UiSemanticsSnapshotV1> {
    if let Some(inline) = s.debug.semantics.as_ref() {
        return Some(inline);
    }
    let fp = s.semantics_fingerprint?;
    semantics_table.get(&(s.window, fp)).copied()
}

fn snapshot_semantics_source<'a>(
    s: &'a UiDiagnosticsSnapshotV1,
    semantics_table: &'a HashMap<(u64, u64), &'a UiSemanticsSnapshotV1>,
) -> &'static str {
    if s.debug.semantics.is_some() {
        "inline"
    } else if resolve_semantics_snapshot(s, semantics_table).is_some() {
        "table"
    } else {
        "none"
    }
}

fn pick_last_snapshot_with_resolved_semantics<'a>(
    snaps: &'a [UiDiagnosticsSnapshotV1],
    warmup_frames: u64,
    semantics_table: &'a HashMap<(u64, u64), &'a UiSemanticsSnapshotV1>,
) -> Option<&'a UiDiagnosticsSnapshotV1> {
    for (idx, s) in snaps.iter().enumerate().rev() {
        if (idx as u64) < warmup_frames {
            break;
        }
        if resolve_semantics_snapshot(s, semantics_table).is_some() {
            return Some(s);
        }
    }
    None
}

pub(super) fn build_bundle_index_json(
    bundle_label: &str,
    windows: &[UiDiagnosticsWindowBundleV1],
    semantics_table: Option<&bundle::UiBundleSemanticsTableV1>,
) -> Value {
    let warmup_frames = DEFAULT_WARMUP_FRAMES;
    let semantics_table = semantics_table_map(semantics_table);

    let mut total_snapshots: u64 = 0;
    let mut windows_out: Vec<Value> = Vec::with_capacity(windows.len());

    for w in windows {
        total_snapshots = total_snapshots.saturating_add(w.snapshots.len() as u64);

        let mut snapshots_out: Vec<Value> = Vec::with_capacity(w.snapshots.len());

        let mut first_frame_id: Option<u64> = None;
        let mut last_frame_id: Option<u64> = None;
        let mut first_timestamp_unix_ms: Option<u64> = None;
        let mut last_timestamp_unix_ms: Option<u64> = None;

        for (idx, s) in w.snapshots.iter().enumerate() {
            let frame_id = s.frame_id;
            let window_snapshot_seq = s.window_snapshot_seq;
            let ts = Some(s.timestamp_unix_ms);

            first_frame_id.get_or_insert(frame_id);
            last_frame_id = Some(frame_id);
            if first_timestamp_unix_ms.is_none() {
                first_timestamp_unix_ms = ts;
            }
            last_timestamp_unix_ms = ts.or(last_timestamp_unix_ms);

            let semantics_source = snapshot_semantics_source(s, &semantics_table);
            let has_semantics = semantics_source == "inline" || semantics_source == "table";

            snapshots_out.push(json!({
                "window_snapshot_seq": window_snapshot_seq,
                "frame_id": frame_id,
                "timestamp_unix_ms": ts,
                "is_warmup": (idx as u64) < warmup_frames,
                "semantics_fingerprint": s.semantics_fingerprint,
                "semantics_source": semantics_source,
                "has_semantics": has_semantics,
            }));
        }

        windows_out.push(json!({
            "window": w.window,
            "snapshots_total": w.snapshots.len(),
            "first_frame_id": first_frame_id,
            "last_frame_id": last_frame_id,
            "first_timestamp_unix_ms": first_timestamp_unix_ms,
            "last_timestamp_unix_ms": last_timestamp_unix_ms,
            "snapshots": snapshots_out,
        }));
    }

    json!({
        "schema_version": 1,
        "kind": "bundle_index",
        "bundle": bundle_label,
        "warmup_frames": warmup_frames,
        "windows_total": windows.len(),
        "snapshots_total": total_snapshots,
        "windows": windows_out,
    })
}

pub(super) fn build_bundle_meta_json(
    bundle_label: &str,
    windows: &[UiDiagnosticsWindowBundleV1],
    semantics_table: Option<&bundle::UiBundleSemanticsTableV1>,
) -> Value {
    let warmup_frames = DEFAULT_WARMUP_FRAMES;
    let semantics_table = semantics_table_map(semantics_table);

    let mut total_snapshots: u64 = 0;
    let mut total_snapshots_with_semantics: u64 = 0;
    let mut total_snapshots_with_inline_semantics: u64 = 0;
    let mut total_snapshots_with_table_semantics: u64 = 0;
    let mut total_unique_semantics_fingerprints: HashSet<u64> = HashSet::new();
    let mut total_unique_test_ids: HashSet<String> = HashSet::new();

    let mut windows_out: Vec<Value> = Vec::with_capacity(windows.len());

    let mut table_entries_total: u64 = 0;
    let mut table_unique_keys: HashSet<(u64, u64)> = HashSet::new();
    for ((window, fp), _sem) in semantics_table.iter() {
        table_entries_total = table_entries_total.saturating_add(1);
        table_unique_keys.insert((*window, *fp));
    }

    for w in windows {
        total_snapshots = total_snapshots.saturating_add(w.snapshots.len() as u64);

        let mut window_snapshots_with_semantics: u64 = 0;
        let mut window_snapshots_with_inline_semantics: u64 = 0;
        let mut window_snapshots_with_table_semantics: u64 = 0;
        let mut window_unique_semantics_fingerprints: HashSet<u64> = HashSet::new();

        for s in &w.snapshots {
            let has_inline = s.debug.semantics.is_some();
            if has_inline {
                window_snapshots_with_inline_semantics =
                    window_snapshots_with_inline_semantics.saturating_add(1);
                total_snapshots_with_inline_semantics =
                    total_snapshots_with_inline_semantics.saturating_add(1);
            }

            let has_resolved = resolve_semantics_snapshot(s, &semantics_table).is_some();
            if has_resolved {
                window_snapshots_with_semantics = window_snapshots_with_semantics.saturating_add(1);
                total_snapshots_with_semantics = total_snapshots_with_semantics.saturating_add(1);
                if !has_inline {
                    window_snapshots_with_table_semantics =
                        window_snapshots_with_table_semantics.saturating_add(1);
                    total_snapshots_with_table_semantics =
                        total_snapshots_with_table_semantics.saturating_add(1);
                }
            }

            if let Some(fp) = s.semantics_fingerprint {
                window_unique_semantics_fingerprints.insert(fp);
                total_unique_semantics_fingerprints.insert(fp);
            }
        }

        let window_table_entries_total: u64 = semantics_table
            .iter()
            .filter(|((window, _fp), _)| *window == w.window)
            .count() as u64;
        let window_table_unique_keys_total: u64 = semantics_table
            .iter()
            .filter(|((window, _fp), _)| *window == w.window)
            .map(|((_window, fp), _)| *fp)
            .collect::<HashSet<u64>>()
            .len() as u64;

        let considered = pick_last_snapshot_with_resolved_semantics(
            &w.snapshots,
            warmup_frames,
            &semantics_table,
        );

        if let Some(snapshot) = considered {
            let nodes = resolve_semantics_snapshot(snapshot, &semantics_table)
                .map(|s| s.nodes.as_slice())
                .unwrap_or(&[]);

            let mut counts: HashMap<&str, u64> = HashMap::new();
            for n in nodes {
                let Some(raw) = n.test_id.as_deref() else {
                    continue;
                };
                let test_id = raw.trim();
                if test_id.is_empty() {
                    continue;
                }
                *counts.entry(test_id).or_insert(0) += 1;
            }
            for k in counts.keys() {
                total_unique_test_ids.insert((*k).to_string());
            }

            let semantics_nodes_total = nodes.len() as u64;
            let test_id_nodes_total: u64 = counts.values().sum();
            let unique_test_ids_total = counts.len() as u64;
            let duplicate_test_ids_total = counts.values().filter(|v| **v > 1).count() as u64;

            windows_out.push(json!({
                "window": w.window,
                "snapshots_total": w.snapshots.len(),
                "snapshots_with_semantics_total": window_snapshots_with_semantics,
                "snapshots_with_inline_semantics_total": window_snapshots_with_inline_semantics,
                "snapshots_with_table_semantics_total": window_snapshots_with_table_semantics,
                "unique_semantics_fingerprints_total": window_unique_semantics_fingerprints.len(),
                "semantics_table_entries_total": window_table_entries_total,
                "semantics_table_unique_keys_total": window_table_unique_keys_total,
                "considered_frame_id": snapshot.frame_id,
                "considered_timestamp_unix_ms": snapshot.timestamp_unix_ms,
                "semantics_nodes_total": semantics_nodes_total,
                "test_id_nodes_total": test_id_nodes_total,
                "unique_test_ids_total": unique_test_ids_total,
                "duplicate_test_ids_total": duplicate_test_ids_total,
            }));
        } else {
            windows_out.push(json!({
                "window": w.window,
                "snapshots_total": w.snapshots.len(),
                "snapshots_with_semantics_total": window_snapshots_with_semantics,
                "snapshots_with_inline_semantics_total": window_snapshots_with_inline_semantics,
                "snapshots_with_table_semantics_total": window_snapshots_with_table_semantics,
                "unique_semantics_fingerprints_total": window_unique_semantics_fingerprints.len(),
                "semantics_table_entries_total": window_table_entries_total,
                "semantics_table_unique_keys_total": window_table_unique_keys_total,
                "considered_frame_id": Value::Null,
                "considered_timestamp_unix_ms": Value::Null,
                "semantics_nodes_total": 0,
                "test_id_nodes_total": 0,
                "unique_test_ids_total": 0,
                "duplicate_test_ids_total": 0,
            }));
        }
    }

    json!({
        "schema_version": 1,
        "kind": "bundle_meta",
        "bundle": bundle_label,
        "warmup_frames": warmup_frames,
        "windows_total": windows.len(),
        "snapshots_total": total_snapshots,
        "total_unique_test_ids": total_unique_test_ids.len(),
        "snapshots_with_semantics_total": total_snapshots_with_semantics,
        "snapshots_with_inline_semantics_total": total_snapshots_with_inline_semantics,
        "snapshots_with_table_semantics_total": total_snapshots_with_table_semantics,
        "semantics_table_entries_total": table_entries_total,
        "semantics_table_unique_keys_total": table_unique_keys.len(),
        "unique_semantics_fingerprints_total": total_unique_semantics_fingerprints.len(),
        "windows": windows_out,
    })
}

pub(super) fn build_test_ids_index_json(
    bundle_label: &str,
    windows: &[UiDiagnosticsWindowBundleV1],
    semantics_table: Option<&bundle::UiBundleSemanticsTableV1>,
) -> Value {
    let warmup_frames = DEFAULT_WARMUP_FRAMES;
    let semantics_table = semantics_table_map(semantics_table);

    const MAX_UNIQUE_TEST_IDS_BUDGET: usize = 50_000;

    let mut total_unique: HashSet<String> = HashSet::new();
    let mut windows_out: Vec<Value> = Vec::new();
    let mut truncated: bool = false;

    for w in windows {
        let Some(snapshot) = pick_last_snapshot_with_resolved_semantics(
            &w.snapshots,
            warmup_frames,
            &semantics_table,
        ) else {
            continue;
        };

        let nodes = resolve_semantics_snapshot(snapshot, &semantics_table)
            .map(|s| s.nodes.as_slice())
            .unwrap_or(&[]);

        let mut counts: HashMap<String, u64> = HashMap::new();
        for n in nodes {
            let Some(raw) = n.test_id.as_deref() else {
                continue;
            };
            let test_id = raw.trim();
            if test_id.is_empty() {
                continue;
            }
            if !counts.contains_key(test_id)
                && (total_unique.len() + counts.len()) >= MAX_UNIQUE_TEST_IDS_BUDGET
            {
                truncated = true;
                continue;
            }
            *counts.entry(test_id.to_string()).or_insert(0) += 1;
        }

        for k in counts.keys() {
            if total_unique.len() >= MAX_UNIQUE_TEST_IDS_BUDGET {
                truncated = true;
                break;
            }
            total_unique.insert(k.clone());
        }

        let semantics_nodes_total = nodes.len() as u64;
        let test_id_nodes_total: u64 = counts.values().sum();
        let unique_test_ids_total = counts.len() as u64;

        let mut items: Vec<(String, u64)> = counts.into_iter().collect();
        items.sort_by(|(a_id, a_count), (b_id, b_count)| {
            b_count.cmp(a_count).then_with(|| a_id.cmp(b_id))
        });

        let duplicates_total = items.iter().filter(|(_id, count)| *count > 1).count() as u64;
        let duplicates_top: Vec<Value> = items
            .iter()
            .filter(|(_id, count)| *count > 1)
            .take(100)
            .map(|(test_id, count)| json!({ "test_id": test_id, "count": count }))
            .collect();

        windows_out.push(json!({
            "window": w.window,
            "frame_id": snapshot.frame_id,
            "semantics_nodes_total": semantics_nodes_total,
            "test_id_nodes_total": test_id_nodes_total,
            "unique_test_ids_total": unique_test_ids_total,
            "duplicate_test_ids_total": duplicates_total,
            "duplicate_test_ids_top": duplicates_top,
            "items": items.into_iter().map(|(test_id, count)| json!({
                "test_id": test_id,
                "count": count,
            })).collect::<Vec<_>>(),
        }));
    }

    json!({
        "schema_version": 1,
        "kind": "test_ids_index",
        "bundle": bundle_label,
        "warmup_frames": warmup_frames,
        "max_unique_test_ids_budget": MAX_UNIQUE_TEST_IDS_BUDGET,
        "truncated": truncated,
        "total_unique_test_ids": total_unique.len(),
        "windows": windows_out,
    })
}
