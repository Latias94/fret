use super::*;

#[test]
fn command_hooks_can_request_focus_on_other_elements() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(120.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();

    let source_el: std::rc::Rc<std::cell::Cell<Option<crate::elements::GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));
    let target_el: std::rc::Rc<std::cell::Cell<Option<crate::elements::GlobalElementId>>> =
        std::rc::Rc::new(std::cell::Cell::new(None));

    let source_el_for_children = source_el.clone();
    let target_el_for_children = target_el.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "command-hooks-focus",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            let root = cx.container(props, move |cx| {
                let mut pressable_props = crate::element::PressableProps::default();
                pressable_props.layout.size.width = Length::Px(Px(40.0));
                pressable_props.layout.size.height = Length::Px(Px(20.0));
                pressable_props.focusable = true;

                let source = cx.keyed("source", |cx| {
                    cx.pressable(pressable_props.clone(), |cx, _st| vec![cx.text("source")])
                });
                source_el_for_children.set(Some(source.id));

                let target = cx.keyed("target", |cx| {
                    cx.pressable(pressable_props.clone(), |cx, _st| vec![cx.text("target")])
                });
                target_el_for_children.set(Some(target.id));

                vec![source, target]
            });

            let target = target_el.get().expect("target element id");
            cx.command_add_on_command_for(
                root.id,
                Arc::new(move |host, acx, command| {
                    if command.as_str() != "test.focus_target" {
                        return false;
                    }
                    host.request_focus(target);
                    host.request_redraw(acx.window);
                    true
                }),
            );

            vec![root]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let source = source_el.get().expect("source element id");
    let target = target_el.get().expect("target element id");
    let source_node =
        crate::declarative::mount::node_for_element_in_window_frame(&mut app, window, source)
            .expect("source node");
    let target_node =
        crate::declarative::mount::node_for_element_in_window_frame(&mut app, window, target)
            .expect("target node");

    ui.set_focus(Some(source_node));
    assert_eq!(ui.focus(), Some(source_node));

    ui.dispatch_command(
        &mut app,
        &mut services,
        &CommandId::from("test.focus_target"),
    );
    assert_eq!(ui.focus(), Some(target_node));
}
