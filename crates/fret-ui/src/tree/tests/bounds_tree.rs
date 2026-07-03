use super::*;

#[test]
fn bounds_tree_supports_overflow_visible_ancestors() {
    // The bounds tree is a hit-test acceleration structure. It must remain correct even when an
    // ancestor does not clip hit-testing (overflow: visible semantics), so that children outside
    // the ancestor bounds can still be targeted.

    let mut trees = super::super::bounds_tree::HitTestBoundsTrees::default();
    let mut layer = super::super::bounds_tree::BoundaryHitTestBoundsState::default();
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
        parent: None,
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
            parent: Some(layer_root),
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
        parent: Some(layer_root),
        bounds: outside_bounds,
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: true,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: false,
    });

    trees.rebuild_for_layer_from_records(layer_root, &records, &mut layer);

    let (query, _stats) = trees.query(Some(&layer), Point::new(Px(125.0), Px(15.0)), false);
    assert_eq!(
        query,
        super::super::bounds_tree::HitTestBoundsTreeQuery::Hit(outside_child)
    );
}

#[test]
fn bounds_tree_clip_stack_uses_recorded_parent_under_stale_parent_pointers() {
    let mut trees = super::super::bounds_tree::HitTestBoundsTrees::default();
    let mut layer = super::super::bounds_tree::BoundaryHitTestBoundsState::default();
    trees.begin_frame(FrameId(1));

    let mut nodes: SlotMap<NodeId, Node<crate::test_host::TestHost>> = SlotMap::with_key();
    let layer_root = nodes.insert(Node::new(EmptyWidget));

    let mut child_nodes: Vec<NodeId> = Vec::new();
    child_nodes.reserve(255);
    for _ in 0..255 {
        let id = nodes.insert(Node::new(EmptyWidget));
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
        parent: None,
        bounds: root_bounds,
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: true,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: false,
    });

    for (idx, child) in child_nodes.iter().copied().enumerate().take(254) {
        let x = (idx as f32) % 50.0;
        let y = ((idx as f32) / 50.0).floor();
        let bounds = Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(1.0), Px(1.0)));
        records.push(super::super::prepaint::InteractionRecord {
            node: child,
            parent: Some(layer_root),
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

    let outside_child = *child_nodes.last().unwrap();
    records.push(super::super::prepaint::InteractionRecord {
        node: outside_child,
        parent: Some(layer_root),
        bounds: Rect::new(
            Point::new(Px(120.0), Px(10.0)),
            Size::new(Px(10.0), Px(10.0)),
        ),
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: true,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: false,
    });

    trees.rebuild_for_layer_from_records(layer_root, &records, &mut layer);

    let (query, _stats) = trees.query(Some(&layer), Point::new(Px(125.0), Px(15.0)), false);
    assert_eq!(
        query,
        super::super::bounds_tree::HitTestBoundsTreeQuery::Miss,
        "clip stack must use recorded prepaint parents, not stale retained parent pointers"
    );
}

#[test]
fn boundary_owned_bounds_tree_reuses_index_on_stable_frame() {
    let (mut trees, mut layer, layer_root, outside_child, records) = large_bounds_tree_fixture();

    trees.begin_frame(FrameId(1));
    trees.rebuild_for_layer_from_records(layer_root, &records, &mut layer);
    let (first_query, _stats) = trees.query(Some(&layer), Point::new(Px(125.0), Px(15.0)), false);
    assert_eq!(
        first_query,
        super::super::bounds_tree::HitTestBoundsTreeQuery::Hit(outside_child)
    );

    trees.begin_frame(FrameId(2));
    layer.mark_unused();
    trees.reuse_for_layer(&mut layer);
    let (stable_query, _stats) = trees.query(Some(&layer), Point::new(Px(125.0), Px(15.0)), false);
    assert_eq!(
        stable_query,
        super::super::bounds_tree::HitTestBoundsTreeQuery::Hit(outside_child)
    );
}

#[test]
fn boundary_owned_bounds_tree_disables_stale_index_when_rebuild_records_are_too_small() {
    let (mut trees, mut layer, layer_root, outside_child, records) = large_bounds_tree_fixture();

    trees.begin_frame(FrameId(1));
    trees.rebuild_for_layer_from_records(layer_root, &records, &mut layer);
    let (first_query, _stats) = trees.query(Some(&layer), Point::new(Px(125.0), Px(15.0)), false);
    assert_eq!(
        first_query,
        super::super::bounds_tree::HitTestBoundsTreeQuery::Hit(outside_child)
    );

    trees.begin_frame(FrameId(2));
    layer.mark_unused();
    trees.rebuild_for_layer_from_records(layer_root, &records[..1], &mut layer);
    let (stale_query, _stats) = trees.query(Some(&layer), Point::new(Px(125.0), Px(15.0)), false);
    assert_eq!(
        stale_query,
        super::super::bounds_tree::HitTestBoundsTreeQuery::Disabled
    );
}

fn large_bounds_tree_fixture() -> (
    super::super::bounds_tree::HitTestBoundsTrees,
    super::super::bounds_tree::BoundaryHitTestBoundsState,
    NodeId,
    NodeId,
    Vec<super::super::prepaint::InteractionRecord>,
) {
    let trees = super::super::bounds_tree::HitTestBoundsTrees::default();
    let layer = super::super::bounds_tree::BoundaryHitTestBoundsState::default();
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
        parent: None,
        bounds: root_bounds,
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: false,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: false,
    });

    for (idx, child) in child_nodes.iter().copied().enumerate().take(254) {
        let x = (idx as f32) % 50.0;
        let y = ((idx as f32) / 50.0).floor();
        let bounds = Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(1.0), Px(1.0)));
        records.push(super::super::prepaint::InteractionRecord {
            node: child,
            parent: Some(layer_root),
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

    let outside_child = *child_nodes.last().unwrap();
    let outside_bounds = Rect::new(
        Point::new(Px(120.0), Px(10.0)),
        Size::new(Px(10.0), Px(10.0)),
    );
    records.push(super::super::prepaint::InteractionRecord {
        node: outside_child,
        parent: Some(layer_root),
        bounds: outside_bounds,
        render_transform_inv: None,
        children_render_transform_inv: None,
        clips_hit_test: true,
        clip_hit_test_corner_radii: None,
        is_focusable: false,
        focus_traversal_children: true,
        can_scroll_descendant_into_view: false,
    });

    (trees, layer, layer_root, outside_child, records)
}

#[derive(Debug, Default, Clone, Copy)]
struct EmptyWidget;

impl<H: UiHost> Widget<H> for EmptyWidget {}
