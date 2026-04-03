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
fn interactive_resize_cached_flow_rebuilds_once_bounds_stabilize() {
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

    let mut page_node = ui.children(roomy_root)[0];
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

    page_node = ui.children(roomy_root)[0];
    let mut card_node = ui.children(page_node)[0];
    let page_instance =
        crate::declarative::frame::element_record_for_node(&mut app, window, page_node)
            .map(|r| r.instance)
            .expect("page instance after compact render");
    match page_instance {
        crate::declarative::frame::ElementInstance::Flex(props) => {
            assert_eq!(
                props.justify,
                crate::element::MainAlign::Start,
                "compact render should already author justify-start on the current page node"
            );
        }
        other => panic!("expected page node to remain a Flex, got {other:?}"),
    }
    assert!(
        ui.interactive_resize_needs_full_rebuild,
        "cached-flow resize frame should remember that a post-resize rebuild is required"
    );
    let stale_card_bounds = ui
        .debug_node_bounds(card_node)
        .expect("card bounds after resize");
    let engine = ui.take_layout_engine();
    let page_style = engine
        .debug_style_for_node(page_node)
        .cloned()
        .expect("page style after cached resize frame");
    ui.put_layout_engine(engine);
    assert_eq!(
        page_style.justify_content,
        Some(taffy::style::JustifyContent::Center),
        "cached-flow resize frame should still reflect the roomy style before settle rebuild"
    );
    assert!(
        stale_card_bounds.origin.y.0 > 0.0,
        "expected stale centered layout before the settle rebuild"
    );

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);
    assert!(
        ui.interactive_resize_needs_full_rebuild,
        "settle frame should keep the deferred rebuild flag armed until a full rebuild runs"
    );

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);

    page_node = ui.children(roomy_root)[0];
    card_node = ui.children(page_node)[0];
    let root_in_engine = ui.layout_engine_has_node(roomy_root);
    let page_in_engine = ui.layout_engine_has_node(page_node);
    let card_in_engine = ui.layout_engine_has_node(card_node);
    let page_instance =
        crate::declarative::frame::element_record_for_node(&mut app, window, page_node)
            .map(|r| r.instance)
            .expect("page instance after settle rebuild");
    match page_instance {
        crate::declarative::frame::ElementInstance::Flex(props) => {
            assert_eq!(
                props.justify,
                crate::element::MainAlign::Start,
                "settled frame should still be authored with justify-start"
            );
        }
        other => panic!("expected page node to remain a Flex, got {other:?}"),
    }
    let rebuilt_card_bounds = ui
        .debug_node_bounds(card_node)
        .expect("card bounds after rebuild");
    let engine = ui.take_layout_engine();
    let root_style = engine.debug_style_for_node(roomy_root).cloned();
    let page_style = engine.debug_style_for_node(page_node).cloned();
    let card_style = engine.debug_style_for_node(card_node).cloned();
    ui.put_layout_engine(engine);

    assert!(
        root_in_engine,
        "settle rebuild should keep the root in the engine"
    );
    assert!(
        page_in_engine,
        "settle rebuild should rebuild the page node into the engine"
    );
    assert!(
        card_in_engine,
        "settle rebuild should rebuild the card node into the engine"
    );
    assert!(
        root_style.is_some(),
        "settle rebuild should retain a root style in the engine"
    );
    assert!(
        page_style.is_some(),
        "page style after settle rebuild: root_in_engine={root_in_engine} page_in_engine={page_in_engine} card_in_engine={card_in_engine} root_style={root_style:?}"
    );
    assert!(
        card_style.is_some(),
        "card style after settle rebuild: root_in_engine={root_in_engine} page_in_engine={page_in_engine} card_in_engine={card_in_engine} root_style={root_style:?} page_style={page_style:?}"
    );

    assert_eq!(
        page_style.unwrap().justify_content,
        Some(taffy::style::JustifyContent::FlexStart),
        "expected a full flow rebuild once interactive resize stabilizes"
    );
    assert_eq!(
        card_style.unwrap().min_size.height,
        taffy::style::Dimension::length(120.0),
        "expected compact min-height constraints to be forwarded after the settle rebuild"
    );
    assert!(
        rebuilt_card_bounds.origin.y.0 <= 0.5,
        "expected compact layout to pin the card to the top once stale flow is rebuilt; rebuilt_card_bounds={rebuilt_card_bounds:?}"
    );
}

#[test]
fn interactive_resize_viewport_root_rebuilds_once_bounds_stabilize() {
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

    let mut page_node = ui.children(viewport_root)[0];
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

    page_node = ui.children(viewport_root)[0];
    let card_node = ui.children(page_node)[0];
    let stale_card_bounds = ui
        .debug_node_bounds(card_node)
        .expect("card bounds after cached viewport resize");
    let engine = ui.take_layout_engine();
    let page_style = engine
        .debug_style_for_node(page_node)
        .cloned()
        .expect("page style after cached viewport resize");
    ui.put_layout_engine(engine);
    assert!(
        ui.interactive_resize_needs_full_rebuild,
        "cached viewport resize should arm the deferred rebuild flag"
    );
    assert_eq!(
        page_style.justify_content,
        Some(taffy::style::JustifyContent::Center),
        "interactive resize should still reuse the previous viewport flow before settle"
    );
    assert!(
        stale_card_bounds.origin.y.0 > 0.0,
        "expected stale centered viewport layout before the settle rebuild"
    );

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);
    assert!(
        ui.interactive_resize_needs_full_rebuild,
        "first stable frame should keep the deferred viewport rebuild armed"
    );

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, compact_bounds, 1.0);

    page_node = ui.children(viewport_root)[0];
    let card_node = ui.children(page_node)[0];
    let rebuilt_card_bounds = ui
        .debug_node_bounds(card_node)
        .expect("card bounds after viewport settle rebuild");
    let engine = ui.take_layout_engine();
    let page_style = engine
        .debug_style_for_node(page_node)
        .cloned()
        .expect("page style after viewport settle rebuild");
    let card_style = engine
        .debug_style_for_node(card_node)
        .cloned()
        .expect("card style after viewport settle rebuild");
    ui.put_layout_engine(engine);

    assert_eq!(
        page_style.justify_content,
        Some(taffy::style::JustifyContent::FlexStart),
        "settled viewport frame should rebuild the compact page style"
    );
    assert_eq!(
        card_style.min_size.height,
        taffy::style::Dimension::length(120.0),
        "settled viewport frame should forward compact min-height constraints"
    );
    assert!(
        rebuilt_card_bounds.origin.y.0 <= 0.5,
        "expected viewport root bounds to refresh after the forced rebuild; rebuilt_card_bounds={rebuilt_card_bounds:?}"
    );
}

#[test]
fn interactive_resize_layout_in_clears_deferred_rebuild_flag_after_forced_rebuild() {
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
        ui.interactive_resize_needs_full_rebuild,
        "cached-flow resize frame should arm the deferred rebuild flag"
    );

    app.advance_frame();
    ui.update_interactive_resize_state_for_layout(app.frame_id(), compact_bounds, 1.0);
    assert!(
        ui.interactive_resize_active(),
        "first stable frame should still count as interactive resize"
    );

    app.advance_frame();
    ui.update_interactive_resize_state_for_layout(app.frame_id(), compact_bounds, 1.0);
    assert!(
        !ui.interactive_resize_active(),
        "second stable frame should settle interactive resize state"
    );
    assert!(
        ui.interactive_resize_requires_full_rebuild(),
        "settled resize state should require one forced rebuild"
    );

    let rebuilt_size = ui.layout_in(&mut app, &mut services, roomy_root, compact_bounds, 1.0);
    assert_eq!(
        rebuilt_size, compact_bounds.size,
        "layout_in should still return the compact root size after the forced rebuild"
    );
    assert!(
        !ui.interactive_resize_needs_full_rebuild,
        "layout_in forced rebuild should clear the deferred rebuild flag"
    );

    let page_node = ui.children(roomy_root)[0];
    let card_node = ui.children(page_node)[0];
    let rebuilt_card_bounds = ui
        .debug_node_bounds(card_node)
        .expect("card bounds after layout_in forced rebuild");
    let engine = ui.take_layout_engine();
    let page_style = engine
        .debug_style_for_node(page_node)
        .cloned()
        .expect("page style after layout_in forced rebuild");
    let card_style = engine
        .debug_style_for_node(card_node)
        .cloned()
        .expect("card style after layout_in forced rebuild");
    ui.put_layout_engine(engine);

    assert_eq!(
        page_style.justify_content,
        Some(taffy::style::JustifyContent::FlexStart),
        "layout_in forced rebuild should rebuild the compact page style"
    );
    assert_eq!(
        card_style.min_size.height,
        taffy::style::Dimension::length(120.0),
        "layout_in forced rebuild should forward compact min-height constraints"
    );
    assert!(
        rebuilt_card_bounds.origin.y.0 <= 0.5,
        "layout_in forced rebuild should refresh retained bounds; rebuilt_card_bounds={rebuilt_card_bounds:?}"
    );
}
