use super::super::*;

#[derive(Debug, Clone)]
pub struct UiDebugDispatchSnapshotNode {
    pub node: NodeId,
    pub parent: Option<NodeId>,
    pub pre: u32,
    pub post: u32,
}

#[derive(Debug, Clone)]
pub struct UiDebugDispatchSnapshot {
    pub frame_id: FrameId,
    pub window: Option<AppWindowId>,
    pub active_layer_roots: Vec<NodeId>,
    pub barrier_root: Option<NodeId>,
    pub nodes: Vec<UiDebugDispatchSnapshotNode>,
}

#[derive(Debug, Clone)]
pub struct UiDebugDispatchSnapshotParityReport {
    pub frame_id: FrameId,
    pub window: Option<AppWindowId>,
    pub active_layer_roots: Vec<NodeId>,
    pub barrier_root: Option<NodeId>,

    pub reachable_count: usize,
    pub snapshot_count: usize,

    /// Nodes that are reachable from the active layer roots via child edges, but missing from the
    /// snapshot forest.
    pub missing_in_snapshot_total: usize,
    pub missing_in_snapshot_sample: Vec<NodeId>,

    /// Nodes that are present in the snapshot forest, but not reachable from the active layer
    /// roots via child edges.
    pub extra_in_snapshot_total: usize,
    pub extra_in_snapshot_sample: Vec<NodeId>,
}

impl UiDebugDispatchSnapshot {
    pub(in crate::tree) fn from_snapshot(snapshot: &UiDispatchSnapshot) -> Self {
        let mut nodes: Vec<UiDebugDispatchSnapshotNode> = Vec::with_capacity(snapshot.nodes.len());
        for &node in &snapshot.nodes {
            let pre = snapshot.pre.get(node).copied().unwrap_or_default();
            let post = snapshot.post.get(node).copied().unwrap_or_default();
            let parent = snapshot.parent.get(node).copied().flatten();
            nodes.push(UiDebugDispatchSnapshotNode {
                node,
                parent,
                pre,
                post,
            });
        }
        Self {
            frame_id: snapshot.frame_id,
            window: snapshot.window,
            active_layer_roots: snapshot.active_layer_roots.clone(),
            barrier_root: snapshot.barrier_root,
            nodes,
        }
    }
}
