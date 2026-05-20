use fret_core::{Point, Rect, Size};
use fret_runtime::{Model, ModelHost};

use crate::core::{EdgeId, NodeId};
use crate::io::NodeGraphViewState;
use crate::ui::NodeGraphInternalsStore;
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

pub(super) fn resolve_node_toolbar_target<H: ModelHost>(
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

pub(super) fn resolve_edge_toolbar_target<H: ModelHost>(
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

pub(super) fn resolve_node_toolbar_window_target<H: ModelHost>(
    view_state: &Model<NodeGraphViewState>,
    requested_node: Option<NodeId>,
    internals: &NodeGraphInternalsStore,
    host: &H,
) -> Option<(Rect, bool)> {
    let (node_id, selected) = resolve_node_toolbar_target(view_state, requested_node, host)?;
    internals
        .snapshot()
        .nodes_window
        .get(&node_id)
        .copied()
        .map(|rect| (rect, selected))
}

pub(super) fn resolve_edge_toolbar_window_target<H: ModelHost>(
    view_state: &Model<NodeGraphViewState>,
    requested_edge: Option<EdgeId>,
    internals: &NodeGraphInternalsStore,
    host: &H,
) -> Option<(Point, bool)> {
    let (edge_id, selected) = resolve_edge_toolbar_target(view_state, requested_edge, host)?;
    internals
        .snapshot()
        .edge_centers_window
        .get(&edge_id)
        .copied()
        .map(|center| (center, selected))
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    use fret_core::{Point, Px, Rect, Size};
    use fret_runtime::{ModelHost, ModelStore};

    use super::{
        NodeGraphToolbarAlign, NodeGraphToolbarPosition, NodeGraphToolbarVisibility,
        resolve_edge_toolbar_window_target, resolve_node_toolbar_window_target, toolbar_align_axis,
        toolbar_position_to_adjacent, toolbar_visible,
    };
    use crate::core::{EdgeId, NodeId};
    use crate::io::NodeGraphViewState;
    use crate::ui::internals::{NodeGraphInternalsSnapshot, NodeGraphInternalsStore};
    use crate::ui::screen_space_placement::{AdjacentPosition, AxisAlign};

    #[derive(Default)]
    struct TestModelHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
    }

    impl fret_runtime::GlobalsHost for TestModelHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            let existing = self.globals.remove(&type_id);
            let mut value = existing
                .and_then(|v| v.downcast::<T>().ok().map(|v| *v))
                .unwrap_or_else(init);
            let out = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            out
        }
    }

    impl ModelHost for TestModelHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

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

    #[test]
    fn node_toolbar_window_target_resolves_selected_fallback_and_requested_nodes() {
        let mut host = TestModelHost::default();
        let node_a = NodeId::from_u128(3101);
        let node_b = NodeId::from_u128(3102);
        let missing = NodeId::from_u128(3103);
        let rect_a = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(30.0), Px(40.0)),
        );
        let rect_b = Rect::new(
            Point::new(Px(50.0), Px(60.0)),
            Size::new(Px(70.0), Px(80.0)),
        );
        let mut view = NodeGraphViewState::default();
        view.selected_nodes = vec![node_b];
        let view = host.models_mut().insert(view);

        let internals = NodeGraphInternalsStore::new();
        let mut snapshot = NodeGraphInternalsSnapshot::default();
        snapshot.nodes_window.insert(node_a, rect_a);
        snapshot.nodes_window.insert(node_b, rect_b);
        internals.update(snapshot);

        assert_eq!(
            resolve_node_toolbar_window_target(&view, None, &internals, &host),
            Some((rect_b, true))
        );
        assert_eq!(
            resolve_node_toolbar_window_target(&view, Some(node_a), &internals, &host),
            Some((rect_a, false))
        );
        assert_eq!(
            resolve_node_toolbar_window_target(&view, Some(node_b), &internals, &host),
            Some((rect_b, true))
        );
        assert_eq!(
            resolve_node_toolbar_window_target(&view, Some(missing), &internals, &host),
            None
        );
    }

    #[test]
    fn edge_toolbar_window_target_resolves_selected_fallback_and_requested_edges() {
        let mut host = TestModelHost::default();
        let edge_a = EdgeId::from_u128(4101);
        let edge_b = EdgeId::from_u128(4102);
        let missing = EdgeId::from_u128(4103);
        let center_a = Point::new(Px(15.0), Px(25.0));
        let center_b = Point::new(Px(55.0), Px(65.0));
        let mut view = NodeGraphViewState::default();
        view.selected_edges = vec![edge_b];
        let view = host.models_mut().insert(view);

        let internals = NodeGraphInternalsStore::new();
        let mut snapshot = NodeGraphInternalsSnapshot::default();
        snapshot.edge_centers_window.insert(edge_a, center_a);
        snapshot.edge_centers_window.insert(edge_b, center_b);
        internals.update(snapshot);

        assert_eq!(
            resolve_edge_toolbar_window_target(&view, None, &internals, &host),
            Some((center_b, true))
        );
        assert_eq!(
            resolve_edge_toolbar_window_target(&view, Some(edge_a), &internals, &host),
            Some((center_a, false))
        );
        assert_eq!(
            resolve_edge_toolbar_window_target(&view, Some(edge_b), &internals, &host),
            Some((center_b, true))
        );
        assert_eq!(
            resolve_edge_toolbar_window_target(&view, Some(missing), &internals, &host),
            None
        );
    }
}
