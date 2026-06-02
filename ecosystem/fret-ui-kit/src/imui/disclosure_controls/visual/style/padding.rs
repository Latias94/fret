use fret_core::{Edges, Px};

use super::super::super::spec::{DisclosureKind, DisclosureSpec};

pub(in crate::imui::disclosure_controls) fn disclosure_content_padding(
    spec: &DisclosureSpec,
) -> Edges {
    match spec.kind {
        DisclosureKind::CollapsingHeader => Edges {
            top: Px(4.0),
            right: Px(0.0),
            bottom: Px(0.0),
            left: Px(0.0),
        },
        DisclosureKind::TreeNode => Edges {
            top: Px(0.0),
            right: Px(0.0),
            bottom: Px(0.0),
            left: Px(0.0),
        },
    }
}
