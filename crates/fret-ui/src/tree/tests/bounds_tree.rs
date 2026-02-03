use super::*;

#[test]
fn bounds_tree_supports_overflow_visible_ancestors() {
    // The bounds tree is a hit-test acceleration structure. It must remain correct even when an
    // ancestor does not clip hit-testing (overflow: visible semantics), so that children outside
    // the ancestor bounds can still be targeted.

    let mut trees = super::super::bounds_tree::HitTestBoundsTrees::default();
    trees.begin_frame(FrameId(1));

    let mut nodes: SlotMap<NodeId, Node<crate::test_host::TestHost>> = SlotMap::with_key();
    let layer_root = nodes.insert(Node::new(EmptyWidget));

    let mut child_nodes: Vec<NodeId> = Vec::new();
    child_nodes.reserve(255);
    for _ in 0..255 {
        let id = nodes.insert(Node::new(EmptyWidget));
        nodes.get_mut(id).unwrap().parent = Some(layer_root);
        child_nodes.push(id);
    }

    let root_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    let mut records: Vec<super::super::prepaint::InteractionRecord> = Vec::new();
    records.reserve(256);

    records.push(super::super::prepaint::InteractionRecord {
        node: layer_root,
        bounds: root_bounds,
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: false,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: false,
    });

    // Fill with children whose bounds are entirely inside the root.
    for (idx, child) in child_nodes.iter().copied().enumerate().take(254) {
        let x = (idx as f32) % 50.0;
        let y = ((idx as f32) / 50.0).floor();
        let bounds = Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(1.0), Px(1.0)));
        records.push(super::super::prepaint::InteractionRecord {
            node: child,
            bounds,
            render_transform_inv: None,
            children_render_transform_inv: None,
            clips_hit_test: true,
            clip_hit_test_corner_radii: None,
            is_focusable: false,
            focus_traversal_children: true,
            can_scroll_descendant_into_view: false,
        });
    }

    // Place one child outside the root bounds. With overflow-visible ancestry, this must still be
    // indexable and queryable.
    let outside_child = *child_nodes.last().unwrap();
    let outside_bounds = Rect::new(
        Point::new(Px(120.0), Px(10.0)),
        Size::new(Px(10.0), Px(10.0)),
    );
    records.push(super::super::prepaint::InteractionRecord {
        node: outside_child,
        bounds: outside_bounds,
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: true,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: false,
    });

    trees.rebuild_for_layer_from_records(layer_root, &records, &nodes);

    let query = trees.query(layer_root, Point::new(Px(125.0), Px(15.0)));
    assert_eq!(
        query,
        super::super::bounds_tree::HitTestBoundsTreeQuery::Hit(outside_child)
    );
}

#[derive(Debug, Default, Clone, Copy)]
struct EmptyWidget;

impl<H: UiHost> Widget<H> for EmptyWidget {}
