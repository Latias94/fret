use fret_core::{Axis, NodeId, Px, Rect, Size};

use crate::widget::{LayoutCx, Widget};
use crate::{UiHost, UiTree};

/// A minimal retained split container that lays out exactly two children along an axis.
///
/// This is intentionally "dumb":
/// - No draggable resize handle.
/// - No gap/padding.
/// - The parent controls the ratio and child order.
///
/// It exists primarily as a convenience for demos and low-level retained trees.
#[derive(Debug, Clone, Copy)]
pub struct FixedSplit {
    axis: Axis,
    /// Fraction of the available space assigned to the first child.
    ///
    /// Values are clamped to `[0.0, 1.0]`.
    first_fraction: f32,
}

impl FixedSplit {
    pub fn new(axis: Axis, first_fraction: f32) -> Self {
        Self {
            axis,
            first_fraction,
        }
    }

    pub fn vertical(first_fraction: f32) -> Self {
        Self::new(Axis::Vertical, first_fraction)
    }

    pub fn horizontal(first_fraction: f32) -> Self {
        Self::new(Axis::Horizontal, first_fraction)
    }

    /// Create a split node without attaching children.
    pub fn create_node<H: UiHost>(ui: &mut UiTree<H>, split: FixedSplit) -> NodeId {
        ui.create_node(split)
    }

    /// Create a split node and attach two children in order.
    pub fn create_node_with_children<H: UiHost>(
        ui: &mut UiTree<H>,
        split: FixedSplit,
        first: NodeId,
        second: NodeId,
    ) -> NodeId {
        let node = Self::create_node(ui, split);
        ui.add_child(node, first);
        ui.add_child(node, second);
        node
    }
}

impl<H: UiHost> Widget<H> for FixedSplit {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        if cx.children.is_empty() {
            return cx.available;
        }

        let first_fraction = self.first_fraction.clamp(0.0, 1.0);
        let origin = cx.bounds.origin;

        match self.axis {
            Axis::Vertical => {
                let top_h = Px((cx.available.height.0 * first_fraction).max(0.0));
                let bottom_h = Px((cx.available.height.0 - top_h.0).max(0.0));

                if let Some(&top) = cx.children.first() {
                    let rect = Rect::new(origin, Size::new(cx.available.width, top_h));
                    let _ = cx.layout_in(top, rect);
                }
                if let Some(&bottom) = cx.children.get(1) {
                    let rect = Rect::new(
                        fret_core::Point::new(origin.x, Px(origin.y.0 + top_h.0)),
                        Size::new(cx.available.width, bottom_h),
                    );
                    let _ = cx.layout_in(bottom, rect);
                }
            }
            Axis::Horizontal => {
                let left_w = Px((cx.available.width.0 * first_fraction).max(0.0));
                let right_w = Px((cx.available.width.0 - left_w.0).max(0.0));

                if let Some(&left) = cx.children.first() {
                    let rect = Rect::new(origin, Size::new(left_w, cx.available.height));
                    let _ = cx.layout_in(left, rect);
                }
                if let Some(&right) = cx.children.get(1) {
                    let rect = Rect::new(
                        fret_core::Point::new(Px(origin.x.0 + left_w.0), origin.y),
                        Size::new(right_w, cx.available.height),
                    );
                    let _ = cx.layout_in(right, rect);
                }
            }
        }

        cx.available
    }
}
