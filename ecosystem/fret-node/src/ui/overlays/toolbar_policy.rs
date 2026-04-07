use fret_core::Size;
use fret_runtime::Model;
use fret_ui::UiHost;

use crate::core::{EdgeId, NodeId};
use crate::io::NodeGraphViewState;
use crate::ui::screen_space_placement::{AdjacentPosition, AxisAlign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeGraphToolbarVisibility {
    /// Show only when the target node or edge is selected.
    WhenSelected,
    /// Show whenever the target node or edge exists.
    Always,
}

impl Default for NodeGraphToolbarVisibility {
    fn default() -> Self {
        Self::WhenSelected
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeGraphToolbarPosition {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeGraphToolbarAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeGraphToolbarSize {
    /// Measure the child (uses `Widget::measure`).
    Auto,
    /// Fixed size in window-space logical pixels.
    Fixed(Size),
}

pub(super) fn toolbar_visible(
    visibility: NodeGraphToolbarVisibility,
    target_selected: bool,
) -> bool {
    match visibility {
        NodeGraphToolbarVisibility::WhenSelected => target_selected,
        NodeGraphToolbarVisibility::Always => true,
    }
}

pub(super) fn toolbar_align_axis(align: NodeGraphToolbarAlign) -> AxisAlign {
    match align {
        NodeGraphToolbarAlign::Start => AxisAlign::Start,
        NodeGraphToolbarAlign::Center => AxisAlign::Center,
        NodeGraphToolbarAlign::End => AxisAlign::End,
    }
}

pub(super) fn toolbar_position_to_adjacent(position: NodeGraphToolbarPosition) -> AdjacentPosition {
    match position {
        NodeGraphToolbarPosition::Top => AdjacentPosition::Top,
        NodeGraphToolbarPosition::Right => AdjacentPosition::Right,
        NodeGraphToolbarPosition::Bottom => AdjacentPosition::Bottom,
        NodeGraphToolbarPosition::Left => AdjacentPosition::Left,
    }
}

pub(super) fn resolve_node_toolbar_target<H: UiHost>(
    view_state: &Model<NodeGraphViewState>,
    requested_node: Option<NodeId>,
    host: &H,
) -> Option<(NodeId, bool)> {
    view_state
        .read_ref(host, |state| {
            if let Some(node) = requested_node {
                Some((node, state.selected_nodes.contains(&node)))
            } else {
                state.selected_nodes.first().copied().map(|id| (id, true))
            }
        })
        .ok()
        .flatten()
}

pub(super) fn resolve_edge_toolbar_target<H: UiHost>(
    view_state: &Model<NodeGraphViewState>,
    requested_edge: Option<EdgeId>,
    host: &H,
) -> Option<(EdgeId, bool)> {
    view_state
        .read_ref(host, |state| {
            if let Some(edge) = requested_edge {
                Some((edge, state.selected_edges.contains(&edge)))
            } else {
                state.selected_edges.first().copied().map(|id| (id, true))
            }
        })
        .ok()
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::{
        NodeGraphToolbarAlign, NodeGraphToolbarPosition, NodeGraphToolbarVisibility,
        toolbar_align_axis, toolbar_position_to_adjacent, toolbar_visible,
    };
    use crate::ui::screen_space_placement::{AdjacentPosition, AxisAlign};

    #[test]
    fn visibility_default_is_when_selected() {
        assert_eq!(
            NodeGraphToolbarVisibility::default(),
            NodeGraphToolbarVisibility::WhenSelected
        );
    }

    #[test]
    fn when_selected_visibility_requires_selected_target() {
        assert!(!toolbar_visible(
            NodeGraphToolbarVisibility::WhenSelected,
            false
        ));
        assert!(toolbar_visible(
            NodeGraphToolbarVisibility::WhenSelected,
            true
        ));
    }

    #[test]
    fn always_visibility_ignores_selection() {
        assert!(toolbar_visible(NodeGraphToolbarVisibility::Always, false));
        assert!(toolbar_visible(NodeGraphToolbarVisibility::Always, true));
    }

    #[test]
    fn toolbar_align_mapping_stays_stable() {
        assert_eq!(
            toolbar_align_axis(NodeGraphToolbarAlign::Start),
            AxisAlign::Start
        );
        assert_eq!(
            toolbar_align_axis(NodeGraphToolbarAlign::Center),
            AxisAlign::Center
        );
        assert_eq!(
            toolbar_align_axis(NodeGraphToolbarAlign::End),
            AxisAlign::End
        );
    }

    #[test]
    fn toolbar_position_mapping_stays_stable() {
        assert_eq!(
            toolbar_position_to_adjacent(NodeGraphToolbarPosition::Top),
            AdjacentPosition::Top
        );
        assert_eq!(
            toolbar_position_to_adjacent(NodeGraphToolbarPosition::Right),
            AdjacentPosition::Right
        );
        assert_eq!(
            toolbar_position_to_adjacent(NodeGraphToolbarPosition::Bottom),
            AdjacentPosition::Bottom
        );
        assert_eq!(
            toolbar_position_to_adjacent(NodeGraphToolbarPosition::Left),
            AdjacentPosition::Left
        );
    }
}
