use super::*;

#[derive(Default)]
struct FocusableLeaf;

impl<H: UiHost> Widget<H> for FocusableLeaf {
    fn is_focusable(&self) -> bool {
        true
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn focus_barrier_can_remain_active_while_layer_is_hit_test_inert() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let base_root = ui.create_node(TestStack);
    let underlay_focusable = ui.create_node(FocusableLeaf);
    ui.set_root(base_root);
    ui.add_child(base_root, underlay_focusable);

    let overlay_root = ui.create_node(TestStack);
    let overlay_a = ui.create_node(FocusableLeaf);
    let overlay_b = ui.create_node(FocusableLeaf);
    ui.add_child(overlay_root, overlay_a);
    ui.add_child(overlay_root, overlay_b);

    // Simulate a close-transition style barrier: the overlay layer is still visible and blocks
    // underlay focus, but it is hit-test-inert for pointer events.
    let overlay_layer = ui.push_overlay_root_with_options(
        overlay_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: false,
            hit_testable: false,
        },
    );

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Without an active focus barrier, focusing inside the overlay must be allowed.
    ui.set_focus(Some(overlay_a));
    assert_eq!(ui.focus(), Some(overlay_a));

    // Activating the focus barrier must not clear focus, even if the barrier layer is not
    // hit-testable.
    ui.set_layer_blocks_underlay_focus(overlay_layer, true);
    assert_eq!(ui.focus(), Some(overlay_a));

    // Focus movement within the barrier scope must remain allowed.
    ui.set_focus(Some(overlay_b));
    assert_eq!(ui.focus(), Some(overlay_b));

    // Focus must not be allowed to escape to the underlay while the barrier is active.
    ui.set_focus(Some(underlay_focusable));
    assert_eq!(
        ui.focus(),
        Some(overlay_b),
        "expected focus barrier to reject underlay focus while active"
    );

    // Dispatch-time barrier enforcement must not clear focus under the same conditions.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Timer {
            token: fret_core::TimerToken(1),
        },
    );
    assert_eq!(ui.focus(), Some(overlay_b));
}
