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

#[test]
fn owner_scoped_action_hooks_coexist_with_legacy_command_hooks() {
    struct ActionOwnerA;
    struct ActionOwnerB;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(160.0), Px(80.0)),
    );
    let mut services = FakeTextService::default();
    let events = app.models_mut().insert(Vec::<&'static str>::new());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "owner-scoped-action-hooks",
        |cx| {
            let events_for_owner_a = events.clone();
            let events_for_owner_b = events.clone();
            let events_for_legacy = events.clone();

            vec![
                cx.container(crate::element::ContainerProps::default(), move |cx| {
                    let id = cx.root_id();

                    cx.action_on_command_for_owner::<ActionOwnerA>(
                        id,
                        Arc::new(move |host, acx, command| {
                            if command.as_str() != "test.shared" {
                                return false;
                            }
                            let _ = host
                                .models_mut()
                                .update(&events_for_owner_a, |events| events.push("action-a"));
                            host.request_redraw(acx.window);
                            true
                        }),
                    );
                    cx.action_on_command_availability_for_owner::<ActionOwnerA>(
                        id,
                        Arc::new(|_host, _acx, command| {
                            if command.as_str() == "test.shared" {
                                return crate::widget::CommandAvailability::Available;
                            }
                            crate::widget::CommandAvailability::NotHandled
                        }),
                    );

                    cx.action_on_command_for_owner::<ActionOwnerB>(
                        id,
                        Arc::new(move |host, acx, command| {
                            if command.as_str() != "test.other" {
                                return false;
                            }
                            let _ = host
                                .models_mut()
                                .update(&events_for_owner_b, |events| events.push("action-b"));
                            host.request_redraw(acx.window);
                            true
                        }),
                    );
                    cx.action_on_command_availability_for_owner::<ActionOwnerB>(
                        id,
                        Arc::new(|_host, _acx, command| {
                            if command.as_str() == "test.other" {
                                return crate::widget::CommandAvailability::Available;
                            }
                            crate::widget::CommandAvailability::NotHandled
                        }),
                    );

                    cx.command_on_command_for(
                        id,
                        Arc::new(move |host, acx, command| {
                            if !matches!(command.as_str(), "test.shared" | "test.legacy") {
                                return false;
                            }
                            let _ = host
                                .models_mut()
                                .update(&events_for_legacy, |events| events.push("legacy"));
                            host.request_redraw(acx.window);
                            true
                        }),
                    );
                    cx.command_on_command_availability_for(
                        id,
                        Arc::new(|_host, _acx, command| match command.as_str() {
                            "test.shared" => crate::widget::CommandAvailability::Blocked,
                            "test.legacy" => crate::widget::CommandAvailability::Available,
                            _ => crate::widget::CommandAvailability::NotHandled,
                        }),
                    );

                    let mut pressable = crate::element::PressableProps::default();
                    pressable.layout.size.width = Length::Px(Px(48.0));
                    pressable.layout.size.height = Length::Px(Px(20.0));
                    pressable.focusable = true;

                    vec![cx.pressable(pressable, |cx, _st| vec![cx.text("focus")])]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let focus_node = ui.children(container_node)[0];
    ui.set_focus(Some(focus_node));

    let shared = CommandId::from("test.shared");
    let other = CommandId::from("test.other");
    let legacy = CommandId::from("test.legacy");

    assert_eq!(
        ui.command_availability(&mut app, &shared),
        crate::widget::CommandAvailability::Available,
        "owner-scoped action availability should win over the legacy command lane",
    );
    assert_eq!(
        ui.command_availability(&mut app, &other),
        crate::widget::CommandAvailability::Available,
    );
    assert_eq!(
        ui.command_availability(&mut app, &legacy),
        crate::widget::CommandAvailability::Available,
    );

    ui.dispatch_command(&mut app, &mut services, &shared);
    ui.dispatch_command(&mut app, &mut services, &other);
    ui.dispatch_command(&mut app, &mut services, &legacy);

    let recorded = app
        .models()
        .read(&events, Clone::clone)
        .expect("recorded command events");
    assert_eq!(recorded, vec!["action-a", "action-b", "legacy"]);
}
