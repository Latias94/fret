use super::super::super::*;

#[test]
fn port_connectable_helpers_respect_node_and_port_overrides() {
    let (graph, from, _candidate_same_dir, _candidate_other_dir, node_default_port) =
        super::test_support::sample_graph();
    let interaction = NodeGraphInteractionState::default();

    assert!(
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::port_is_connectable_base(
            &graph,
            &interaction,
            from,
        )
    );
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::port_is_connectable_start(
            &graph,
            &interaction,
            from,
        )
    );
    assert!(
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::port_is_connectable_end(
            &graph,
            &interaction,
            from,
        )
    );

    assert!(
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::port_is_connectable_base(
            &graph,
            &interaction,
            node_default_port,
        )
    );
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::port_is_connectable_end(
            &graph,
            &interaction,
            node_default_port,
        )
    );
}

#[test]
fn should_add_bundle_port_requires_unique_same_direction_candidate() {
    let (graph, from, candidate_same_dir, candidate_other_dir, _node_default_port) =
        super::test_support::sample_graph();

    assert!(
        NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::should_add_bundle_port(
            &graph,
            from,
            &[],
            candidate_same_dir,
        )
    );
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::should_add_bundle_port(
            &graph,
            from,
            &[],
            from,
        )
    );
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::should_add_bundle_port(
            &graph,
            from,
            &[candidate_same_dir],
            candidate_same_dir,
        )
    );
    assert!(
        !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::should_add_bundle_port(
            &graph,
            from,
            &[],
            candidate_other_dir,
        )
    );
}
