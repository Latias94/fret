//! Node graph toolbars (UI-only).

use std::sync::Arc;

use fret_core::{Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
use fret_ui::{UiHost, retained_bridge::*};

use crate::core::{EdgeId, NodeId};
use crate::io::NodeGraphViewState;
use crate::ui::NodeGraphInternalsStore;

use super::{clamp_rect_to_bounds, layout_hidden_child_and_release_focus};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeGraphToolbarVisibility {
    /// Show only when the target node is selected.
    WhenSelected,
    /// Show whenever the target node exists (independent of selection).
    Always,
}

impl Default for NodeGraphToolbarVisibility {
    fn default() -> Self {
        Self::WhenSelected
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeGraphToolbarSize {
    /// Measure the child (uses `Widget::measure`).
    Auto,
    /// Fixed size in window-space logical pixels.
    Fixed(Size),
}

/// A window-space toolbar anchored to a node's derived window rect (XyFlow `NodeToolbar`-style).
///
/// Expected children:
/// - child 0: the toolbar content widget (should implement `Widget::measure` for `Auto` sizing).
pub struct NodeGraphNodeToolbar {
    canvas_node: fret_core::NodeId,
    graph: Model<crate::Graph>,
    view_state: Model<NodeGraphViewState>,
    internals: Arc<NodeGraphInternalsStore>,

    node: Option<NodeId>,
    visibility: NodeGraphToolbarVisibility,
    position: NodeGraphToolbarPosition,
    align: NodeGraphToolbarAlign,
    size: NodeGraphToolbarSize,
    gap_px: f32,
    offset: Point,

    last_child_bounds: Option<Rect>,
}

impl NodeGraphNodeToolbar {
    pub fn new(
        canvas_node: fret_core::NodeId,
        graph: Model<crate::Graph>,
        view_state: Model<NodeGraphViewState>,
        internals: Arc<NodeGraphInternalsStore>,
    ) -> Self {
        Self {
            canvas_node,
            graph,
            view_state,
            internals,
            node: None,
            visibility: NodeGraphToolbarVisibility::WhenSelected,
            position: NodeGraphToolbarPosition::Top,
            align: NodeGraphToolbarAlign::Center,
            size: NodeGraphToolbarSize::Auto,
            gap_px: 8.0,
            offset: Point::new(Px(0.0), Px(0.0)),
            last_child_bounds: None,
        }
    }

    /// Anchors the toolbar to a specific node id.
    pub fn for_node(mut self, node: NodeId) -> Self {
        self.node = Some(node);
        self
    }

    pub fn with_visibility(mut self, visibility: NodeGraphToolbarVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn with_position(mut self, position: NodeGraphToolbarPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_align(mut self, align: NodeGraphToolbarAlign) -> Self {
        self.align = align;
        self
    }

    pub fn with_gap_px(mut self, gap_px: f32) -> Self {
        self.gap_px = gap_px;
        self
    }

    pub fn with_offset_px(mut self, dx: f32, dy: f32) -> Self {
        self.offset = Point::new(Px(dx), Px(dy));
        self
    }

    pub fn with_size(mut self, size: NodeGraphToolbarSize) -> Self {
        self.size = size;
        self
    }

    fn positioned_rect_for(
        bounds: Rect,
        node: Rect,
        size: Size,
        position: NodeGraphToolbarPosition,
        align: NodeGraphToolbarAlign,
        gap_px: f32,
        offset: Point,
    ) -> Rect {
        let gap = gap_px.max(0.0);
        let w = size.width.0.max(0.0);
        let h = size.height.0.max(0.0);

        let x_start = node.origin.x.0;
        let x_center = node.origin.x.0 + 0.5 * (node.size.width.0 - w);
        let x_end = node.origin.x.0 + (node.size.width.0 - w).max(0.0);

        let y_start = node.origin.y.0;
        let y_center = node.origin.y.0 + 0.5 * (node.size.height.0 - h);
        let y_end = node.origin.y.0 + (node.size.height.0 - h).max(0.0);

        let (x, y) = match position {
            NodeGraphToolbarPosition::Top => {
                let x = match align {
                    NodeGraphToolbarAlign::Start => x_start,
                    NodeGraphToolbarAlign::Center => x_center,
                    NodeGraphToolbarAlign::End => x_end,
                };
                (x, node.origin.y.0 - gap - h)
            }
            NodeGraphToolbarPosition::Bottom => {
                let x = match align {
                    NodeGraphToolbarAlign::Start => x_start,
                    NodeGraphToolbarAlign::Center => x_center,
                    NodeGraphToolbarAlign::End => x_end,
                };
                (x, node.origin.y.0 + node.size.height.0 + gap)
            }
            NodeGraphToolbarPosition::Left => {
                let y = match align {
                    NodeGraphToolbarAlign::Start => y_start,
                    NodeGraphToolbarAlign::Center => y_center,
                    NodeGraphToolbarAlign::End => y_end,
                };
                (node.origin.x.0 - gap - w, y)
            }
            NodeGraphToolbarPosition::Right => {
                let y = match align {
                    NodeGraphToolbarAlign::Start => y_start,
                    NodeGraphToolbarAlign::Center => y_center,
                    NodeGraphToolbarAlign::End => y_end,
                };
                (node.origin.x.0 + node.size.width.0 + gap, y)
            }
        };

        let rect = Rect::new(
            Point::new(Px(x + offset.x.0), Px(y + offset.y.0)),
            Size::new(Px(w), Px(h)),
        );
        clamp_rect_to_bounds(rect, bounds)
    }

    fn resolve_child_size<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        child: fret_core::NodeId,
    ) -> Size {
        match self.size {
            NodeGraphToolbarSize::Fixed(size) => size,
            NodeGraphToolbarSize::Auto => {
                let avail = cx.bounds.size;
                let constraints = LayoutConstraints::new(
                    LayoutSize::new(None, None),
                    LayoutSize::new(
                        AvailableSpace::Definite(avail.width),
                        AvailableSpace::Definite(avail.height),
                    ),
                );
                cx.measure_in(child, constraints)
            }
        }
    }

    fn resolve_target_node<H: UiHost>(&self, host: &H) -> Option<(NodeId, bool)> {
        self.view_state
            .read_ref(host, |s| {
                if let Some(node) = self.node {
                    Some((node, s.selected_nodes.contains(&node)))
                } else {
                    s.selected_nodes.first().copied().map(|id| (id, true))
                }
            })
            .ok()
            .flatten()
    }
}

impl<H: UiHost> Widget<H> for NodeGraphNodeToolbar {
    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        self.last_child_bounds
            .is_some_and(|rect| rect.contains(position))
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.graph, Invalidation::Layout);
        cx.observe_model(&self.view_state, Invalidation::Layout);

        let child = cx.children.get(0).copied();
        self.last_child_bounds = None;

        let Some((node_id, is_selected)) = self.resolve_target_node(&*cx.app) else {
            if let Some(child) = child {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            }
            return cx.bounds.size;
        };

        if self.visibility == NodeGraphToolbarVisibility::WhenSelected && !is_selected {
            if let Some(child) = child {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            }
            return cx.bounds.size;
        }

        let snapshot = self.internals.snapshot();
        let Some(node_rect) = snapshot.nodes_window.get(&node_id).copied() else {
            if let Some(child) = child {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            }
            return cx.bounds.size;
        };

        if let Some(child) = child {
            let size = self.resolve_child_size(cx, child);
            if size.width.0 <= 0.0 && size.height.0 <= 0.0 {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            } else {
                let rect = Self::positioned_rect_for(
                    cx.bounds,
                    node_rect,
                    size,
                    self.position,
                    self.align,
                    self.gap_px,
                    self.offset,
                );
                self.last_child_bounds = Some(rect);
                cx.layout_in(child, rect);
            }
        }

        cx.bounds.size
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            }
        }
    }
}

/// A window-space toolbar anchored to an edge center derived from internals (XyFlow `EdgeToolbar`-style).
///
/// Expected children:
/// - child 0: the toolbar content widget (should implement `Widget::measure` for `Auto` sizing).
pub struct NodeGraphEdgeToolbar {
    canvas_node: fret_core::NodeId,
    graph: Model<crate::Graph>,
    view_state: Model<NodeGraphViewState>,
    internals: Arc<NodeGraphInternalsStore>,

    edge: Option<EdgeId>,
    visibility: NodeGraphToolbarVisibility,
    align_x: NodeGraphToolbarAlign,
    align_y: NodeGraphToolbarAlign,
    size: NodeGraphToolbarSize,
    offset: Point,

    last_child_bounds: Option<Rect>,
}

impl NodeGraphEdgeToolbar {
    pub fn new(
        canvas_node: fret_core::NodeId,
        graph: Model<crate::Graph>,
        view_state: Model<NodeGraphViewState>,
        internals: Arc<NodeGraphInternalsStore>,
    ) -> Self {
        Self {
            canvas_node,
            graph,
            view_state,
            internals,
            edge: None,
            visibility: NodeGraphToolbarVisibility::WhenSelected,
            align_x: NodeGraphToolbarAlign::Center,
            align_y: NodeGraphToolbarAlign::Center,
            size: NodeGraphToolbarSize::Auto,
            offset: Point::new(Px(0.0), Px(0.0)),
            last_child_bounds: None,
        }
    }

    /// Anchors the toolbar to a specific edge id.
    pub fn for_edge(mut self, edge: EdgeId) -> Self {
        self.edge = Some(edge);
        self
    }

    pub fn with_visibility(mut self, visibility: NodeGraphToolbarVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn with_align_x(mut self, align_x: NodeGraphToolbarAlign) -> Self {
        self.align_x = align_x;
        self
    }

    pub fn with_align_y(mut self, align_y: NodeGraphToolbarAlign) -> Self {
        self.align_y = align_y;
        self
    }

    pub fn with_offset_px(mut self, dx: f32, dy: f32) -> Self {
        self.offset = Point::new(Px(dx), Px(dy));
        self
    }

    pub fn with_size(mut self, size: NodeGraphToolbarSize) -> Self {
        self.size = size;
        self
    }

    fn positioned_rect_for(
        bounds: Rect,
        anchor: Point,
        size: Size,
        align_x: NodeGraphToolbarAlign,
        align_y: NodeGraphToolbarAlign,
        offset: Point,
    ) -> Rect {
        let w = size.width.0.max(0.0);
        let h = size.height.0.max(0.0);

        let x = match align_x {
            NodeGraphToolbarAlign::Start => anchor.x.0,
            NodeGraphToolbarAlign::Center => anchor.x.0 - 0.5 * w,
            NodeGraphToolbarAlign::End => anchor.x.0 - w,
        };

        let y = match align_y {
            NodeGraphToolbarAlign::Start => anchor.y.0,
            NodeGraphToolbarAlign::Center => anchor.y.0 - 0.5 * h,
            NodeGraphToolbarAlign::End => anchor.y.0 - h,
        };

        let rect = Rect::new(
            Point::new(Px(x + offset.x.0), Px(y + offset.y.0)),
            Size::new(Px(w), Px(h)),
        );
        clamp_rect_to_bounds(rect, bounds)
    }

    fn resolve_child_size<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        child: fret_core::NodeId,
    ) -> Size {
        match self.size {
            NodeGraphToolbarSize::Fixed(size) => size,
            NodeGraphToolbarSize::Auto => {
                let avail = cx.bounds.size;
                let constraints = LayoutConstraints::new(
                    LayoutSize::new(None, None),
                    LayoutSize::new(
                        AvailableSpace::Definite(avail.width),
                        AvailableSpace::Definite(avail.height),
                    ),
                );
                cx.measure_in(child, constraints)
            }
        }
    }

    fn resolve_target_edge<H: UiHost>(&self, host: &H) -> Option<(EdgeId, bool)> {
        self.view_state
            .read_ref(host, |s| {
                if let Some(edge) = self.edge {
                    Some((edge, s.selected_edges.contains(&edge)))
                } else {
                    s.selected_edges.first().copied().map(|id| (id, true))
                }
            })
            .ok()
            .flatten()
    }
}

impl<H: UiHost> Widget<H> for NodeGraphEdgeToolbar {
    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        self.last_child_bounds
            .is_some_and(|rect| rect.contains(position))
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.observe_model(&self.graph, Invalidation::Layout);
        cx.observe_model(&self.view_state, Invalidation::Layout);

        let child = cx.children.get(0).copied();
        self.last_child_bounds = None;

        let Some((edge_id, is_selected)) = self.resolve_target_edge(&*cx.app) else {
            if let Some(child) = child {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            }
            return cx.bounds.size;
        };

        if self.visibility == NodeGraphToolbarVisibility::WhenSelected && !is_selected {
            if let Some(child) = child {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            }
            return cx.bounds.size;
        }

        let snapshot = self.internals.snapshot();
        let Some(center) = snapshot.edge_centers_window.get(&edge_id).copied() else {
            if let Some(child) = child {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            }
            return cx.bounds.size;
        };

        if let Some(child) = child {
            let size = self.resolve_child_size(cx, child);
            if size.width.0 <= 0.0 && size.height.0 <= 0.0 {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            } else {
                let rect = Self::positioned_rect_for(
                    cx.bounds,
                    center,
                    size,
                    self.align_x,
                    self.align_y,
                    self.offset,
                );
                self.last_child_bounds = Some(rect);
                cx.layout_in(child, rect);
            }
        }

        cx.bounds.size
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            }
        }
    }
}

#[cfg(test)]
mod node_toolbar_tests {
    use super::{
        NodeGraphNodeToolbar, NodeGraphToolbarAlign, NodeGraphToolbarPosition,
        NodeGraphToolbarVisibility,
    };
    use fret_core::{Point, Px, Rect, Size};

    #[test]
    fn positioned_rect_top_center_matches_expected_math() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );
        let node = Rect::new(
            Point::new(Px(50.0), Px(40.0)),
            Size::new(Px(40.0), Px(20.0)),
        );
        let size = Size::new(Px(30.0), Px(10.0));

        let rect = NodeGraphNodeToolbar::positioned_rect_for(
            bounds,
            node,
            size,
            NodeGraphToolbarPosition::Top,
            NodeGraphToolbarAlign::Center,
            8.0,
            Point::new(Px(0.0), Px(0.0)),
        );

        assert_eq!(rect.origin.x.0, 55.0);
        assert_eq!(rect.origin.y.0, 22.0);
        assert_eq!(rect.size.width.0, 30.0);
        assert_eq!(rect.size.height.0, 10.0);
    }

    #[test]
    fn positioned_rect_right_start_matches_expected_math() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );
        let node = Rect::new(
            Point::new(Px(50.0), Px(40.0)),
            Size::new(Px(40.0), Px(20.0)),
        );
        let size = Size::new(Px(30.0), Px(10.0));

        let rect = NodeGraphNodeToolbar::positioned_rect_for(
            bounds,
            node,
            size,
            NodeGraphToolbarPosition::Right,
            NodeGraphToolbarAlign::Start,
            8.0,
            Point::new(Px(0.0), Px(0.0)),
        );

        assert_eq!(rect.origin.x.0, 98.0);
        assert_eq!(rect.origin.y.0, 40.0);
    }

    #[test]
    fn positioned_rect_is_clamped_to_canvas_bounds() {
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(80.0)));
        let node = Rect::new(
            Point::new(Px(85.0), Px(10.0)),
            Size::new(Px(20.0), Px(20.0)),
        );
        let size = Size::new(Px(50.0), Px(10.0));

        let rect = NodeGraphNodeToolbar::positioned_rect_for(
            bounds,
            node,
            size,
            NodeGraphToolbarPosition::Right,
            NodeGraphToolbarAlign::Start,
            8.0,
            Point::new(Px(0.0), Px(0.0)),
        );

        // Desired x would be 85 + 20 + 8 = 113, but must clamp to 100 - 50 = 50.
        assert_eq!(rect.origin.x.0, 50.0);
        assert_eq!(rect.origin.y.0, 10.0);
    }

    #[test]
    fn visibility_default_is_when_selected() {
        assert_eq!(
            NodeGraphToolbarVisibility::default(),
            NodeGraphToolbarVisibility::WhenSelected
        );
    }
}

#[cfg(test)]
mod edge_toolbar_tests {
    use super::{NodeGraphEdgeToolbar, NodeGraphToolbarAlign};
    use fret_core::{Point, Px, Rect, Size};

    #[test]
    fn positioned_rect_center_center_centers_about_anchor() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );
        let anchor = Point::new(Px(50.0), Px(60.0));
        let size = Size::new(Px(20.0), Px(10.0));

        let rect = NodeGraphEdgeToolbar::positioned_rect_for(
            bounds,
            anchor,
            size,
            NodeGraphToolbarAlign::Center,
            NodeGraphToolbarAlign::Center,
            Point::new(Px(0.0), Px(0.0)),
        );

        assert_eq!(rect.origin.x.0, 40.0);
        assert_eq!(rect.origin.y.0, 55.0);
    }

    #[test]
    fn positioned_rect_is_clamped_to_bounds() {
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(80.0)));
        let anchor = Point::new(Px(95.0), Px(10.0));
        let size = Size::new(Px(50.0), Px(10.0));

        let rect = NodeGraphEdgeToolbar::positioned_rect_for(
            bounds,
            anchor,
            size,
            NodeGraphToolbarAlign::Start,
            NodeGraphToolbarAlign::Start,
            Point::new(Px(0.0), Px(0.0)),
        );

        assert_eq!(rect.origin.x.0, 50.0);
        assert_eq!(rect.origin.y.0, 10.0);
    }
}
