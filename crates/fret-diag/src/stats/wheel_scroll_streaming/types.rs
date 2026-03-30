use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub(super) struct SnapshotMeta {
    pub(super) frame_id: u64,
    pub(super) semantics_fingerprint: Option<u64>,
    pub(super) semantics_window_id: u64,
    pub(super) hit: Option<u64>,
    pub(super) vlist_offset: Option<f64>,
}

#[derive(Debug, Clone)]
pub(super) struct WindowWheelMeta {
    pub(super) window_id: u64,
    pub(super) wheel_frame: u64,
    pub(super) before: Option<SnapshotMeta>,
    pub(super) after: Option<SnapshotMeta>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct SemanticsLite {
    pub(super) target_node_id: Option<u64>,
    pub(super) parents: HashMap<u64, u64>,
}

pub(super) fn semantics_lite_from_nodes(
    nodes: &[serde_json::Value],
    test_id: &str,
) -> SemanticsLite {
    let mut out = SemanticsLite::default();
    for node in nodes {
        let Some(id) = node.get("id").and_then(|v| v.as_u64()) else {
            continue;
        };
        if let Some(parent) = node.get("parent").and_then(|v| v.as_u64()) {
            out.parents.insert(id, parent);
        }
        if out.target_node_id.is_none()
            && node
                .get("test_id")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s == test_id)
        {
            out.target_node_id = Some(id);
        }
    }
    out
}

pub(super) fn resolve_semantics_lite(
    bundle_path: &Path,
    inline: &HashMap<(u64, u64), Option<SemanticsLite>>,
    snapshot: &SnapshotMeta,
    window_id: u64,
    test_id: &str,
) -> Result<Option<SemanticsLite>, String> {
    if let Some(v) = inline.get(&(window_id, snapshot.frame_id))
        && v.is_some()
    {
        return Ok(v.clone());
    }
    let Some(fp) = snapshot.semantics_fingerprint else {
        return Ok(None);
    };
    let nodes = crate::json_bundle::stream_read_semantics_table_nodes(
        bundle_path,
        snapshot.semantics_window_id,
        fp,
    )?;
    let Some(nodes) = nodes else {
        return Ok(None);
    };
    Ok(Some(semantics_lite_from_nodes(nodes.as_slice(), test_id)))
}
