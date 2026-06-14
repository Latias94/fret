use super::*;

#[derive(Clone)]
struct CountingSemantics {
    label: &'static str,
    calls: Arc<AtomicUsize>,
}

impl<H: UiHost> Widget<H> for CountingSemantics {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }

    fn semantics(&mut self, cx: &mut SemanticsCx<'_, H>) {
        self.calls.fetch_add(1, Ordering::Relaxed);
        *cx.label = Some(self.label.to_string());
    }
}

#[derive(Clone)]
struct OffsetChildrenTransform {
    offset_y_px: Arc<AtomicUsize>,
}

impl<H: UiHost> Widget<H> for OffsetChildrenTransform {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }

    fn children_render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
        let offset_y = self.offset_y_px.load(Ordering::Relaxed) as f32;
        (offset_y > 0.0).then(|| Transform2D::translation(Point::new(Px(0.0), Px(-offset_y))))
    }
}

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
fn semantics_snapshot_rebuilds_clean_descendants_when_dirty_ancestor_transform_changes() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let offset_y_px = Arc::new(AtomicUsize::new(0));
    let transformed_parent = ui.create_node(OffsetChildrenTransform {
        offset_y_px: offset_y_px.clone(),
    });
    let child_calls = Arc::new(AtomicUsize::new(0));
    let child = ui.create_node(CountingSemantics {
        label: "child",
        calls: child_calls.clone(),
    });

    ui.add_child(root, transformed_parent);
    ui.add_child(transformed_parent, child);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let first_bounds = ui
        .semantics_snapshot()
        .and_then(|snapshot| snapshot.nodes.iter().find(|node| node.id == child))
        .map(|node| node.bounds)
        .expect("first child semantics bounds");
    assert_eq!(first_bounds.origin.y, Px(0.0));
    assert_eq!(child_calls.load(Ordering::Relaxed), 1);

    offset_y_px.store(40, Ordering::Relaxed);
    ui.mark_invalidation_with_source(
        transformed_parent,
        Invalidation::HitTest,
        UiDebugInvalidationSource::Notify,
    );
    assert!(ui.request_semantics_snapshot_if_dirty());
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let second_bounds = ui
        .semantics_snapshot()
        .and_then(|snapshot| snapshot.nodes.iter().find(|node| node.id == child))
        .map(|node| node.bounds)
        .expect("second child semantics bounds");
    assert_eq!(second_bounds.origin.y, Px(-40.0));
    assert_eq!(
        child_calls.load(Ordering::Relaxed),
        2,
        "dirty ancestor transforms must force clean descendants to rebuild bounds"
    );
}

#[test]
fn semantics_snapshot_reuses_clean_subtrees_between_dirty_refreshes() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let dirty_calls = Arc::new(AtomicUsize::new(0));
    let clean_calls = Arc::new(AtomicUsize::new(0));
    let dirty = ui.create_node(CountingSemantics {
        label: "dirty",
        calls: dirty_calls.clone(),
    });
    let clean = ui.create_node(CountingSemantics {
        label: "clean",
        calls: clean_calls.clone(),
    });
    ui.add_child(root, dirty);
    ui.add_child(root, clean);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(dirty_calls.load(Ordering::Relaxed), 1);
    assert_eq!(clean_calls.load(Ordering::Relaxed), 1);

    ui.mark_invalidation_with_source(
        dirty,
        Invalidation::Paint,
        UiDebugInvalidationSource::Notify,
    );
    assert!(ui.request_semantics_snapshot_if_dirty());
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(dirty_calls.load(Ordering::Relaxed), 2);
    assert_eq!(
        clean_calls.load(Ordering::Relaxed),
        1,
        "clean sibling semantics should replay from the previous snapshot"
    );
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    assert!(
        snap.nodes
            .iter()
            .any(|node| node.id == dirty && node.label.as_deref() == Some("dirty"))
    );
    assert!(
        snap.nodes
            .iter()
            .any(|node| node.id == clean && node.label.as_deref() == Some("clean"))
    );
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
fn semantics_snapshot_dirty_gate_rearms_on_semantic_invalidations() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    assert!(ui.request_semantics_snapshot_if_dirty());
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(ui.semantics_snapshot().is_some());
    assert!(
        !ui.request_semantics_snapshot_if_dirty(),
        "fresh semantics should not schedule another rebuild"
    );

    ui.invalidate(root, Invalidation::Paint);
    assert!(
        !ui.request_semantics_snapshot_if_dirty(),
        "plain paint invalidation should not dirty accessibility semantics"
    );

    ui.invalidate(root, Invalidation::HitTestOnly);
    assert!(
        ui.request_semantics_snapshot_if_dirty(),
        "hit-test/bounds changes can affect semantics bounds and should rearm the snapshot"
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(!ui.request_semantics_snapshot_if_dirty());

    ui.mark_invalidation_with_source(root, Invalidation::Paint, UiDebugInvalidationSource::Notify);
    assert!(
        ui.request_semantics_snapshot_if_dirty(),
        "widget notification paint invalidations may affect semantic state and should rearm the snapshot"
    );
}

#[test]
fn semantics_snapshot_dirty_gate_ignores_animation_frame_paint() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    assert!(ui.request_semantics_snapshot_if_dirty());
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(!ui.request_semantics_snapshot_if_dirty());

    ui.invalidate_with_source_and_detail(
        root,
        Invalidation::Paint,
        UiDebugInvalidationSource::Notify,
        UiDebugInvalidationDetail::AnimationFrameRequest,
    );
    assert!(
        !ui.request_semantics_snapshot_if_dirty(),
        "paint-only animation frames should not force accessibility tree rebuilds"
    );
}

#[test]
fn semantics_snapshot_dirty_gate_tracks_pointer_capture_owner() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    ui.set_root(root);
    let capture = ui.create_node(TestStack);
    ui.add_child(root, capture);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    assert!(ui.request_semantics_snapshot_if_dirty());
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(ui.semantics_snapshot().expect("snapshot").captured, None);
    assert!(!ui.request_semantics_snapshot_if_dirty());

    ui.captured.insert(fret_core::PointerId(0), capture);
    assert!(
        ui.request_semantics_snapshot_if_dirty(),
        "pointer capture changes semantic owner state even when layout is otherwise clean"
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("snapshot");
    assert_eq!(snap.captured, Some(capture));
    assert!(
        snap.nodes
            .iter()
            .any(|node| node.id == capture && node.flags.captured)
    );

    ui.captured.remove(&fret_core::PointerId(0));
    assert!(
        ui.request_semantics_snapshot_if_dirty(),
        "pointer capture release must clear stale semantic owner state"
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("snapshot");
    assert_eq!(snap.captured, None);
    assert!(
        snap.nodes
            .iter()
            .all(|node| node.id != capture || !node.flags.captured)
    );
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
    let other_focusable = ui.create_node(Focusable);
    ui.add_child(root, text_input);
    ui.add_child(root, other_focusable);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_input));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(
        ui.focus(),
        Some(other_focusable),
        "expected Tab to advance focus to the next focusable"
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

#[cfg(feature = "diagnostics")]
#[test]
fn ime_reserved_tab_reports_reserved_for_ime_when_text_widget_consumes() {
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

    let store = app
        .global::<fret_runtime::WindowShortcutRoutingDiagnosticsStore>()
        .expect("expected shortcut routing diagnostics");
    let decisions = store.snapshot_since(window, 0, 10);
    let last = decisions.last().expect("expected a routing decision");

    assert_eq!(last.phase, fret_runtime::ShortcutRoutingPhase::PostDispatch);
    assert!(last.deferred);
    assert!(last.focus_is_text_input);
    assert!(last.ime_composing);
    assert_eq!(last.key, KeyCode::Tab);
    assert_eq!(
        last.outcome,
        fret_runtime::ShortcutRoutingOutcome::ReservedForIme
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
    let other_focusable = ui.create_node(Focusable);
    ui.add_child(root, text_input);
    ui.add_child(root, other_focusable);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(40.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(text_input));

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

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::Tab,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );
    assert_eq!(
        ui.focus(),
        Some(other_focusable),
        "expected Tab to advance focus after IME commit"
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
