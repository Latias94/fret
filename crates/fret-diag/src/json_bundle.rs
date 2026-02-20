use serde_json::Value;

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
