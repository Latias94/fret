use super::*;

struct Noop;

impl<H: UiHost> Widget<H> for Noop {
    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::new(Px(0.0), Px(0.0))
    }
}

struct TimerCounter {
    hits: Arc<AtomicUsize>,
}

impl<H: UiHost> Widget<H> for TimerCounter {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if matches!(event, Event::Timer { .. }) {
            self.hits.fetch_add(1, Ordering::SeqCst);
            cx.stop_propagation();
        }
    }

    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::new(Px(10.0), Px(10.0))
    }
}

#[test]
fn timer_dispatch_uses_visible_layer_snapshot_when_input_layers_are_empty() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(Noop);
    let base_layer = ui.set_base_root(root);

    // Reproduce the "visible but hit-test-inert" state (e.g. transition frames).
    ui.set_layer_hit_testable(base_layer, false);

    let mut services = FakeUiServices;
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Timer {
            token: fret_core::TimerToken::default(),
        },
    );
}

#[test]
fn timer_broadcast_reaches_descendant_handlers_when_target_is_missing() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut ui = UiTree::new();
    ui.set_window(window);

    let hits = Arc::new(AtomicUsize::new(0));

    let root = ui.create_node(TestStack);
    let child = ui.create_node(TimerCounter { hits: hits.clone() });
    ui.add_child(root, child);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Timer {
            token: fret_core::TimerToken::default(),
        },
    );

    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "expected timer broadcast to reach descendant handlers when no live timer target is available"
    );
}

#[test]
fn timer_dispatch_resolves_live_attached_element_target_over_stale_detached_seed() {
    use crate::elements::NodeEntry;

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut ui = UiTree::new();
    ui.set_window(window);

    let hits = Arc::new(AtomicUsize::new(0));
    let target_element = crate::elements::GlobalElementId(4242);

    let root = ui.create_node(TestStack);
    let live_target =
        ui.create_node_for_element(target_element, TimerCounter { hits: hits.clone() });
    let stale_detached = ui.create_node_for_element(
        target_element,
        TimerCounter {
            hits: Arc::new(AtomicUsize::new(0)),
        },
    );
    ui.add_child(root, live_target);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let token = fret_runtime::TimerToken(7);
    let frame_id = app.frame_id();
    crate::elements::record_timer_target(&mut app, window, token, target_element);
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            target_element,
            NodeEntry {
                node: stale_detached,
                last_seen_frame: frame_id,
                root: target_element,
            },
        );
    });

    ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "expected timer dispatch to target the live attached node for the element instead of the stale detached node_entry"
    );
}

#[test]
fn final_layout_frame_syncs_hovered_pressable_node_to_live_attached_element() {
    use crate::elements::NodeEntry;

    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let element = crate::elements::GlobalElementId(4343);
    let root = ui.create_node(TestStack);
    let live_node = ui.create_node_for_element(element, TestStack);
    let stale_detached = ui.create_node_for_element(element, TestStack);
    ui.add_child(root, live_node);
    ui.set_root(root);

    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            element,
            NodeEntry {
                node: stale_detached,
                last_seen_frame: frame_id,
                root: element,
            },
        );
    });
    let _ = crate::elements::update_hovered_pressable_with_node(
        &mut app,
        window,
        Some((element, stale_detached)),
    );

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (prev_element, prev_node, next_element, next_node) =
        crate::elements::update_hovered_pressable(&mut app, window, None);
    assert_eq!(prev_element, Some(element));
    assert_eq!(
        prev_node,
        Some(live_node),
        "expected final layout-frame sync to retarget hovered pressable state to the live attached node for the element"
    );
    assert_eq!(next_element, None);
    assert_eq!(next_node, None);
}
