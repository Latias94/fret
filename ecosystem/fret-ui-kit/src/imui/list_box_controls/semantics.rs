use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::SemanticsDecoration;

pub(in crate::imui::list_box_controls) fn list_box_semantics(
    label: Option<Arc<str>>,
    multiselectable: bool,
) -> SemanticsDecoration {
    let mut semantics = SemanticsDecoration::default().role(SemanticsRole::ListBox);
    if let Some(label) = label {
        semantics = semantics.label(label);
    }
    if multiselectable {
        semantics = semantics.multiselectable(true);
    }
    semantics
}
