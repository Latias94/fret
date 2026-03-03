use super::*;

struct Noop;

impl<H: UiHost> Widget<H> for Noop {
    fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
        Size::new(Px(0.0), Px(0.0))
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
