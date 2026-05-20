use fret_core::{Point, Rect, Size};

use crate::ui::screen_space_placement::{rect_adjacent_to_rect, rect_anchored_at_point};

use super::toolbar_policy::{
    NodeGraphToolbarAlign, NodeGraphToolbarPosition, NodeGraphToolbarVisibility,
    toolbar_align_axis, toolbar_position_to_adjacent, toolbar_visible,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum ToolbarChildLayoutPlan {
    Hidden,
    Layout { rect: Rect },
}

impl ToolbarChildLayoutPlan {
    pub(super) fn rect(self) -> Option<Rect> {
        match self {
            Self::Hidden => None,
            Self::Layout { rect } => Some(rect),
        }
    }
}

pub(super) fn visible_toolbar_anchor<Anchor: Copy>(
    target: Option<(Anchor, bool)>,
    visibility: NodeGraphToolbarVisibility,
) -> Option<Anchor> {
    let (anchor, selected) = target?;
    toolbar_visible(visibility, selected).then_some(anchor)
}

pub(super) fn plan_node_toolbar_child_layout(
    bounds: Rect,
    target_rect: Option<Rect>,
    child_size: Size,
    position: NodeGraphToolbarPosition,
    align: NodeGraphToolbarAlign,
    gap_px: f32,
    offset: Point,
) -> ToolbarChildLayoutPlan {
    let Some(target_rect) = target_rect else {
        return ToolbarChildLayoutPlan::Hidden;
    };
    if toolbar_size_is_empty(child_size) {
        return ToolbarChildLayoutPlan::Hidden;
    }

    ToolbarChildLayoutPlan::Layout {
        rect: rect_adjacent_to_rect(
            bounds,
            target_rect,
            child_size,
            toolbar_position_to_adjacent(position),
            toolbar_align_axis(align),
            gap_px,
            offset,
        ),
    }
}

pub(super) fn plan_edge_toolbar_child_layout(
    bounds: Rect,
    target_center: Option<Point>,
    child_size: Size,
    align_x: NodeGraphToolbarAlign,
    align_y: NodeGraphToolbarAlign,
    offset: Point,
) -> ToolbarChildLayoutPlan {
    let Some(target_center) = target_center else {
        return ToolbarChildLayoutPlan::Hidden;
    };
    if toolbar_size_is_empty(child_size) {
        return ToolbarChildLayoutPlan::Hidden;
    }

    ToolbarChildLayoutPlan::Layout {
        rect: rect_anchored_at_point(
            bounds,
            target_center,
            child_size,
            toolbar_align_axis(align_x),
            toolbar_align_axis(align_y),
            offset,
        ),
    }
}

pub(super) fn toolbar_child_hit_test(child_bounds: Option<Rect>, position: Point) -> bool {
    child_bounds.is_some_and(|rect| rect.contains(position))
}

fn toolbar_size_is_empty(size: Size) -> bool {
    size.width.0 <= 0.0 && size.height.0 <= 0.0
}

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px, Rect, Size};

    use crate::ui::overlays::toolbar_layout_policy::{
        ToolbarChildLayoutPlan, plan_edge_toolbar_child_layout, plan_node_toolbar_child_layout,
        toolbar_child_hit_test, visible_toolbar_anchor,
    };
    use crate::ui::overlays::toolbar_policy::{
        NodeGraphToolbarAlign, NodeGraphToolbarPosition, NodeGraphToolbarVisibility,
    };

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        )
    }

    fn target_rect() -> Rect {
        Rect::new(
            Point::new(Px(50.0), Px(40.0)),
            Size::new(Px(40.0), Px(20.0)),
        )
    }

    #[test]
    fn toolbar_visible_anchor_honors_selection_visibility() {
        assert_eq!(
            visible_toolbar_anchor(
                Some((target_rect(), false)),
                NodeGraphToolbarVisibility::Always
            ),
            Some(target_rect())
        );
        assert_eq!(
            visible_toolbar_anchor(
                Some((target_rect(), false)),
                NodeGraphToolbarVisibility::WhenSelected,
            ),
            None
        );
        assert_eq!(
            visible_toolbar_anchor::<Rect>(None, NodeGraphToolbarVisibility::Always),
            None
        );
    }

    #[test]
    fn node_toolbar_child_layout_preserves_positioning_and_clamping() {
        let plan = plan_node_toolbar_child_layout(
            bounds(),
            Some(target_rect()),
            Size::new(Px(30.0), Px(10.0)),
            NodeGraphToolbarPosition::Top,
            NodeGraphToolbarAlign::Center,
            8.0,
            Point::new(Px(0.0), Px(0.0)),
        );

        assert_eq!(
            plan,
            ToolbarChildLayoutPlan::Layout {
                rect: Rect::new(
                    Point::new(Px(55.0), Px(22.0)),
                    Size::new(Px(30.0), Px(10.0))
                ),
            }
        );

        let clamped = plan_node_toolbar_child_layout(
            Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(80.0))),
            Some(Rect::new(
                Point::new(Px(85.0), Px(10.0)),
                Size::new(Px(20.0), Px(20.0)),
            )),
            Size::new(Px(50.0), Px(10.0)),
            NodeGraphToolbarPosition::Right,
            NodeGraphToolbarAlign::Start,
            8.0,
            Point::new(Px(0.0), Px(0.0)),
        );

        assert_eq!(
            clamped.rect(),
            Some(Rect::new(
                Point::new(Px(50.0), Px(10.0)),
                Size::new(Px(50.0), Px(10.0)),
            ))
        );
    }

    #[test]
    fn edge_toolbar_child_layout_preserves_anchor_alignment() {
        let plan = plan_edge_toolbar_child_layout(
            bounds(),
            Some(Point::new(Px(50.0), Px(60.0))),
            Size::new(Px(20.0), Px(10.0)),
            NodeGraphToolbarAlign::Center,
            NodeGraphToolbarAlign::Center,
            Point::new(Px(0.0), Px(0.0)),
        );

        assert_eq!(
            plan.rect(),
            Some(Rect::new(
                Point::new(Px(40.0), Px(55.0)),
                Size::new(Px(20.0), Px(10.0)),
            ))
        );
    }

    #[test]
    fn toolbar_layout_hides_missing_or_empty_targets_and_hit_tests_child_bounds() {
        assert_eq!(
            plan_node_toolbar_child_layout(
                bounds(),
                None,
                Size::new(Px(30.0), Px(10.0)),
                NodeGraphToolbarPosition::Top,
                NodeGraphToolbarAlign::Center,
                8.0,
                Point::new(Px(0.0), Px(0.0)),
            ),
            ToolbarChildLayoutPlan::Hidden
        );
        assert_eq!(
            plan_edge_toolbar_child_layout(
                bounds(),
                Some(Point::new(Px(50.0), Px(60.0))),
                Size::new(Px(0.0), Px(0.0)),
                NodeGraphToolbarAlign::Center,
                NodeGraphToolbarAlign::Center,
                Point::new(Px(0.0), Px(0.0)),
            ),
            ToolbarChildLayoutPlan::Hidden
        );

        let child_bounds = Rect::new(
            Point::new(Px(40.0), Px(55.0)),
            Size::new(Px(20.0), Px(10.0)),
        );
        assert!(toolbar_child_hit_test(
            Some(child_bounds),
            Point::new(Px(50.0), Px(60.0)),
        ));
        assert!(!toolbar_child_hit_test(
            Some(child_bounds),
            Point::new(Px(10.0), Px(10.0)),
        ));
        assert!(!toolbar_child_hit_test(
            None,
            Point::new(Px(50.0), Px(60.0)),
        ));
    }
}
