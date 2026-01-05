use fret_core::NodeId;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

/// Resolve an active descendant `NodeId` from a list of element IDs and an active index.
///
/// This is a small helper for cmdk/listbox-like composite widgets where:
/// - focus stays on an owner node (often a `TextField`), and
/// - the highlighted option is exposed via `active_descendant` (ADR 0073).
pub fn active_descendant_for_index<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    elements: &[GlobalElementId],
    active_index: Option<usize>,
) -> Option<NodeId> {
    let element = active_index.and_then(|idx| elements.get(idx).copied())?;
    cx.node_for_element(element)
}
