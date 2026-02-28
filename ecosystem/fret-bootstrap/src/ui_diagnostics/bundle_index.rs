use std::collections::{HashMap, HashSet};

use serde_json::{Value, json};

use super::{UiDiagnosticsSnapshotV1, UiDiagnosticsWindowBundleV1, UiSemanticsSnapshotV1, bundle};

const DEFAULT_WARMUP_FRAMES: u64 = 0;
const TEST_ID_BLOOM_TAIL_SNAPSHOTS_PER_WINDOW: usize = 64;
const TEST_ID_BLOOM_MAX_SEMANTICS_KEYS_PER_WINDOW: usize = 128;

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
    let mut semantics_blooms_windows: Vec<Value> = Vec::new();
    let mut semantics_blooms_keys_total: u64 = 0;

    for w in windows {
        total_snapshots = total_snapshots.saturating_add(w.snapshots.len() as u64);

        let mut snapshots_out: Vec<Value> = Vec::with_capacity(w.snapshots.len());

        let mut first_frame_id: Option<u64> = None;
        let mut last_frame_id: Option<u64> = None;
        let mut first_timestamp_unix_ms: Option<u64> = None;
        let mut last_timestamp_unix_ms: Option<u64> = None;

        let bloom_start_idx = w
            .snapshots
            .len()
            .saturating_sub(TEST_ID_BLOOM_TAIL_SNAPSHOTS_PER_WINDOW);

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

            let test_id_bloom_hex = if idx >= bloom_start_idx && has_semantics {
                let nodes = resolve_semantics_snapshot(s, &semantics_table)
                    .map(|s| s.nodes.as_slice())
                    .unwrap_or(&[]);
                let mut bloom = super::test_id_bloom::TestIdBloomV1::new();
                for n in nodes {
                    if let Some(test_id) = n.test_id.as_deref() {
                        bloom.add(test_id);
                    }
                }
                Some(bloom.to_hex())
            } else {
                None
            };

            snapshots_out.push(json!({
                "window_snapshot_seq": window_snapshot_seq,
                "frame_id": frame_id,
                "timestamp_unix_ms": ts,
                "is_warmup": (idx as u64) < warmup_frames,
                "semantics_fingerprint": s.semantics_fingerprint,
                "semantics_source": semantics_source,
                "has_semantics": has_semantics,
                "test_id_bloom_hex": test_id_bloom_hex,
            }));
        }

        let mut keys: HashSet<(u64, u8)> = HashSet::new();
        let mut items: Vec<Value> = Vec::new();
        for s in w.snapshots.iter().rev() {
            if keys.len() >= TEST_ID_BLOOM_MAX_SEMANTICS_KEYS_PER_WINDOW {
                break;
            }
            let Some(fp) = s.semantics_fingerprint else {
                continue;
            };

            let semantics_source = snapshot_semantics_source(s, &semantics_table);
            let source_tag = match semantics_source {
                "inline" => 0u8,
                "table" => 1u8,
                _ => continue,
            };
            if !keys.insert((fp, source_tag)) {
                continue;
            }

            let nodes = resolve_semantics_snapshot(s, &semantics_table)
                .map(|s| s.nodes.as_slice())
                .unwrap_or(&[]);
            let mut bloom = super::test_id_bloom::TestIdBloomV1::new();
            for n in nodes {
                if let Some(test_id) = n.test_id.as_deref() {
                    bloom.add(test_id);
                }
            }
            items.push(json!({
                "semantics_fingerprint": fp,
                "semantics_source": semantics_source,
                "test_id_bloom_hex": bloom.to_hex(),
            }));
        }

        if !items.is_empty() {
            semantics_blooms_keys_total =
                semantics_blooms_keys_total.saturating_add(items.len() as u64);
            semantics_blooms_windows.push(json!({
                "window": w.window,
                "items": items,
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
        "test_id_bloom": {
            "schema_version": super::test_id_bloom::TEST_ID_BLOOM_V1_SCHEMA_VERSION,
            "m_bits": super::test_id_bloom::TEST_ID_BLOOM_V1_M_BITS,
            "k": super::test_id_bloom::TEST_ID_BLOOM_V1_K,
            "tail_snapshots_per_window": TEST_ID_BLOOM_TAIL_SNAPSHOTS_PER_WINDOW,
            "computed_from": "resolved_semantics_nodes",
            "semantics_source": "resolved",
            "covers_semantics_sources": ["inline", "table"],
        },
        "semantics_blooms": {
            "schema_version": 1,
            "m_bits": super::test_id_bloom::TEST_ID_BLOOM_V1_M_BITS,
            "k": super::test_id_bloom::TEST_ID_BLOOM_V1_K,
            "computed_from": "resolved_semantics_nodes",
            "max_keys_per_window": TEST_ID_BLOOM_MAX_SEMANTICS_KEYS_PER_WINDOW,
            "keys_total": semantics_blooms_keys_total,
            "windows": semantics_blooms_windows,
        },
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

pub(super) fn build_window_map_json(
    bundle_label: &str,
    windows: &[UiDiagnosticsWindowBundleV1],
) -> Value {
    let warmup_frames = DEFAULT_WARMUP_FRAMES;

    let mut windows_out: Vec<Value> = Vec::with_capacity(windows.len());

    for w in windows {
        let first = w.snapshots.first();
        let last = w.snapshots.last();

        let last_bounds = last.map(|s| s.window_bounds.clone());
        let last_scale_factor = last.map(|s| s.scale_factor);
        let last_primary_pointer_type = last.and_then(|s| s.primary_pointer_type.clone());
        let last_hover_detection = last
            .and_then(|s| s.caps.as_ref())
            .map(|c| c.ui_window_hover_detection.clone());
        let last_docking_interaction_present = last
            .and_then(|s| s.debug.docking_interaction.as_ref())
            .is_some();

        windows_out.push(json!({
            "window": w.window,
            "snapshots_total": w.snapshots.len(),
            "events_total": w.events.len(),
            "first_seen": first.map(|s| json!({
                "tick_id": s.tick_id,
                "frame_id": s.frame_id,
                "timestamp_unix_ms": s.timestamp_unix_ms,
                "window_snapshot_seq": s.window_snapshot_seq,
            })),
            "last_seen": last.map(|s| json!({
                "tick_id": s.tick_id,
                "frame_id": s.frame_id,
                "timestamp_unix_ms": s.timestamp_unix_ms,
                "window_snapshot_seq": s.window_snapshot_seq,
                "window_bounds": last_bounds,
                "scale_factor": last_scale_factor,
                "primary_pointer_type": last_primary_pointer_type,
                "ui_window_hover_detection": last_hover_detection,
                "docking_interaction_present": last_docking_interaction_present,
            })),
        }));
    }

    json!({
        "schema_version": 1,
        "kind": "window_map",
        "bundle": bundle_label,
        "warmup_frames": warmup_frames,
        "windows_total": windows.len(),
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
