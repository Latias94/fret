//! Node graph toolbars (UI-only).

use std::sync::Arc;

use fret_core::{Point, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
use fret_ui::{UiHost, retained_bridge::*};

use crate::core::{EdgeId, NodeId};
use crate::io::NodeGraphViewState;
use crate::ui::NodeGraphInternalsStore;
use crate::ui::screen_space_placement::{rect_adjacent_to_rect, rect_anchored_at_point};

use super::layout_hidden_child_and_release_focus;
use super::toolbar_policy::{
    NodeGraphToolbarAlign, NodeGraphToolbarPosition, NodeGraphToolbarSize,
    NodeGraphToolbarVisibility, resolve_edge_toolbar_target, resolve_node_toolbar_target,
    toolbar_align_axis, toolbar_position_to_adjacent, toolbar_visible,
};

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
        rect_adjacent_to_rect(
            bounds,
            node,
            size,
            toolbar_position_to_adjacent(position),
            toolbar_align_axis(align),
            gap_px,
            offset,
        )
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

        let Some((node_id, is_selected)) =
            resolve_node_toolbar_target(&self.view_state, self.node, &*cx.app)
        else {
            if let Some(child) = child {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            }
            return cx.bounds.size;
        };

        if !toolbar_visible(self.visibility, is_selected) {
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
        rect_anchored_at_point(
            bounds,
            anchor,
            size,
            toolbar_align_axis(align_x),
            toolbar_align_axis(align_y),
            offset,
        )
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

        let Some((edge_id, is_selected)) =
            resolve_edge_toolbar_target(&self.view_state, self.edge, &*cx.app)
        else {
            if let Some(child) = child {
                layout_hidden_child_and_release_focus(cx, child, self.canvas_node);
            }
            return cx.bounds.size;
        };

        if !toolbar_visible(self.visibility, is_selected) {
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
    use super::{NodeGraphNodeToolbar, NodeGraphToolbarAlign, NodeGraphToolbarPosition};
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
