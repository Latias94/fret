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

