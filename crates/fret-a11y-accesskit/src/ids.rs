use accesskit::NodeId;
use slotmap::{Key, KeyData};

pub(crate) const ROOT_ID: NodeId = NodeId(0);
pub(crate) const SYNTHETIC_TEXT_RUN_BIT: u64 = 1 << 63;

pub(crate) fn to_accesskit_id(node: fret_core::NodeId) -> NodeId {
    NodeId(node.data().as_ffi().wrapping_add(1))
}

pub(crate) fn from_accesskit_id(node: NodeId) -> Option<fret_core::NodeId> {
    if node.0 == 0 {
        return None;
    }
    Some(fret_core::NodeId::from(KeyData::from_ffi(
        node.0.wrapping_sub(1),
    )))
}

pub(crate) fn text_run_id_for(node: fret_core::NodeId) -> NodeId {
    NodeId(to_accesskit_id(node).0 | SYNTHETIC_TEXT_RUN_BIT)
}

pub(crate) fn parent_from_synthetic_id(node: NodeId) -> Option<fret_core::NodeId> {
    if (node.0 & SYNTHETIC_TEXT_RUN_BIT) == 0 {
        return None;
    }
    from_accesskit_id(NodeId(node.0 & !SYNTHETIC_TEXT_RUN_BIT))
}
