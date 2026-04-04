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

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);
    assert!(
        ui.interactive_resize_active(),
        "first stable frame should still count as interactive resize"
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        roomy_root,
        "first stable frame after cached-flow resize",
    );

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);
    assert!(
        !ui.interactive_resize_active(),
        "second stable frame should settle interactive resize state"
    );
    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "authoritative same-frame rebuild should not leave a deferred rebuild armed"
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        roomy_root,
        "settled frame after cached-flow resize",
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

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);
    assert!(
        ui.interactive_resize_active(),
        "first stable viewport frame should still count as interactive resize"
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        viewport_root,
        "first stable viewport frame",
    );

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);
    assert!(
        !ui.interactive_resize_active(),
        "second stable viewport frame should settle interactive resize state"
    );
    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "viewport-root resize should not leave a deferred rebuild armed"
    );
    assert_authoritative_compact_flow(
        &mut ui,
        &mut app,
        window,
        viewport_root,
        "settled viewport frame",
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

    app.advance_frame();
    let steady_size = ui.layout_in(&mut app, &mut services, roomy_root, compact_bounds, 1.0);
    assert_eq!(
        steady_size, compact_bounds.size,
        "first stable layout_in should preserve the compact root size while resize is still active"
    );
    assert!(
        ui.interactive_resize_active(),
        "first stable layout_in should still count as interactive resize"
    );

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
        "second stable layout_in should settle interactive resize state"
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
        "first stable layout should preserve the compact root size while resize is still active"
    );
    assert!(
        ui.interactive_resize_active(),
        "first stable layout should still count as interactive resize"
    );

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
        "second stable layout should settle interactive resize state"
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
