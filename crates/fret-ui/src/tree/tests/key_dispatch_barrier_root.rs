use super::*;

struct KeyCounter {
    hits: fret_runtime::Model<u32>,
}

impl KeyCounter {
    fn new(hits: fret_runtime::Model<u32>) -> Self {
        Self { hits }
    }
}

impl<H: UiHost> Widget<H> for KeyCounter {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if matches!(event, Event::KeyDown { .. }) {
            let _ = cx
                .app
                .models_mut()
                .update(&self.hits, |v: &mut u32| *v += 1);
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn key_events_route_to_focus_barrier_root_when_unfocused() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let underlay_hits = app.models_mut().insert(0u32);
    let overlay_hits = app.models_mut().insert(0u32);

    let base_root = ui.create_node(KeyCounter::new(underlay_hits.clone()));
    ui.set_root(base_root);

    let overlay_root = ui.create_node(KeyCounter::new(overlay_hits.clone()));
    let overlay_layer = ui.push_overlay_root_with_options(
        overlay_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: false,
            hit_testable: false,
        },
    );
    ui.set_layer_blocks_underlay_focus(overlay_layer, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // With a focus barrier active and no focused node, key events must not route into the underlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::KeyA,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(
        app.models().get_copied(&underlay_hits).unwrap_or_default(),
        0,
        "expected underlay to not receive key events while a focus barrier is active"
    );
    assert_eq!(
        app.models().get_copied(&overlay_hits).unwrap_or_default(),
        1,
        "expected overlay barrier root to receive key events while unfocused"
    );
}
