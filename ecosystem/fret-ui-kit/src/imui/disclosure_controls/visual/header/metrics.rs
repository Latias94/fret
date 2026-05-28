use std::sync::Arc;

use fret_core::{Edges, Px};

use super::{DisclosureKind, DisclosureSpec};

pub(super) fn header_indicator(spec: &DisclosureSpec, open_now: bool) -> Option<Arc<str>> {
    if spec.leaf {
        None
    } else if open_now {
        Some(Arc::from("v"))
    } else {
        Some(Arc::from(">"))
    }
}

pub(super) fn header_row_padding(spec: &DisclosureSpec) -> Edges {
    match spec.kind {
        DisclosureKind::CollapsingHeader => Edges {
            top: Px(4.0),
            right: Px(6.0),
            bottom: Px(4.0),
            left: Px(6.0),
        },
        DisclosureKind::TreeNode => {
            let indent = Px(16.0 * (spec.level.saturating_sub(1) as f32));
            Edges {
                top: Px(2.0),
                right: Px(6.0),
                bottom: Px(2.0),
                left: Px(6.0 + indent.0),
            }
        }
    }
}

pub(super) fn header_border_edges(kind: DisclosureKind) -> Edges {
    match kind {
        DisclosureKind::CollapsingHeader => Edges::all(Px(1.0)),
        DisclosureKind::TreeNode => Edges::all(Px(0.0)),
    }
}
