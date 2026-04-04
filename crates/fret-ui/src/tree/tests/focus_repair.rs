use super::*;

#[test]
fn focus_repair_prefers_live_attached_node_over_stale_detached_node_entry() {
    use crate::elements::NodeEntry;

    struct DetachedDummy;

    impl<H: UiHost> Widget<H> for DetachedDummy {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(Px(0.0), Px(0.0))
        }
    }

    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let mut focusable: Option<GlobalElementId> = None;
    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let root_name = "focus-repair-live-node";

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        |cx| {
            vec![
                cx.pressable_with_id(crate::element::PressableProps::default(), |cx, _st, id| {
                    focusable = Some(id);
                    vec![cx.text("focusable")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let focusable = focusable.expect("focusable element id");
    let live_node =
        crate::declarative::node_for_element_in_window_frame(&mut app, window, focusable)
            .expect("live attached focusable node");

    let stale_detached = ui.create_node_for_element(focusable, DetachedDummy);
    ui.set_focus(Some(stale_detached));

    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            focusable,
            NodeEntry {
                node: stale_detached,
                last_seen_frame: frame_id,
                root: crate::elements::global_root(window, root_name),
            },
        );
    });

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(live_node),
        "expected final-layout focus repair to prefer the live attached node over a stale detached node_entry seed"
    );
}
