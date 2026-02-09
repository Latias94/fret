use super::*;

#[test]
fn roving_flex_arrow_keys_move_focus_and_update_selection() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let model = app
        .models_mut()
        .insert(Option::<Arc<str>>::Some(Arc::from("a")));
    let values: Arc<[Arc<str>]> = Arc::from([Arc::from("a"), Arc::from("b"), Arc::from("c")]);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "roving-flex",
        |cx| {
            let props = crate::element::RovingFlexProps {
                flex: crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                roving: crate::element::RovingFocusProps {
                    enabled: true,
                    wrap: true,
                    disabled: Arc::from([false, true, false]),
                },
            };

            vec![cx.roving_flex(props, |cx| {
                let values = values.clone();
                let model = model.clone();
                cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                    use crate::action::RovingNavigateResult;
                    use fret_core::KeyCode;

                    let Some(current) = it.current else {
                        return RovingNavigateResult::NotHandled;
                    };

                    let forward = match it.key {
                        KeyCode::ArrowDown => true,
                        KeyCode::ArrowUp => false,
                        _ => return RovingNavigateResult::NotHandled,
                    };

                    let len = it.len;
                    let is_disabled =
                        |idx: usize| -> bool { it.disabled.get(idx).copied().unwrap_or(false) };

                    let mut target: Option<usize> = None;
                    if it.wrap {
                        for step in 1..=len {
                            let idx = if forward {
                                (current + step) % len
                            } else {
                                (current + len - (step % len)) % len
                            };
                            if !is_disabled(idx) {
                                target = Some(idx);
                                break;
                            }
                        }
                    } else if forward {
                        target = ((current + 1)..len).find(|&i| !is_disabled(i));
                    } else if current > 0 {
                        target = (0..current).rev().find(|&i| !is_disabled(i));
                    }

                    RovingNavigateResult::Handled { target }
                }));
                cx.roving_on_active_change(Arc::new(move |host, _cx, idx| {
                    let Some(value) = values.get(idx).cloned() else {
                        return;
                    };
                    let next = Some(value);
                    let _ = host
                        .models_mut()
                        .update(&model, |v: &mut Option<Arc<str>>| *v = next);
                }));

                let mut make = |label: &'static str| {
                    cx.pressable(
                        crate::element::PressableProps::default(),
                        |child_cx, _st| vec![child_cx.text(label)],
                    )
                };
                vec![make("a"), make("b"), make("c")]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let roving = ui.children(root)[0];
    let a = ui.children(roving)[0];
    let c = ui.children(roving)[2];
    ui.set_focus(Some(a));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(
        ui.focus(),
        Some(c),
        "expected ArrowDown to skip disabled child"
    );
    assert_eq!(
        app.models().get_cloned(&model).flatten().as_deref(),
        Some("c"),
    );
}


#[test]
fn roving_flex_treats_descendant_focus_as_active_item() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "roving-flex-focus-within",
        |cx| {
            let props = crate::element::RovingFlexProps {
                flex: crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                roving: crate::element::RovingFocusProps {
                    enabled: true,
                    wrap: true,
                    disabled: Arc::from([false, false]),
                },
            };

            vec![cx.roving_flex(props, |cx| {
                cx.roving_on_navigate(Arc::new(|_host, _cx, it| {
                    use crate::action::RovingNavigateResult;
                    use fret_core::KeyCode;

                    let Some(current) = it.current else {
                        return RovingNavigateResult::NotHandled;
                    };

                    let forward = match it.key {
                        KeyCode::ArrowDown => true,
                        KeyCode::ArrowUp => false,
                        _ => return RovingNavigateResult::NotHandled,
                    };

                    let len = it.len;
                    let target = if forward {
                        (current + 1) % len
                    } else {
                        (current + len - 1) % len
                    };

                    RovingNavigateResult::Handled {
                        target: Some(target),
                    }
                }));

                let mut make = |label: &'static str| {
                    cx.pressable(
                        crate::element::PressableProps::default(),
                        move |child_cx, _st| {
                            vec![child_cx.pressable(
                                crate::element::PressableProps::default(),
                                |inner_cx, _st| vec![inner_cx.text(label)],
                            )]
                        },
                    )
                };

                vec![make("a"), make("b")]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let roving = ui.children(root)[0];
    let a = ui.children(roving)[0];
    let a_inner = ui.children(a)[0];
    let b = ui.children(roving)[1];

    ui.set_focus(Some(a_inner));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(
        ui.focus(),
        Some(b),
        "expected roving to treat descendant focus as within the active item",
    );
}


#[test]
fn roving_flex_typeahead_hook_can_choose_target_index() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "roving-flex-typeahead-hook",
        |cx| {
            let props = crate::element::RovingFlexProps {
                flex: crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    ..Default::default()
                },
                roving: crate::element::RovingFocusProps {
                    enabled: true,
                    wrap: true,
                    disabled: Arc::from([false, false, false]),
                },
            };

            vec![cx.roving_flex(props, |cx| {
                cx.roving_on_typeahead(Arc::new(
                    |_host, _cx, it| {
                        if it.input == 'c' { Some(2) } else { None }
                    },
                ));

                let mut make = |label: &'static str| {
                    cx.pressable(
                        crate::element::PressableProps::default(),
                        |child_cx, _st| vec![child_cx.text(label)],
                    )
                };
                vec![make("a"), make("b"), make("c")]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let roving = ui.children(root)[0];
    let a = ui.children(roving)[0];
    let c = ui.children(roving)[2];
    ui.set_focus(Some(a));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::KeyC,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(ui.focus(), Some(c));
}

