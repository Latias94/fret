use serde_json::Value;
use std::collections::HashMap;

pub(crate) fn pick_last_snapshot_after_warmup<'a>(
    snaps: &'a [Value],
    warmup_frames: u64,
) -> Option<&'a Value> {
    snaps
        .iter()
        .rev()
        .find(|s| snapshot_frame_id(s) >= warmup_frames)
        .or_else(|| snaps.last())
}

pub(crate) fn pick_last_snapshot_with_semantics_after_warmup<'a>(
    snaps: &'a [Value],
    warmup_frames: u64,
) -> Option<&'a Value> {
    snaps
        .iter()
        .rev()
        .find(|s| snapshot_frame_id(s) >= warmup_frames && snapshot_semantics_nodes(s).is_some())
        .or_else(|| {
            snaps
                .iter()
                .rev()
                .find(|s| snapshot_semantics_nodes(s).is_some())
        })
        .or_else(|| pick_last_snapshot_after_warmup(snaps, warmup_frames))
}

pub(crate) fn pick_last_snapshot_with_resolved_semantics_after_warmup<'a>(
    snaps: &'a [Value],
    warmup_frames: u64,
    semantics: &SemanticsResolver<'a>,
) -> Option<&'a Value> {
    snaps
        .iter()
        .rev()
        .find(|s| snapshot_frame_id(s) >= warmup_frames && semantics.nodes(s).is_some())
        .or_else(|| snaps.iter().rev().find(|s| semantics.nodes(s).is_some()))
        .or_else(|| pick_last_snapshot_after_warmup(snaps, warmup_frames))
}

pub(crate) fn snapshot_frame_id(snapshot: &Value) -> u64 {
    snapshot
        .get("frame_id")
        .and_then(|v| v.as_u64())
        .or_else(|| snapshot.get("frameId").and_then(|v| v.as_u64()))
        .unwrap_or(0)
}

pub(crate) fn snapshot_window_snapshot_seq(snapshot: &Value) -> Option<u64> {
    snapshot
        .get("window_snapshot_seq")
        .and_then(|v| v.as_u64())
        .or_else(|| snapshot.get("windowSnapshotSeq").and_then(|v| v.as_u64()))
}

pub(crate) fn snapshot_window_id(snapshot: &Value) -> Option<u64> {
    snapshot
        .get("window")
        .and_then(|v| v.as_u64())
        .or_else(|| snapshot.get("window_id").and_then(|v| v.as_u64()))
        .or_else(|| snapshot.get("windowId").and_then(|v| v.as_u64()))
}

pub(crate) fn snapshot_semantics_fingerprint(snapshot: &Value) -> Option<u64> {
    snapshot
        .get("semantics_fingerprint")
        .and_then(|v| v.as_u64())
        .or_else(|| {
            snapshot
                .get("semanticsFingerprint")
                .and_then(|v| v.as_u64())
        })
}

pub(crate) fn snapshot_semantics(snapshot: &Value) -> Option<&Value> {
    snapshot
        .get("debug")
        .and_then(|v| v.get("semantics"))
        .or_else(|| snapshot.get("semantics"))
        .or_else(|| snapshot.get("semantic_tree"))
        .or_else(|| snapshot.get("semanticTree"))
        .or_else(|| snapshot.get("tree"))
}

pub(crate) fn snapshot_semantics_nodes(snapshot: &Value) -> Option<&[Value]> {
    snapshot_semantics(snapshot)?
        .get("nodes")
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
}

fn bundle_semantics_table_entries(bundle: &Value) -> Option<&[Value]> {
    bundle
        .get("tables")
        .and_then(|v| v.get("semantics"))
        .and_then(|v| v.get("entries"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
}

pub(crate) struct SemanticsResolver<'a> {
    entries: Option<&'a [Value]>,
    by_window_fp: HashMap<(u64, u64), usize>,
}

impl<'a> SemanticsResolver<'a> {
    pub(crate) fn new(bundle: &'a Value) -> Self {
        let entries = bundle_semantics_table_entries(bundle);
        let mut by_window_fp: HashMap<(u64, u64), usize> = HashMap::new();
        if let Some(entries) = entries {
            for (idx, e) in entries.iter().enumerate() {
                let Some(window) = e.get("window").and_then(|v| v.as_u64()) else {
                    continue;
                };
                let Some(fp) = e.get("semantics_fingerprint").and_then(|v| v.as_u64()) else {
                    continue;
                };
                by_window_fp.insert((window, fp), idx);
            }
        }
        Self {
            entries,
            by_window_fp,
        }
    }

    pub(crate) fn semantics_snapshot(&self, snapshot: &'a Value) -> Option<&'a Value> {
        if let Some(sem) = snapshot_semantics(snapshot) {
            return Some(sem);
        }
        let entries = self.entries?;
        let window = snapshot_window_id(snapshot)?;
        let fp = snapshot_semantics_fingerprint(snapshot)?;
        let idx = *self.by_window_fp.get(&(window, fp))?;
        entries
            .get(idx)
            .and_then(|e| e.get("semantics"))
            .or_else(|| entries.get(idx).and_then(|e| e.get("semantic")))
    }

    pub(crate) fn nodes(&self, snapshot: &'a Value) -> Option<&'a [Value]> {
        if let Some(nodes) = snapshot_semantics_nodes(snapshot) {
            return Some(nodes);
        }
        self.semantics_snapshot(snapshot)?
            .get("nodes")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
    }
}
