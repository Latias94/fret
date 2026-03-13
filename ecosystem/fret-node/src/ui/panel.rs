//! Panel composition primitives for node graph editor shells (UI-only).
//!
//! This is the Fret equivalent of XyFlow's `<Panel />`: an overlay surface that is positioned in
//! window space (outside the canvas render transform) and participates in hit-testing only within
//! its child bounds (events fall through to the canvas elsewhere).

use fret_core::{Point, Px, Rect, Size};
use fret_ui::layout_constraints::{AvailableSpace, LayoutConstraints, LayoutSize};
use fret_ui::{UiHost, retained_bridge::*};

use crate::ui::screen_space_placement::{AxisAlign, rect_in_bounds};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeGraphPanelPosition {
    TopLeft,
    TopCenter,
    TopRight,
    LeftCenter,
    Center,
    RightCenter,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeGraphPanelSize {
    /// Measure the child (uses `Widget::measure`).
    ///
    /// If the measured size is zero (default for widgets that don't implement `measure`), falls
    /// back to `Fill`.
    Auto,
    /// Fill all available space inside the panel.
    Fill,
    /// Fixed size in window-space logical pixels.
    Fixed(Size),
}

fn panel_position_alignments(position: NodeGraphPanelPosition) -> (AxisAlign, AxisAlign) {
    match position {
        NodeGraphPanelPosition::TopLeft => (AxisAlign::Start, AxisAlign::Start),
        NodeGraphPanelPosition::TopCenter => (AxisAlign::Center, AxisAlign::Start),
        NodeGraphPanelPosition::TopRight => (AxisAlign::End, AxisAlign::Start),
        NodeGraphPanelPosition::LeftCenter => (AxisAlign::Start, AxisAlign::Center),
        NodeGraphPanelPosition::Center => (AxisAlign::Center, AxisAlign::Center),
        NodeGraphPanelPosition::RightCenter => (AxisAlign::End, AxisAlign::Center),
        NodeGraphPanelPosition::BottomLeft => (AxisAlign::Start, AxisAlign::End),
        NodeGraphPanelPosition::BottomCenter => (AxisAlign::Center, AxisAlign::End),
        NodeGraphPanelPosition::BottomRight => (AxisAlign::End, AxisAlign::End),
    }
}

/// A window-space overlay panel that positions its child within `bounds`.
///
/// Expected children:
/// - child 0: the panel content widget.
pub struct NodeGraphPanel {
    position: NodeGraphPanelPosition,
    size: NodeGraphPanelSize,
    margin_px: f32,
    offset: Point,
    last_child_bounds: Option<Rect>,
}

impl NodeGraphPanel {
    pub fn new(position: NodeGraphPanelPosition) -> Self {
        Self {
            position,
            size: NodeGraphPanelSize::Auto,
            margin_px: 0.0,
            offset: Point::new(Px(0.0), Px(0.0)),
            last_child_bounds: None,
        }
    }

    pub fn with_size(mut self, size: NodeGraphPanelSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_margin_px(mut self, margin_px: f32) -> Self {
        self.margin_px = margin_px;
        self
    }

    pub fn with_offset_px(mut self, dx: f32, dy: f32) -> Self {
        self.offset = Point::new(Px(dx), Px(dy));
        self
    }

    fn positioned_rect(&self, bounds: Rect, size: Size) -> Rect {
        let (align_x, align_y) = panel_position_alignments(self.position);
        rect_in_bounds(bounds, size, align_x, align_y, self.margin_px, self.offset)
    }

    fn resolve_child_size<H: UiHost>(
        &mut self,
        cx: &mut LayoutCx<'_, H>,
        child: fret_core::NodeId,
    ) -> Size {
        let available = cx.bounds.size;
        match self.size {
            NodeGraphPanelSize::Fill => available,
            NodeGraphPanelSize::Fixed(size) => size,
            NodeGraphPanelSize::Auto => {
                let margin = self.margin_px.max(0.0);
                let avail_w = (available.width.0 - 2.0 * margin).max(0.0);
                let avail_h = (available.height.0 - 2.0 * margin).max(0.0);
                let constraints = LayoutConstraints::new(
                    LayoutSize::new(None, None),
                    LayoutSize::new(
                        AvailableSpace::Definite(Px(avail_w)),
                        AvailableSpace::Definite(Px(avail_h)),
                    ),
                );
                let measured = cx.measure_in(child, constraints);
                if measured.width.0 > 0.0 || measured.height.0 > 0.0 {
                    measured
                } else {
                    available
                }
            }
        }
    }
}

impl Default for NodeGraphPanel {
    fn default() -> Self {
        Self::new(NodeGraphPanelPosition::TopLeft)
    }
}

impl<H: UiHost> Widget<H> for NodeGraphPanel {
    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        self.last_child_bounds
            .is_some_and(|rect| rect.contains(position))
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let child = cx.children.get(0).copied();
        self.last_child_bounds = None;

        if let Some(child) = child {
            let size = self.resolve_child_size(cx, child);
            let rect = self.positioned_rect(cx.bounds, size);
            self.last_child_bounds = Some(rect);
            cx.layout_in(child, rect);
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
mod tests {
    use super::{NodeGraphPanel, NodeGraphPanelPosition};
    use fret_core::{Point, Px, Rect, Size};

    #[test]
    fn positioned_rect_top_right_respects_margin() {
        let panel = NodeGraphPanel::new(NodeGraphPanelPosition::TopRight).with_margin_px(10.0);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );
        let rect = panel.positioned_rect(bounds, Size::new(Px(20.0), Px(10.0)));
        assert_eq!(rect.origin.x.0, 170.0);
        assert_eq!(rect.origin.y.0, 10.0);
    }
}
