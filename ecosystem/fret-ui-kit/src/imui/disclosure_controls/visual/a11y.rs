use fret_core::SemanticsRole;
use fret_ui::element::PressableA11y;

use super::super::spec::{DisclosureKind, DisclosureSpec};

pub(in crate::imui::disclosure_controls) fn disclosure_a11y(
    spec: &DisclosureSpec,
    open_now: bool,
) -> PressableA11y {
    match spec.kind {
        DisclosureKind::CollapsingHeader => PressableA11y {
            role: Some(SemanticsRole::Button),
            label: Some(spec.label.clone()),
            expanded: spec.has_children().then_some(open_now),
            ..Default::default()
        },
        DisclosureKind::TreeNode => PressableA11y {
            role: Some(SemanticsRole::TreeItem),
            label: Some(spec.label.clone()),
            level: Some(spec.level),
            selected: spec.selected,
            expanded: spec.has_children().then_some(open_now),
            pos_in_set: spec.pos_in_set,
            set_size: spec.set_size,
            ..Default::default()
        },
    }
}
