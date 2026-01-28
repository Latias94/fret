use super::*;

#[test]
fn semantics_snapshot_includes_visible_roots_and_barrier() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let base = ui.create_node(TestStack);
    ui.set_root(base);
    let base_child = ui.create_node(TestStack);
    ui.add_child(base, base_child);

    let overlay_root = ui.create_node(TestStack);
    ui.push_overlay_root(overlay_root, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert_eq!(snap.roots.len(), 2);
    assert_eq!(snap.barrier_root, Some(overlay_root));
    assert_eq!(snap.focus_barrier_root, Some(overlay_root));
    assert_eq!(
        snap.nodes.iter().find(|n| n.id == base).unwrap().role,
        SemanticsRole::Window
    );
    assert_ne!(
        snap.nodes
            .iter()
            .find(|n| n.id == overlay_root)
            .unwrap()
            .role,
        SemanticsRole::Window
    );
    assert!(snap.nodes.iter().any(|n| n.id == base));
    assert!(snap.nodes.iter().any(|n| n.id == base_child));
    assert!(snap.nodes.iter().any(|n| n.id == overlay_root));
}

#[test]
fn semantics_snapshot_exposes_focus_barrier_root_independently_of_pointer_barrier() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let base = ui.create_node(TestStack);
    ui.set_root(base);

    let overlay_root = ui.create_node(TestStack);
    let layer = ui.push_overlay_root(overlay_root, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert_eq!(snap.barrier_root, Some(overlay_root));
    assert_eq!(snap.focus_barrier_root, Some(overlay_root));

    ui.set_layer_blocks_underlay_focus(layer, false);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert_eq!(snap.barrier_root, Some(overlay_root));
    assert_eq!(snap.focus_barrier_root, None);
}

#[test]
fn modal_barrier_clears_focus_and_capture_in_underlay() {
    struct CaptureOnDown;

    impl<H: UiHost> Widget<H> for CaptureOnDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            true
        }

        fn is_focusable(&self) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
                cx.request_focus(cx.node);
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let underlay = ui.create_node(CaptureOnDown);
    ui.add_child(root, underlay);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_in(&mut app, &mut services, root, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(underlay));
    assert_eq!(ui.captured(), Some(underlay));

    let overlay_root = ui.create_node(TestStack);
    let _layer = ui.push_overlay_root(overlay_root, true);

    assert_eq!(ui.focus(), None);
    assert_eq!(ui.captured(), None);
}

#[test]
fn focus_traversal_includes_roots_above_modal_barrier() {
    #[derive(Default)]
    struct Focusable;

    impl<H: UiHost> Widget<H> for Focusable {
        fn is_focusable(&self) -> bool {
            true
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let base_root = ui.create_node(TestStack);
    let underlay_focusable = ui.create_node(Focusable);
    ui.add_child(base_root, underlay_focusable);
    ui.set_root(base_root);

    let modal_root = ui.create_node(TestStack);
    let modal_focusable = ui.create_node(Focusable);
    ui.add_child(modal_root, modal_focusable);
    ui.push_overlay_root(modal_root, true);

    // Simulate a nested "portal" overlay that lives above the modal barrier (e.g. combobox popover
    // inside a dialog).
    let popup_root = ui.create_node(TestStack);
    let popup_focusable = ui.create_node(Focusable);
    ui.add_child(popup_root, popup_focusable);
    ui.push_overlay_root(popup_root, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Under a modal barrier, traversal must not reach underlay focusables.
    ui.set_focus(Some(modal_focusable));
    let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert_eq!(ui.focus(), Some(popup_focusable));

    let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert_eq!(ui.focus(), Some(modal_focusable));

    // Reverse direction should also wrap within the active layers set.
    let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.previous"));
    assert_eq!(ui.focus(), Some(popup_focusable));
}

#[test]
fn focus_traversal_prefers_topmost_overlay_root() {
    #[derive(Default)]
    struct Focusable;

    impl<H: UiHost> Widget<H> for Focusable {
        fn is_focusable(&self) -> bool {
            true
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let base_root = ui.create_node(TestStack);
    let base_focusable = ui.create_node(Focusable);
    ui.add_child(base_root, base_focusable);
    ui.set_root(base_root);

    let overlay_root = ui.create_node(TestStack);
    let overlay_focusable = ui.create_node(Focusable);
    ui.add_child(overlay_root, overlay_focusable);
    ui.push_overlay_root(overlay_root, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.set_focus(Some(base_focusable));
    let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert_eq!(ui.focus(), Some(overlay_focusable));

    let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert_eq!(ui.focus(), Some(base_focusable));
}

#[test]
fn tab_focus_next_runs_when_text_input_not_composing() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![BindingV1 {
                command: Some("focus.next".into()),
                platform: None,
                when: None,
                keys: KeySpecV1 {
                    mods: vec![],
                    key: "Tab".into(),
                },
            }],
        })
        .expect("valid keymap"),
    });

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text_input = ui.create_node(crate::text_input::TextInput::new());
    ui.add_child(root, text_input);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_input));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Command { command, .. } if *command == CommandId::from("focus.next")
        )),
        "expected focus traversal command effect"
    );
}

#[test]
fn tab_focus_next_is_suppressed_during_ime_composition() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![BindingV1 {
                command: Some("focus.next".into()),
                platform: None,
                when: None,
                keys: KeySpecV1 {
                    mods: vec![],
                    key: "Tab".into(),
                },
            }],
        })
        .expect("valid keymap"),
    });

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text_input = ui.create_node(crate::text_input::TextInput::new());
    ui.add_child(root, text_input);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_input));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "toukyou".into(),
            cursor: Some((0, 0)),
        }),
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    let effects = app.take_effects();
    assert!(
        !effects.iter().any(|e| matches!(
            e,
            Effect::Command { command, .. } if *command == CommandId::from("focus.next")
        )),
        "did not expect focus traversal command effect during IME composition"
    );
}

#[test]
fn tab_focus_next_is_suppressed_when_preedit_empty_but_cursor_present() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![BindingV1 {
                command: Some("focus.next".into()),
                platform: None,
                when: None,
                keys: KeySpecV1 {
                    mods: vec![],
                    key: "Tab".into(),
                },
            }],
        })
        .expect("valid keymap"),
    });

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text_input = ui.create_node(crate::text_input::TextInput::new());
    ui.add_child(root, text_input);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_input));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "".into(),
            cursor: Some((0, 0)),
        }),
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    let effects = app.take_effects();
    assert!(
        !effects.iter().any(|e| matches!(
            e,
            Effect::Command { command, .. } if *command == CommandId::from("focus.next")
        )),
        "did not expect focus traversal command effect during IME composition"
    );
}

#[test]
fn tab_focus_next_runs_after_ime_commit_clears_composition() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![BindingV1 {
                command: Some("focus.next".into()),
                platform: None,
                when: None,
                keys: KeySpecV1 {
                    mods: vec![],
                    key: "Tab".into(),
                },
            }],
        })
        .expect("valid keymap"),
    });

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text_input = ui.create_node(crate::text_input::TextInput::new());
    ui.add_child(root, text_input);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_input));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "toukyou".into(),
            cursor: Some((0, 0)),
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Ime(fret_core::ImeEvent::Commit("東京".into())),
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| matches!(
            e,
            Effect::Command { command, .. } if *command == CommandId::from("focus.next")
        )),
        "expected focus traversal command effect after IME commit"
    );
}

#[test]
fn reserved_shortcuts_are_suppressed_during_ime_composition() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![
                BindingV1 {
                    command: Some("test.tab".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Tab".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.enter".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Enter".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.numpad_enter".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "NumpadEnter".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.space".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Space".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.escape".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Escape".into(),
                    },
                },
            ],
        })
        .expect("valid keymap"),
    });

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text_input = ui.create_node(crate::text_input::TextInput::new());
    ui.add_child(root, text_input);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_input));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "toukyou".into(),
            cursor: Some((0, 0)),
        }),
    );
    let _ = app.take_effects();

    for key in [
        KeyCode::Tab,
        KeyCode::Enter,
        KeyCode::NumpadEnter,
        KeyCode::Space,
        KeyCode::Escape,
    ] {
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
    }

    let effects = app.take_effects();
    assert!(
        !effects.iter().any(|e| matches!(e, Effect::Command { .. })),
        "did not expect any shortcut commands during IME composition"
    );
}

#[test]
fn reserved_shortcuts_are_suppressed_during_text_area_ime_composition() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![
                BindingV1 {
                    command: Some("test.tab".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Tab".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.enter".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Enter".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.numpad_enter".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "NumpadEnter".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.space".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Space".into(),
                    },
                },
                BindingV1 {
                    command: Some("test.escape".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Escape".into(),
                    },
                },
            ],
        })
        .expect("valid keymap"),
    });

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text_area = ui.create_node(crate::text_area::TextArea::default());
    ui.add_child(root, text_area);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_area));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "nihao".into(),
            cursor: Some((0, 0)),
        }),
    );
    let _ = app.take_effects();

    for key in [
        KeyCode::Tab,
        KeyCode::Enter,
        KeyCode::NumpadEnter,
        KeyCode::Space,
        KeyCode::Escape,
    ] {
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );
    }

    let effects = app.take_effects();
    assert!(
        !effects.iter().any(|e| matches!(e, Effect::Command { .. })),
        "did not expect any shortcut commands during IME composition"
    );
}

#[test]
fn tab_focus_next_is_suppressed_during_text_area_ime_composition() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![BindingV1 {
                command: Some("focus.next".into()),
                platform: None,
                when: None,
                keys: KeySpecV1 {
                    mods: vec![],
                    key: "Tab".into(),
                },
            }],
        })
        .expect("valid keymap"),
    });

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text_area = ui.create_node(crate::text_area::TextArea::default());
    ui.add_child(root, text_area);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_area));

    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Ime(fret_core::ImeEvent::Preedit {
            text: "toukyou".into(),
            cursor: Some((0, 0)),
        }),
    );
    let _ = app.take_effects();

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    let effects = app.take_effects();
    assert!(
        !effects.iter().any(|e| matches!(
            e,
            Effect::Command { command, .. } if *command == CommandId::from("focus.next")
        )),
        "did not expect focus traversal command effect during IME composition"
    );
}

#[test]
fn remove_layer_uninstalls_overlay_and_removes_subtree() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let overlay_root = ui.create_node(TestStack);
    let overlay_child = ui.create_node(TestStack);
    ui.add_child(overlay_root, overlay_child);
    let layer = ui.push_overlay_root(overlay_root, true);

    // Pretend an overlay widget captured focus/pointer.
    ui.focus = Some(overlay_child);
    ui.captured.insert(fret_core::PointerId(0), overlay_child);

    let mut services = FakeUiServices;
    let removed_root = ui.remove_layer(&mut services, layer);

    assert_eq!(removed_root, Some(overlay_root));
    assert!(ui.layers.get(layer).is_none());
    assert!(!ui.layer_order.contains(&layer));
    assert!(!ui.root_to_layer.contains_key(&overlay_root));

    assert!(ui.nodes.get(overlay_root).is_none());
    assert!(ui.nodes.get(overlay_child).is_none());
    assert_eq!(ui.focus(), None);
    assert_eq!(ui.captured(), None);
}

#[test]
fn event_cx_bounds_tracks_translated_nodes() {
    struct BoundsProbe {
        out: Model<Point>,
    }

    impl BoundsProbe {
        fn new(out: Model<Point>) -> Self {
            Self { out }
        }
    }

    impl<H: UiHost> Widget<H> for BoundsProbe {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if !matches!(event, Event::Pointer(PointerEvent::Move { .. })) {
                return;
            }
            let origin = cx.bounds.origin;
            let _ = cx
                .app
                .models_mut()
                .update(&self.out, |v: &mut Point| *v = origin);
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let out = app.models_mut().insert(Point::new(Px(0.0), Px(0.0)));

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let probe = ui.create_node(BoundsProbe::new(out.clone()));
    ui.add_child(root, probe);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let size = Size::new(Px(120.0), Px(40.0));

    ui.layout_in(
        &mut app,
        &mut services,
        root,
        Rect::new(Point::new(Px(0.0), Px(0.0)), size),
        1.0,
    );

    // Layout again with the same size but translated origin: the tree uses a fast-path that
    // translates node bounds without re-running widget.layout for the subtree.
    ui.layout_in(
        &mut app,
        &mut services,
        root,
        Rect::new(Point::new(Px(0.0), Px(100.0)), size),
        1.0,
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(110.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let origin = app.models().get_copied(&out).unwrap_or_default();
    assert_eq!(origin, Point::new(Px(0.0), Px(100.0)));
}
