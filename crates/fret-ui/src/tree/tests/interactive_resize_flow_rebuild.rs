use super::*;

fn clear_all_invalidations(ui: &mut UiTree<crate::test_host::TestHost>) {
    for node in ui.nodes.values_mut() {
        node.invalidation = InvalidationFlags::default();
    }
    ui.layout_invalidations_count = 0;
    ui.invalidated_layout_nodes = 0;
    ui.invalidated_paint_nodes = 0;
    ui.invalidated_hit_test_nodes = 0;
}

fn assert_resize_settles_after_quiet_layout_all(
    ui: &mut UiTree<crate::test_host::TestHost>,
    app: &mut crate::test_host::TestHost,
    services: &mut FakeUiServices,
    bounds: Rect,
    scale_factor: f32,
    context: &str,
) {
    let settle_frames = interactive_resize_stable_frames_required();
    assert!(
        settle_frames > 0,
        "{context}: test expects a positive interactive resize settle window"
    );

    for quiet_frame in 1..settle_frames {
        app.advance_frame();
        ui.layout_all(app, services, bounds, scale_factor);
        assert!(
            ui.interactive_resize_active(),
            "{context}: quiet frame {quiet_frame} should still count as interactive resize"
        );
    }

    app.advance_frame();
    ui.layout_all(app, services, bounds, scale_factor);
    assert!(
        !ui.interactive_resize_active(),
        "{context}: resize should settle after {settle_frames} quiet frames"
    );
}

fn render_resize_sensitive_root(
    ui: &mut UiTree<crate::test_host::TestHost>,
    app: &mut crate::test_host::TestHost,
    services: &mut FakeUiServices,
    window: AppWindowId,
    bounds: Rect,
    roomy: bool,
) -> NodeId {
    declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "interactive-resize-flow-rebuild",
        |cx| {
            let mut page = crate::element::FlexProps::default();
            page.layout.size.width = crate::element::Length::Fill;
            page.layout.size.height = crate::element::Length::Fill;
            page.direction = fret_core::Axis::Vertical;
            page.align = crate::element::CrossAlign::Center;
            page.justify = if roomy {
                crate::element::MainAlign::Center
            } else {
                crate::element::MainAlign::Start
            };

            let mut card = crate::element::ContainerProps::default();
            card.layout.size.width = crate::element::Length::Fill;
            card.layout.size.max_width = Some(crate::element::Length::Px(Px(120.0)));
            if !roomy {
                card.layout.size.min_height = Some(crate::element::Length::Px(Px(120.0)));
                card.layout.size.max_height = Some(crate::element::Length::Px(Px(120.0)));
            }

            let mut body = crate::element::FlexProps::default();
            body.layout.size.width = crate::element::Length::Fill;
            body.direction = fret_core::Axis::Vertical;

            vec![cx.flex(page, |cx| {
                vec![cx.container(card, |cx| {
                    vec![cx.flex(body, |cx| vec![cx.text("header"), cx.text("footer")])]
                })]
            })]
        },
    )
}

fn assert_authoritative_compact_flow(
    ui: &mut UiTree<crate::test_host::TestHost>,
    app: &mut crate::test_host::TestHost,
    window: AppWindowId,
    root: NodeId,
    context: &str,
) {
    let page_node = ui.children(root)[0];
    let card_node = ui.children(page_node)[0];
    let page_instance = crate::declarative::frame::element_record_for_node(app, window, page_node)
        .map(|r| r.instance)
        .expect("page instance for compact flow");
    match page_instance {
        crate::declarative::frame::ElementInstance::Flex(props) => {
            assert_eq!(
                props.justify,
                crate::element::MainAlign::Start,
                "{context}: compact flow should author justify-start"
            );
        }
        other => panic!("{context}: expected page node to remain a Flex, got {other:?}"),
    }

    let card_bounds = ui
        .debug_node_bounds(card_node)
        .expect("card bounds for compact flow");
    let engine = ui.take_layout_engine();
    let page_style = engine
        .debug_style_for_node(page_node)
        .cloned()
        .expect("page style for compact flow");
    let card_style = engine
        .debug_style_for_node(card_node)
        .cloned()
        .expect("card style for compact flow");
    ui.put_layout_engine(engine);

    assert_eq!(
        page_style.justify_content,
        Some(taffy::style::JustifyContent::FlexStart),
        "{context}: layout engine should rebuild the compact page style in the same resize frame"
    );
    assert_eq!(
        card_style.min_size.height,
        taffy::style::Dimension::length(120.0),
        "{context}: compact flow should forward min-height constraints immediately"
    );
    assert!(
        card_bounds.origin.y.0 <= 0.5,
        "{context}: compact flow should pin the card to the top immediately; card_bounds={card_bounds:?}"
    );
}

struct DynamicViewportRoot {
    child: NodeId,
    viewport: std::sync::Arc<std::sync::Mutex<Rect>>,
}

impl<H: UiHost> Widget<H> for DynamicViewportRoot {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let viewport = *self.viewport.lock().expect("viewport lock");
        let _ = cx.layout_viewport_root(self.child, viewport);
        cx.available
    }
}

#[test]
fn interactive_resize_cached_flow_rebuilds_authoritatively_when_descendants_turn_layout_dirty() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let roomy_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(680.0), Px(760.0)),
    );
    let compact_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(560.0)),
    );
    let mut services = FakeUiServices;

    let roomy_root =
        render_resize_sensitive_root(&mut ui, &mut app, &mut services, window, roomy_bounds, true);
    ui.set_root(roomy_root);
    ui.layout_all(&mut app, &mut services, roomy_bounds, 1.0);

    let page_node = ui.children(roomy_root)[0];
    let engine = ui.take_layout_engine();
    let page_style = engine
        .debug_style_for_node(page_node)
        .cloned()
        .expect("page style after roomy layout");
    ui.put_layout_engine(engine);
    assert_eq!(
        page_style.justify_content,
        Some(taffy::style::JustifyContent::Center)
    );

    app.advance_frame();

    let compact_root = render_resize_sensitive_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        compact_bounds,
        false,
    );
    assert_eq!(compact_root, roomy_root, "expected stable root identity");
    ui.set_root(roomy_root);
    clear_all_invalidations(&mut ui);
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);

    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "layout-dirty descendant changes should not defer the flow rebuild until resize settles"
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        roomy_root,
        "cached-flow resize frame",
    );

    assert_resize_settles_after_quiet_layout_all(
        &mut ui,
        &mut app,
        &mut services,
        compact_bounds,
        1.0,
        "cached-flow resize frame",
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        roomy_root,
        "settled frame after cached-flow resize",
    );
    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "authoritative same-frame rebuild should not leave a deferred rebuild armed"
    );
}

#[test]
fn interactive_resize_viewport_root_rebuilds_authoritatively_when_descendants_turn_layout_dirty() {
    use std::sync::{Arc, Mutex};

    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let roomy_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(680.0), Px(760.0)),
    );
    let compact_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(560.0)),
    );
    let viewport = Arc::new(Mutex::new(roomy_bounds));
    let mut services = FakeUiServices;

    let viewport_root =
        render_resize_sensitive_root(&mut ui, &mut app, &mut services, window, roomy_bounds, true);
    let root = ui.create_node(DynamicViewportRoot {
        child: viewport_root,
        viewport: viewport.clone(),
    });
    ui.set_children(root, vec![viewport_root]);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, roomy_bounds, 1.0);

    let page_node = ui.children(viewport_root)[0];
    let engine = ui.take_layout_engine();
    let page_style = engine
        .debug_style_for_node(page_node)
        .cloned()
        .expect("page style after roomy viewport layout");
    ui.put_layout_engine(engine);
    assert_eq!(
        page_style.justify_content,
        Some(taffy::style::JustifyContent::Center)
    );

    app.advance_frame();
    *viewport.lock().expect("viewport lock") = compact_bounds;
    let compact_root = render_resize_sensitive_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        compact_bounds,
        false,
    );
    assert_eq!(
        compact_root, viewport_root,
        "expected stable viewport root identity"
    );
    clear_all_invalidations(&mut ui);
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);

    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "viewport-root resize should not defer rebuild when descendant authoring changed"
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        viewport_root,
        "viewport-root resize frame",
    );

    assert_resize_settles_after_quiet_layout_all(
        &mut ui,
        &mut app,
        &mut services,
        compact_bounds,
        1.0,
        "viewport-root resize frame",
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        viewport_root,
        "settled viewport frame",
    );
    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "viewport-root resize should not leave a deferred rebuild armed"
    );
}

#[test]
fn interactive_resize_layout_in_keeps_authoritative_flow_without_deferred_rebuild() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let roomy_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(680.0), Px(760.0)),
    );
    let compact_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(560.0)),
    );
    let mut services = FakeUiServices;

    let roomy_root =
        render_resize_sensitive_root(&mut ui, &mut app, &mut services, window, roomy_bounds, true);
    ui.set_root(roomy_root);
    ui.layout_all(&mut app, &mut services, roomy_bounds, 1.0);

    app.advance_frame();
    let compact_root = render_resize_sensitive_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        compact_bounds,
        false,
    );
    assert_eq!(compact_root, roomy_root, "expected stable root identity");
    ui.set_root(roomy_root);
    clear_all_invalidations(&mut ui);
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);
    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "layout_in path should start from an authoritative compact flow without a deferred rebuild"
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        roomy_root,
        "layout_in compact resize frame",
    );

    let settle_frames = interactive_resize_stable_frames_required();
    for quiet_frame in 1..settle_frames {
        app.advance_frame();
        let steady_size = ui.layout_in(&mut app, &mut services, roomy_root, compact_bounds, 1.0);
        assert_eq!(
            steady_size, compact_bounds.size,
            "quiet layout_in frame {quiet_frame} should preserve the compact root size"
        );
        assert!(
            ui.interactive_resize_active(),
            "quiet layout_in frame {quiet_frame} should still count as interactive resize"
        );
    }

    app.advance_frame();
    let rebuilt_size = ui.layout_in(&mut app, &mut services, roomy_root, compact_bounds, 1.0);
    assert_eq!(
        rebuilt_size, compact_bounds.size,
        "layout_in should still return the compact root size after resize settles"
    );
    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "layout_in should keep the deferred rebuild flag clear"
    );
    assert!(
        !ui.interactive_resize_active(),
        "layout_in should settle after the configured quiet window"
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        roomy_root,
        "settled layout_in frame",
    );
}

#[test]
fn interactive_resize_layout_advances_resize_state_without_deferred_rebuild() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let roomy_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(680.0), Px(760.0)),
    );
    let compact_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(560.0)),
    );
    let mut services = FakeUiServices;

    let roomy_root =
        render_resize_sensitive_root(&mut ui, &mut app, &mut services, window, roomy_bounds, true);
    ui.set_root(roomy_root);
    ui.layout_all(&mut app, &mut services, roomy_bounds, 1.0);

    app.advance_frame();
    let compact_root = render_resize_sensitive_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        compact_bounds,
        false,
    );
    assert_eq!(compact_root, roomy_root, "expected stable root identity");
    ui.set_root(roomy_root);
    clear_all_invalidations(&mut ui);
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);
    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "layout path should start from an authoritative compact flow without a deferred rebuild"
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        roomy_root,
        "layout compact resize frame",
    );

    let settle_frames = interactive_resize_stable_frames_required();
    for quiet_frame in 1..settle_frames {
        app.advance_frame();
        let steady_size = ui.layout(
            &mut app,
            &mut services,
            roomy_root,
            compact_bounds.size,
            1.0,
        );
        assert_eq!(
            steady_size, compact_bounds.size,
            "quiet layout frame {quiet_frame} should preserve the compact root size"
        );
        assert!(
            ui.interactive_resize_active(),
            "quiet layout frame {quiet_frame} should still count as interactive resize"
        );
    }

    app.advance_frame();
    let rebuilt_size = ui.layout(
        &mut app,
        &mut services,
        roomy_root,
        compact_bounds.size,
        1.0,
    );
    assert_eq!(
        rebuilt_size, compact_bounds.size,
        "settled layout should still return the compact root size after the forced rebuild"
    );
    assert!(
        !ui.interactive_resize_active(),
        "layout should settle after the configured quiet window"
    );
    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "layout path should keep the deferred rebuild flag clear"
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        roomy_root,
        "settled layout frame",
    );
}

#[test]
fn interactive_resize_cached_flow_reuse_defers_full_rebuild_until_quiet_window() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let initial_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(680.0), Px(760.0)),
    );
    let resized_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(720.0), Px(760.0)),
    );
    let mut services = FakeUiServices;

    let root = ui.create_node_for_element(crate::elements::GlobalElementId(1), TestStack);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, initial_bounds, 1.0);
    clear_all_invalidations(&mut ui);

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, resized_bounds, 1.0);
    let resize_record = ui
        .debug_layout_request_build_roots()
        .iter()
        .find(|record| record.root == root)
        .expect("resize request-build record");
    assert_eq!(
        resize_record.mode, "cached_flow_reuse",
        "clean roots should use cached flow during interactive resize"
    );
    assert!(
        ui.interactive_resize_needs_full_rebuild,
        "cached-flow resize should arm a post-resize authoritative rebuild"
    );

    let settle_frames = interactive_resize_stable_frames_required();
    for quiet_frame in 1..settle_frames {
        app.advance_frame();
        ui.layout_all(&mut app, &mut services, resized_bounds, 1.0);
        assert!(
            ui.interactive_resize_active(),
            "quiet frame {quiet_frame} should not settle resize yet"
        );
        assert!(
            ui.interactive_resize_needs_full_rebuild,
            "quiet frame {quiet_frame} should keep the post-resize rebuild armed"
        );
        assert!(
            ui.debug_layout_request_build_roots().is_empty(),
            "quiet frame {quiet_frame} should stay on the layout fast path"
        );
    }

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, resized_bounds, 1.0);
    let settle_record = ui
        .debug_layout_request_build_roots()
        .iter()
        .find(|record| record.root == root)
        .expect("post-resize rebuild request-build record");
    assert_eq!(
        settle_record.mode, "build_flow",
        "post-resize settle should rebuild authoritative flow once"
    );
    assert!(
        settle_record.layout_invalidated,
        "post-resize rebuild should mark the root layout-dirty"
    );
    assert!(
        !ui.interactive_resize_active(),
        "resize should settle after the configured quiet window"
    );
    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "post-resize rebuild should consume the deferred rebuild flag"
    );
}

#[test]
fn layout_request_build_roots_sample_dirty_descendant_sources() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let leaf = ui.create_node_for_element(crate::elements::GlobalElementId(3), TestStack);
    let child = ui.create_node_for_element(crate::elements::GlobalElementId(2), TestStack);
    let root = ui.create_node_for_element(crate::elements::GlobalElementId(1), TestStack);
    ui.set_children(child, vec![leaf]);
    ui.set_children(root, vec![child]);
    ui.set_root(root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    );
    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for node in [leaf, child, root] {
        ui.test_clear_node_invalidations(node);
    }

    app.advance_frame();
    ui.begin_debug_frame_if_needed(app.frame_id());
    ui.invalidate_with_source_and_detail(
        leaf,
        Invalidation::Layout,
        UiDebugInvalidationSource::Other,
        UiDebugInvalidationDetail::ScrollHandleLayout,
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let root_record = ui
        .debug_layout_request_build_roots()
        .iter()
        .find(|r| r.root == root)
        .expect("request-build record for root");
    let sample = root_record
        .dirty_descendants
        .iter()
        .find(|d| d.source_root == Some(leaf))
        .expect("dirty descendant sample attributed to leaf invalidation");

    assert_eq!(
        sample.detail,
        Some(UiDebugInvalidationDetail::ScrollHandleLayout)
    );
    assert_eq!(sample.source, Some(UiDebugInvalidationSource::Other));
    assert!(
        sample.node == child || sample.node == leaf,
        "sample should point at a dirty descendant, got {:?}",
        sample.node
    );
    assert_eq!(root_record.descendant_layout_dirty_count, 2);
}

#[test]
fn layout_request_build_roots_classify_initial_mount_dirty_descendants() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let leaf = ui.create_node_for_element(crate::elements::GlobalElementId(3), TestStack);
    let child = ui.create_node_for_element(crate::elements::GlobalElementId(2), TestStack);
    let root = ui.create_node_for_element(crate::elements::GlobalElementId(1), TestStack);
    ui.set_children(child, vec![leaf]);
    ui.set_children(root, vec![child]);
    ui.set_root(root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    );
    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let root_record = ui
        .debug_layout_request_build_roots()
        .iter()
        .find(|r| r.root == root)
        .expect("request-build record for root");

    assert!(
        root_record.dirty_descendants.iter().any(|d| {
            d.source_root == Some(d.node)
                && d.detail == Some(UiDebugInvalidationDetail::InitialMount)
        }),
        "expected initial layout-dirty descendants to be classified as InitialMount: {:?}",
        root_record.dirty_descendants
    );
}

#[test]
fn layout_request_build_roots_classify_structural_child_rewrites() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let leaf = ui.create_node_for_element(crate::elements::GlobalElementId(3), TestStack);
    let parent = ui.create_node_for_element(crate::elements::GlobalElementId(2), TestStack);
    let root = ui.create_node_for_element(crate::elements::GlobalElementId(1), TestStack);
    ui.set_children(root, vec![parent]);
    ui.set_root(root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    );
    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for node in [leaf, parent, root] {
        ui.test_clear_node_invalidations(node);
    }

    app.advance_frame();
    ui.begin_debug_frame_if_needed(app.frame_id());
    ui.set_children(parent, vec![leaf]);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let root_record = ui
        .debug_layout_request_build_roots()
        .iter()
        .find(|r| r.root == root)
        .expect("request-build record for root");

    let sample = root_record
        .dirty_descendants
        .iter()
        .find(|d| d.node == parent)
        .expect("dirty descendant sample for structurally changed parent");
    assert_eq!(
        sample.detail,
        Some(UiDebugInvalidationDetail::StructuralChildrenChanged)
    );
}

#[test]
fn layout_request_build_roots_classify_view_cache_layout_dirty_expansion() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let leaf = ui.create_node_for_element(crate::elements::GlobalElementId(3), TestStack);
    let boundary = ui.create_node_for_element(crate::elements::GlobalElementId(2), TestStack);
    let root = ui.create_node_for_element(crate::elements::GlobalElementId(1), TestStack);
    ui.set_node_view_cache_flags(boundary, true, true, true);
    ui.set_children(boundary, vec![leaf]);
    ui.set_children(root, vec![boundary]);
    ui.set_root(root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    );
    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for node in [leaf, boundary, root] {
        ui.test_clear_node_invalidations(node);
    }

    app.advance_frame();
    ui.begin_debug_frame_if_needed(app.frame_id());
    ui.test_set_layout_invalidation(boundary, true);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let root_record = ui
        .debug_layout_request_build_roots()
        .iter()
        .find(|r| r.root == root)
        .expect("request-build record for root");

    assert!(
        root_record.dirty_descendants.iter().any(|d| {
            d.node == leaf
                && d.source_root == Some(boundary)
                && d.detail == Some(UiDebugInvalidationDetail::ViewCacheLayoutDirtyExpansion)
        }),
        "expected expanded view-cache descendants to carry ViewCacheLayoutDirtyExpansion detail: {:?}",
        root_record.dirty_descendants
    );
}
