use super::*;
use std::any::{Any, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct BoundaryId(NodeId);

impl BoundaryId {
    pub(crate) fn from_node(node: NodeId) -> Self {
        Self(node)
    }

    pub(super) fn node(self) -> NodeId {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ViewBoundaryKind {
    Node,
    ViewCacheRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BoundaryParentLayoutDependency {
    ParentDependent,
    ContainedWhenBoundsKnown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BoundaryLayoutDependencies {
    pub(super) parent: BoundaryParentLayoutDependency,
    pub(super) layout_definite: bool,
}

impl BoundaryLayoutDependencies {
    fn from_view_cache_flags(flags: ViewCacheFlags) -> Self {
        let parent = if flags.contained_layout {
            BoundaryParentLayoutDependency::ContainedWhenBoundsKnown
        } else {
            BoundaryParentLayoutDependency::ParentDependent
        };
        Self {
            parent,
            layout_definite: flags.layout_definite,
        }
    }

    pub(super) fn allows_contained_relayout(self) -> bool {
        self.parent == BoundaryParentLayoutDependency::ContainedWhenBoundsKnown
            && self.layout_definite
    }
}

pub(super) struct ViewBoundaryState {
    pub(super) id: BoundaryId,
    pub(super) parent: Option<BoundaryId>,
    pub(super) kind: ViewBoundaryKind,
    pub(super) layout_dependencies: BoundaryLayoutDependencies,
    pub(super) prepaint: BoundaryPrepaintState,
}

impl ViewBoundaryState {
    fn new(id: BoundaryId, parent: Option<BoundaryId>, flags: ViewCacheFlags) -> Self {
        Self {
            id,
            parent,
            kind: if flags.enabled {
                ViewBoundaryKind::ViewCacheRoot
            } else {
                ViewBoundaryKind::Node
            },
            layout_dependencies: BoundaryLayoutDependencies::from_view_cache_flags(flags),
            prepaint: BoundaryPrepaintState::default(),
        }
    }

    fn refresh(&mut self, parent: Option<BoundaryId>, flags: ViewCacheFlags) {
        self.parent = parent;
        self.kind = if flags.enabled {
            ViewBoundaryKind::ViewCacheRoot
        } else {
            ViewBoundaryKind::Node
        };
        self.layout_dependencies = BoundaryLayoutDependencies::from_view_cache_flags(flags);
    }
}

#[derive(Default)]
pub(super) struct BoundaryPrepaintState {
    outputs: PrepaintOutputs,
}

impl BoundaryPrepaintState {
    pub(super) fn begin_outputs(&mut self, key: PaintCacheKey) {
        self.outputs.begin_frame(key);
    }

    pub(super) fn set_output<T: Any>(&mut self, value: T) {
        self.outputs.set(value);
    }

    pub(super) fn set_output_box(&mut self, ty: TypeId, value: Box<dyn Any>) {
        self.outputs.set_box(ty, value);
    }

    pub(super) fn output<T: Any>(&self) -> Option<&T> {
        self.outputs.get::<T>()
    }

    pub(super) fn output_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.outputs.get_mut::<T>()
    }

    pub(super) fn output_any(&self, ty: TypeId) -> Option<&dyn Any> {
        self.outputs.get_any(ty)
    }

    pub(super) fn output_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        self.outputs.get_any_mut(ty)
    }
}

#[derive(Default)]
struct PrepaintOutputs {
    key: Option<PaintCacheKey>,
    values: Vec<(TypeId, Box<dyn Any>)>,
}

impl PrepaintOutputs {
    fn begin_frame(&mut self, key: PaintCacheKey) {
        if self.key != Some(key) {
            self.key = Some(key);
            self.values.clear();
        }
    }

    fn set<T: Any>(&mut self, value: T) {
        self.set_box(TypeId::of::<T>(), Box::new(value));
    }

    fn set_box(&mut self, ty: TypeId, value: Box<dyn Any>) {
        if let Some((_, existing)) = self.values.iter_mut().find(|(id, _)| *id == ty) {
            *existing = value;
            return;
        }
        self.values.push((ty, value));
    }

    fn get<T: Any>(&self) -> Option<&T> {
        self.get_any(TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.get_any_mut(TypeId::of::<T>())
            .and_then(|value| value.downcast_mut::<T>())
    }

    fn get_any(&self, ty: TypeId) -> Option<&dyn Any> {
        self.values
            .iter()
            .find(|(id, _)| *id == ty)
            .map(|(_, value)| value.as_ref())
    }

    fn get_any_mut(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        self.values
            .iter_mut()
            .find(|(id, _)| *id == ty)
            .map(|(_, value)| value.as_mut())
    }
}

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn ensure_view_boundary_state(
        &mut self,
        node: NodeId,
    ) -> Option<&mut ViewBoundaryState> {
        let flags = self.nodes.get(node)?.view_cache;
        let parent = self.nearest_parent_view_boundary(node);

        if !self.view_boundaries.contains_key(node) {
            self.view_boundaries.insert(
                node,
                ViewBoundaryState::new(BoundaryId::from_node(node), parent, flags),
            );
        }

        let state = self.view_boundaries.get_mut(node)?;
        state.refresh(parent, flags);
        Some(state)
    }

    pub(in crate::tree) fn sync_view_boundary_state_for_node(&mut self, node: NodeId) {
        if self.nodes.get(node).is_none() {
            self.view_boundaries.remove(node);
            return;
        }

        if self.view_boundaries.contains_key(node)
            || self
                .nodes
                .get(node)
                .is_some_and(|n| n.view_cache.enabled || n.widget_prepaint_enabled)
        {
            let _ = self.ensure_view_boundary_state(node);
        }
    }

    pub(in crate::tree) fn remove_view_boundary_state(&mut self, node: NodeId) {
        self.view_boundaries.remove(node);
    }

    fn nearest_parent_view_boundary(&self, node: NodeId) -> Option<BoundaryId> {
        let mut current = self.nodes.get(node).and_then(|n| n.parent);
        while let Some(id) = current {
            if self.view_boundaries.contains_key(id)
                || self
                    .nodes
                    .get(id)
                    .is_some_and(|n| n.view_cache.enabled || n.widget_prepaint_enabled)
            {
                return Some(BoundaryId::from_node(id));
            }
            current = self.nodes.get(id).and_then(|n| n.parent);
        }
        None
    }

    pub fn debug_boundary_prepaint_owner_for_node(&self, node: NodeId) -> &'static str {
        if self.view_boundaries.contains_key(node) {
            "view_boundary_prepaint_state"
        } else {
            "none"
        }
    }

    pub(in crate::tree) fn boundary_allows_contained_relayout(&self, node: NodeId) -> bool {
        self.view_boundaries.get(node).is_some_and(|state| {
            debug_assert_eq!(state.id.node(), node);
            state.layout_dependencies.allows_contained_relayout()
        })
    }

    pub fn debug_boundary_stats(&self) -> Vec<UiDebugBoundaryStats> {
        if !self.debug_enabled {
            return Vec::new();
        }

        let mut out: Vec<UiDebugBoundaryStats> = self
            .view_boundaries
            .iter()
            .map(|(node, state)| UiDebugBoundaryStats {
                id: state.id.node(),
                parent: state.parent.map(BoundaryId::node),
                element: self.nodes.get(node).and_then(|n| n.element),
                kind: match state.kind {
                    ViewBoundaryKind::Node => "node",
                    ViewBoundaryKind::ViewCacheRoot => "view_cache_root",
                },
                source: match state.kind {
                    ViewBoundaryKind::Node => "runtime",
                    ViewBoundaryKind::ViewCacheRoot => "view_cache",
                },
                prepaint_owner: "view_boundary_prepaint_state",
                layout_dependency: match state.layout_dependencies.parent {
                    BoundaryParentLayoutDependency::ParentDependent => "parent_dependent",
                    BoundaryParentLayoutDependency::ContainedWhenBoundsKnown => {
                        "contained_when_bounds_known"
                    }
                },
                layout_definite: state.layout_dependencies.layout_definite,
            })
            .collect();
        out.sort_by_key(|stats| stats.id.data().as_ffi());
        out
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_exists(&self, node: NodeId) -> bool {
        self.view_boundaries.contains_key(node)
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_parent(&self, node: NodeId) -> Option<NodeId> {
        self.view_boundaries
            .get(node)
            .and_then(|state| state.parent)
            .map(BoundaryId::node)
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_prepaint_output<T: Any>(&self, node: NodeId) -> Option<&T> {
        self.view_boundaries.get(node)?.prepaint.output::<T>()
    }

    #[cfg(test)]
    pub(crate) fn test_view_boundary_allows_contained_relayout(&self, node: NodeId) -> bool {
        self.view_boundaries
            .get(node)
            .is_some_and(|state| state.layout_dependencies.allows_contained_relayout())
    }
}
