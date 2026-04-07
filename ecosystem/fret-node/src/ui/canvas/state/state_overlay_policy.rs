use crate::core::{CanvasPoint, EdgeId, GroupId, PortId};

#[derive(Debug, Clone)]
pub(crate) enum ContextMenuTarget {
    Background,
    BackgroundInsertNodePicker {
        at: CanvasPoint,
    },
    ConnectionInsertNodePicker {
        from: PortId,
        at: CanvasPoint,
    },
    Edge(EdgeId),
    EdgeInsertNodePicker(EdgeId),
    ConnectionConvertPicker {
        from: PortId,
        to: PortId,
        at: CanvasPoint,
    },
    Group(GroupId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearcherRowsMode {
    Catalog,
    Flat,
}
