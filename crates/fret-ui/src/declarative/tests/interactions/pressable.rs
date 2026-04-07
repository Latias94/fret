use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct ClipboardWriteHookState {
    pending: Option<fret_core::ClipboardToken>,
    successes: u32,
    failures: u32,
    last_token: Option<fret_core::ClipboardToken>,
    last_failure_kind: Option<fret_core::ClipboardAccessErrorKind>,
}

impl ClipboardWriteHookState {
    fn pending(token: fret_core::ClipboardToken) -> Self {
        Self {
            pending: Some(token),
            ..Default::default()
        }
    }
}

#[derive(Clone)]
struct ClipboardWriteHookPressable {
    label: &'static str,
    state: fret_runtime::Model<ClipboardWriteHookState>,
}

fn record_clipboard_write_completion(
    host: &mut dyn crate::action::UiActionHost,
    state: &fret_runtime::Model<ClipboardWriteHookState>,
    token: fret_core::ClipboardToken,
    outcome: &fret_core::ClipboardWriteOutcome,
) -> bool {
    let mut handled = false;
    let _ = host
        .models_mut()
        .update(state, |hook_state: &mut ClipboardWriteHookState| {
            if hook_state.pending != Some(token) {
                return;
            }

            handled = true;
            hook_state.pending = None;
            hook_state.last_token = Some(token);
            hook_state.last_failure_kind = None;
            match outcome {
                fret_core::ClipboardWriteOutcome::Succeeded => {
                    hook_state.successes = hook_state.successes.saturating_add(1);
                }
                fret_core::ClipboardWriteOutcome::Failed { error } => {
                    hook_state.failures = hook_state.failures.saturating_add(1);
                    hook_state.last_failure_kind = Some(error.kind);
                }
            }
        });
    handled
}

fn render_clipboard_write_hook_pressables(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    name: &'static str,
    pressables: Vec<ClipboardWriteHookPressable>,
) -> NodeId {
    render_root(ui, app, services, window, bounds, name, move |cx| {
        vec![cx.column(crate::element::ColumnProps::default(), {
            move |cx| {
                let mut children = Vec::new();
                for pressable in &pressables {
                    let label = pressable.label;
                    let state = pressable.state.clone();
                    children.push(cx.pressable(
                        crate::element::PressableProps::default(),
                        move |cx, _state| {
                            let state = state.clone();
                            cx.pressable_on_clipboard_write_completed(Arc::new(
                                move |host, _action_cx, token, outcome| {
                                    record_clipboard_write_completion(host, &state, token, outcome)
                                },
                            ));
                            vec![cx.text(label)]
                        },
                    ));
                }
                children
            }
        })]
    })
}

fn dispatch_clipboard_write_completed(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    token: fret_core::ClipboardToken,
    outcome: fret_core::ClipboardWriteOutcome,
) {
    ui.dispatch_event(
        app,
        services,
        &fret_core::Event::ClipboardWriteCompleted { token, outcome },
    );
}

#[test]
fn pressable_state_reports_focused_when_focused() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let focused = Rc::new(Cell::new(false));
    let pressable_element_id: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));

    fn render_frame(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        focused_out: Rc<Cell<bool>>,
        pressable_id_out: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
    ) -> NodeId {
        render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "pressable-state-reports-focused",
            move |cx| {
                let focused_out = focused_out.clone();
                let pressable_id_out = pressable_id_out.clone();
                vec![cx.pressable_with_id(
                    crate::element::PressableProps::default(),
                    move |cx, st, id| {
                        pressable_id_out.set(Some(id));
                        focused_out.set(st.focused);
                        vec![cx.text("pressable")]
                    },
                )]
            },
        )
    }

    // First frame: render once to establish stable identity + node mapping.
    let root = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        focused.clone(),
        pressable_element_id.clone(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(!focused.get());

    let pressable_element = pressable_element_id.get().expect("pressable element id");
    let pressable_node = crate::elements::node_for_element(&mut app, window, pressable_element)
        .expect("pressable node");
    ui.set_focus(Some(pressable_node));

    // Second frame: the authoring context should observe the focused element.
    app.advance_frame();
    let root = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        focused.clone(),
        pressable_element_id.clone(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(focused.get());
}

#[test]
fn element_context_reports_focus_within_for_focused_descendant() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let focus_within = Rc::new(Cell::new(false));
    let outer_element_id: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let inner_element_id: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));

    fn render_frame(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        focus_within_out: Rc<Cell<bool>>,
        outer_id_out: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
        inner_id_out: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
    ) -> NodeId {
        render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "element-context-focus-within",
            move |cx| {
                let focus_within_out = focus_within_out.clone();
                let outer_id_out = outer_id_out.clone();
                let inner_id_out = inner_id_out.clone();
                vec![cx.pressable_with_id(
                    crate::element::PressableProps::default(),
                    move |cx, _st, outer_id| {
                        outer_id_out.set(Some(outer_id));
                        focus_within_out.set(cx.is_focus_within_element(outer_id));
                        let inner_id_out = inner_id_out.clone();
                        vec![cx.pressable_with_id(
                            crate::element::PressableProps::default(),
                            move |cx, _st, inner_id| {
                                inner_id_out.set(Some(inner_id));
                                vec![cx.text("inner")]
                            },
                        )]
                    },
                )]
            },
        )
    }

    let root = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        focus_within.clone(),
        outer_element_id.clone(),
        inner_element_id.clone(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(!focus_within.get());

    let inner_element = inner_element_id.get().expect("inner element id");
    let inner_node =
        crate::elements::node_for_element(&mut app, window, inner_element).expect("inner node");
    ui.set_focus(Some(inner_node));

    app.advance_frame();
    let root = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        focus_within.clone(),
        outer_element_id,
        inner_element_id,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(focus_within.get());
}

#[test]
fn element_context_focus_within_ignores_stale_detached_node_entries() {
    use crate::elements::NodeEntry;
    use std::cell::Cell;
    use std::rc::Rc;

    struct DetachedDummy;

    impl<H: UiHost> Widget<H> for DetachedDummy {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(Px(0.0), Px(0.0))
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let focus_within = Rc::new(Cell::new(false));
    let outer_element_id: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let inner_element_id: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let root_name = "element-context-focus-within-stale-node-entry";

    fn render_frame(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        root_name: &'static str,
        focus_within_out: Rc<Cell<bool>>,
        outer_id_out: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
        inner_id_out: Rc<Cell<Option<crate::elements::GlobalElementId>>>,
    ) -> NodeId {
        render_root(ui, app, services, window, bounds, root_name, move |cx| {
            let focus_within_out = focus_within_out.clone();
            let outer_id_out = outer_id_out.clone();
            let inner_id_out = inner_id_out.clone();
            vec![cx.pressable_with_id(
                crate::element::PressableProps::default(),
                move |cx, _st, outer_id| {
                    outer_id_out.set(Some(outer_id));
                    focus_within_out.set(cx.is_focus_within_element(outer_id));
                    let inner_id_out = inner_id_out.clone();
                    vec![cx.pressable_with_id(
                        crate::element::PressableProps::default(),
                        move |cx, _st, inner_id| {
                            inner_id_out.set(Some(inner_id));
                            vec![cx.text("inner")]
                        },
                    )]
                },
            )]
        })
    }

    let root = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        focus_within.clone(),
        outer_element_id.clone(),
        inner_element_id.clone(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let outer_element = outer_element_id.get().expect("outer element id");
    let inner_element = inner_element_id.get().expect("inner element id");
    let inner_node =
        crate::elements::node_for_element(&mut app, window, inner_element).expect("inner node");
    ui.set_focus(Some(inner_node));

    let stale_outer = ui.create_node_for_element(outer_element, DetachedDummy);
    let stale_inner = ui.create_node_for_element(inner_element, DetachedDummy);
    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            outer_element,
            NodeEntry {
                node: stale_outer,
                last_seen_frame: frame_id,
                root: crate::elements::global_root(window, root_name),
            },
        );
        st.set_node_entry(
            inner_element,
            NodeEntry {
                node: stale_inner,
                last_seen_frame: frame_id,
                root: crate::elements::global_root(window, root_name),
            },
        );
    });

    app.advance_frame();
    let root = render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        focus_within.clone(),
        outer_element_id,
        inner_element_id,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        focus_within.get(),
        "expected ElementContext::is_focus_within_element to use the live window-frame nodes instead of stale detached node_entry seeds"
    );
}

#[test]
fn pressable_on_activate_hook_runs_on_pointer_activation() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let activated = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-activate-hook-pointer",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let activated = activated.clone();
                    cx.pressable_on_activate(Arc::new(move |host, _cx, reason| {
                        assert_eq!(reason, ActivateReason::Pointer);
                        let _ = host
                            .models_mut()
                            .update(&activated, |v: &mut bool| *v = true);
                    }));
                    vec![cx.text("activate")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&activated), Some(false));

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let position = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&activated), Some(true));
}

#[test]
fn pressable_clipboard_write_completed_hook_handles_matching_success_token() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let token = fret_core::ClipboardToken(41);
    let hook_state = app
        .models_mut()
        .insert(ClipboardWriteHookState::pending(token));

    let root = render_clipboard_write_hook_pressables(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clipboard-write-matching-success",
        vec![ClipboardWriteHookPressable {
            label: "copy",
            state: hook_state.clone(),
        }],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    dispatch_clipboard_write_completed(
        &mut ui,
        &mut app,
        &mut services,
        token,
        fret_core::ClipboardWriteOutcome::Succeeded,
    );

    assert_eq!(
        app.models().get_copied(&hook_state),
        Some(ClipboardWriteHookState {
            pending: None,
            successes: 1,
            failures: 0,
            last_token: Some(token),
            last_failure_kind: None,
        })
    );
}

#[test]
fn pressable_clipboard_write_completed_hook_ignores_non_matching_token() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let pending_token = fret_core::ClipboardToken(7);
    let hook_state = app
        .models_mut()
        .insert(ClipboardWriteHookState::pending(pending_token));

    let root = render_clipboard_write_hook_pressables(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clipboard-write-non-matching-token",
        vec![ClipboardWriteHookPressable {
            label: "copy",
            state: hook_state.clone(),
        }],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    dispatch_clipboard_write_completed(
        &mut ui,
        &mut app,
        &mut services,
        fret_core::ClipboardToken(999),
        fret_core::ClipboardWriteOutcome::Succeeded,
    );

    assert_eq!(
        app.models().get_copied(&hook_state),
        Some(ClipboardWriteHookState::pending(pending_token))
    );
}

#[test]
fn pressable_clipboard_write_completed_hook_routes_multiple_pending_tokens_across_pressables() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let first_token = fret_core::ClipboardToken(11);
    let second_token = fret_core::ClipboardToken(29);
    let first_state = app
        .models_mut()
        .insert(ClipboardWriteHookState::pending(first_token));
    let second_state = app
        .models_mut()
        .insert(ClipboardWriteHookState::pending(second_token));

    let root = render_clipboard_write_hook_pressables(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clipboard-write-multiple-pending",
        vec![
            ClipboardWriteHookPressable {
                label: "copy-a",
                state: first_state.clone(),
            },
            ClipboardWriteHookPressable {
                label: "copy-b",
                state: second_state.clone(),
            },
        ],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    dispatch_clipboard_write_completed(
        &mut ui,
        &mut app,
        &mut services,
        second_token,
        fret_core::ClipboardWriteOutcome::Succeeded,
    );
    dispatch_clipboard_write_completed(
        &mut ui,
        &mut app,
        &mut services,
        first_token,
        fret_core::ClipboardWriteOutcome::Succeeded,
    );

    assert_eq!(
        app.models().get_copied(&first_state),
        Some(ClipboardWriteHookState {
            pending: None,
            successes: 1,
            failures: 0,
            last_token: Some(first_token),
            last_failure_kind: None,
        })
    );
    assert_eq!(
        app.models().get_copied(&second_state),
        Some(ClipboardWriteHookState {
            pending: None,
            successes: 1,
            failures: 0,
            last_token: Some(second_token),
            last_failure_kind: None,
        })
    );
}

#[test]
fn pressable_clipboard_write_completed_hook_routes_failure_without_success_side_effect() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let token = fret_core::ClipboardToken(61);
    let hook_state = app
        .models_mut()
        .insert(ClipboardWriteHookState::pending(token));

    let root = render_clipboard_write_hook_pressables(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clipboard-write-failure",
        vec![ClipboardWriteHookPressable {
            label: "copy",
            state: hook_state.clone(),
        }],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    dispatch_clipboard_write_completed(
        &mut ui,
        &mut app,
        &mut services,
        token,
        fret_core::ClipboardWriteOutcome::Failed {
            error: fret_core::ClipboardAccessError {
                kind: fret_core::ClipboardAccessErrorKind::Unavailable,
                message: Some("clipboard unavailable".to_string()),
            },
        },
    );

    assert_eq!(
        app.models().get_copied(&hook_state),
        Some(ClipboardWriteHookState {
            pending: None,
            successes: 0,
            failures: 1,
            last_token: Some(token),
            last_failure_kind: Some(fret_core::ClipboardAccessErrorKind::Unavailable),
        })
    );
}

#[test]
fn pressable_clears_pressed_and_releases_capture_on_move_without_buttons() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let pressable_element_id_cell: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let pressable_element_id_cell_for_render = pressable_element_id_cell.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clear-pressed-on-move-without-buttons",
        move |cx| {
            let pressable_element_id_cell = pressable_element_id_cell_for_render.clone();
            vec![cx.pressable_with_id(
                crate::element::PressableProps::default(),
                move |cx, _state, id| {
                    pressable_element_id_cell.set(Some(id));
                    vec![cx.text("pressable")]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_element_id = pressable_element_id_cell
        .get()
        .expect("missing pressable element id");
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        Some(pressable_node)
    );
    assert!(
        crate::elements::is_pressed_pressable(&mut app, window, pressable_element_id),
        "expected pressable to set pressed state on pointer down"
    );

    // Simulate a runner/platform edge case: we never receive `PointerEvent::Up`, but we do observe
    // that no buttons are pressed anymore.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(inside.x.0 + 10.0), inside.y),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.captured_for(fret_core::PointerId(0)), None);
    assert!(!crate::elements::is_pressed_pressable(
        &mut app,
        window,
        pressable_element_id
    ));
}

#[test]
fn pressable_clears_pressed_state_on_pointer_cancel() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let pressable_element_id_cell: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let pressable_element_id_cell_for_render = pressable_element_id_cell.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clear-pressed-on-pointer-cancel",
        move |cx| {
            let pressable_element_id_cell = pressable_element_id_cell_for_render.clone();
            vec![cx.pressable_with_id(
                crate::element::PressableProps::default(),
                move |cx, _state, id| {
                    pressable_element_id_cell.set(Some(id));
                    vec![cx.text("pressable")]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_element_id = pressable_element_id_cell
        .get()
        .expect("missing pressable element id");
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    let pointer_id = fret_core::PointerId(0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.captured_for(pointer_id), Some(pressable_node));
    assert!(crate::elements::is_pressed_pressable(
        &mut app,
        window,
        pressable_element_id
    ));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id,
            position: Some(inside),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );

    assert_eq!(ui.captured_for(pointer_id), None);
    assert!(!crate::elements::is_pressed_pressable(
        &mut app,
        window,
        pressable_element_id
    ));
}

#[test]
fn pressable_clears_pressed_state_when_element_is_removed() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let pressable_element_id_cell: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let pressable_element_id_cell_for_render = pressable_element_id_cell.clone();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clear-pressed-on-unmount",
        move |cx| {
            let pressable_element_id_cell = pressable_element_id_cell_for_render.clone();
            vec![cx.pressable_with_id(
                crate::element::PressableProps::default(),
                move |cx, _state, id| {
                    pressable_element_id_cell.set(Some(id));
                    vec![cx.text("pressable")]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_element_id = pressable_element_id_cell
        .get()
        .expect("missing pressable element id");
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        crate::elements::is_pressed_pressable(&mut app, window, pressable_element_id),
        "expected pressable to be pressed after pointer down"
    );

    // Drop the pressable element without sending pointer up/cancel events (e.g. overlay closes).
    app.advance_frame();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-clear-pressed-on-unmount",
        |_cx| Vec::new(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        !crate::elements::is_pressed_pressable(&mut app, window, pressable_element_id),
        "expected pressed state to clear when the element is removed"
    );
}

#[test]
fn pressable_on_hover_change_hook_runs_on_pointer_move() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let hovered = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-hover-change-hook",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let hovered = hovered.clone();
                    cx.pressable_on_hover_change(Arc::new(move |host, _cx, is_hovered| {
                        let _ = host
                            .models_mut()
                            .update(&hovered, |v: &mut bool| *v = is_hovered);
                    }));
                    vec![cx.text("hover me")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&hovered), Some(false));

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: inside,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&hovered), Some(true));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(pressable_bounds.origin.x.0 + 200.0), Px(2.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&hovered), Some(false));
}

#[test]
fn pressable_on_hover_change_hook_runs_after_wheel_scroll_without_pointer_move() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let hovered = app.models_mut().insert(None::<u32>);
    let handle = crate::scroll::ScrollHandle::default();
    let item_h = Px(20.0);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-hover-after-wheel-scroll",
        |cx| {
            let scroll = cx.scroll(
                crate::element::ScrollProps {
                    layout: {
                        let mut layout = crate::element::LayoutStyle::default();
                        layout.size.width = crate::element::Length::Fill;
                        layout.size.height = crate::element::Length::Fill;
                        layout.overflow = crate::element::Overflow::Clip;
                        layout
                    },
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(crate::element::ColumnProps::default(), |cx| {
                        (0..20)
                            .map(|idx| {
                                let hovered = hovered.clone();
                                cx.keyed(idx, move |cx| {
                                    cx.pressable(
                                        crate::element::PressableProps {
                                            layout: {
                                                let mut layout =
                                                    crate::element::LayoutStyle::default();
                                                layout.size.width = crate::element::Length::Fill;
                                                layout.size.height =
                                                    crate::element::Length::Px(item_h);
                                                layout
                                            },
                                            ..Default::default()
                                        },
                                        move |cx, _state| {
                                            cx.pressable_on_hover_change(Arc::new(
                                                move |host, _cx, is_hovered| {
                                                    if !is_hovered {
                                                        return;
                                                    }
                                                    let _ = host
                                                        .models_mut()
                                                        .update(&hovered, |v: &mut Option<u32>| {
                                                            *v = Some(idx as u32)
                                                        });
                                                },
                                            ));
                                            vec![cx.text(format!("Item {idx}"))]
                                        },
                                    )
                                })
                            })
                            .collect::<Vec<_>>()
                    })]
                },
            );
            vec![scroll]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&hovered), Some(None));

    let position = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(Some(0)));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            pointer_id: fret_core::PointerId(0),
            position,
            delta: Point::new(Px(0.0), Px(-20.0)),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(Some(1)));
}

#[test]
fn touch_pan_scroll_steals_capture_from_pressable_and_clears_pressed_state() {
    use std::cell::Cell;
    use std::rc::Rc;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let handle = crate::scroll::ScrollHandle::default();
    let handle_for_ui = handle.clone();
    let pressed_element: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "touch-pan-steals-capture-from-pressable",
        {
            let pressed_element = pressed_element.clone();
            move |cx| {
                let scroll = cx.scroll(
                    crate::element::ScrollProps {
                        layout: {
                            let mut layout = crate::element::LayoutStyle::default();
                            layout.size.width = crate::element::Length::Fill;
                            layout.size.height = crate::element::Length::Fill;
                            layout.overflow = crate::element::Overflow::Clip;
                            layout
                        },
                        axis: crate::element::ScrollAxis::Y,
                        scroll_handle: Some(handle_for_ui.clone()),
                        ..Default::default()
                    },
                    move |cx| {
                        let pressed_element = pressed_element.clone();
                        vec![
                            cx.column(crate::element::ColumnProps::default(), move |cx| {
                                (0..30)
                                    .map(|idx| {
                                        let pressed_element = pressed_element.clone();
                                        cx.keyed(idx, move |cx| {
                                            let mut props =
                                                crate::element::PressableProps::default();
                                            props.layout.size.width = crate::element::Length::Fill;
                                            props.layout.size.height =
                                                crate::element::Length::Px(Px(20.0));

                                            if idx == 0 {
                                                cx.pressable_with_id(props, move |cx, _st, id| {
                                                    pressed_element.set(Some(id));
                                                    vec![cx.text("Row 0")]
                                                })
                                            } else {
                                                cx.pressable(props, move |cx, _st| {
                                                    vec![cx.text(format!("Row {idx}"))]
                                                })
                                            }
                                        })
                                    })
                                    .collect::<Vec<_>>()
                            }),
                        ]
                    },
                );
                vec![scroll]
            }
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        handle.max_offset().y.0 > 0.01,
        "expected scrollable content (max_offset={:?})",
        handle.max_offset()
    );

    let pressable_element = pressed_element.get().expect("pressable element id");
    let pressable_node = crate::elements::node_for_element(&mut app, window, pressable_element)
        .expect("pressable node");
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 4.0),
        Px(pressable_bounds.origin.y.0 + 4.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Touch,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(inside.x, Px(inside.y.0 - 80.0)),
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );

    assert!(
        handle.offset().y.0 > 0.01,
        "expected touch pan to scroll parent (offset={:?})",
        handle.offset()
    );
    assert!(
        !crate::elements::is_pressed_pressable(&mut app, window, pressable_element),
        "expected pressable pressed state to be cleared when touch pan scroll starts"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(inside.x, Px(inside.y.0 - 80.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_type: fret_core::PointerType::Touch,
        }),
    );
}

#[test]
fn touch_pan_scroll_live_target_resolution_ignores_stale_detached_node_entry() {
    use crate::elements::NodeEntry;
    use std::cell::Cell;
    use std::rc::Rc;

    struct DetachedDummy;

    impl<H: UiHost> Widget<H> for DetachedDummy {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(Px(0.0), Px(0.0))
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let handle = crate::scroll::ScrollHandle::default();
    let handle_for_ui = handle.clone();
    let pressed_element: Rc<Cell<Option<crate::elements::GlobalElementId>>> =
        Rc::new(Cell::new(None));
    let root_name = "touch-pan-live-target-stale-node-entry";

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        root_name,
        {
            let pressed_element = pressed_element.clone();
            move |cx| {
                let scroll = cx.scroll(
                    crate::element::ScrollProps {
                        layout: {
                            let mut layout = crate::element::LayoutStyle::default();
                            layout.size.width = crate::element::Length::Fill;
                            layout.size.height = crate::element::Length::Fill;
                            layout.overflow = crate::element::Overflow::Clip;
                            layout
                        },
                        axis: crate::element::ScrollAxis::Y,
                        scroll_handle: Some(handle_for_ui.clone()),
                        ..Default::default()
                    },
                    move |cx| {
                        let pressed_element = pressed_element.clone();
                        vec![
                            cx.column(crate::element::ColumnProps::default(), move |cx| {
                                (0..30)
                                    .map(|idx| {
                                        let pressed_element = pressed_element.clone();
                                        cx.keyed(idx, move |cx| {
                                            let mut props =
                                                crate::element::PressableProps::default();
                                            props.layout.size.width = crate::element::Length::Fill;
                                            props.layout.size.height =
                                                crate::element::Length::Px(Px(20.0));

                                            if idx == 0 {
                                                cx.pressable_with_id(props, move |cx, _st, id| {
                                                    pressed_element.set(Some(id));
                                                    vec![cx.text("Row 0")]
                                                })
                                            } else {
                                                cx.pressable(props, move |cx, _st| {
                                                    vec![cx.text(format!("Row {idx}"))]
                                                })
                                            }
                                        })
                                    })
                                    .collect::<Vec<_>>()
                            }),
                        ]
                    },
                );
                vec![scroll]
            }
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        handle.max_offset().y.0 > 0.01,
        "expected scrollable content (max_offset={:?})",
        handle.max_offset()
    );

    let pressable_element = pressed_element.get().expect("pressable element id");
    let pressable_node = crate::elements::node_for_element(&mut app, window, pressable_element)
        .expect("pressable node");
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 4.0),
        Px(pressable_bounds.origin.y.0 + 4.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: inside,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Touch,
        }),
    );

    let stale_detached = ui.create_node_for_element(pressable_element, DetachedDummy);
    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            pressable_element,
            NodeEntry {
                node: stale_detached,
                last_seen_frame: frame_id,
                root: crate::elements::global_root(window, root_name),
            },
        );
    });

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(inside.x, Px(inside.y.0 - 80.0)),
            buttons: MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );

    assert!(
        handle.offset().y.0 > 0.01,
        "expected touch pan live-target resolution to ignore stale detached node_entry seeds and keep routing into the scroll parent (offset={:?})",
        handle.offset()
    );
    assert!(
        !crate::elements::is_pressed_pressable(&mut app, window, pressable_element),
        "expected pressable pressed state to be cleared even when the retained node_entry was replaced by a stale detached node"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(inside.x, Px(inside.y.0 - 80.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_type: fret_core::PointerType::Touch,
        }),
    );
}

#[test]
fn pressable_hover_state_ignores_touch_pointer_moves() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let hovered = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-hover-ignores-touch",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let hovered = hovered.clone();
                    cx.pressable_on_hover_change(Arc::new(move |host, _cx, is_hovered| {
                        let _ = host
                            .models_mut()
                            .update(&hovered, |v: &mut bool| *v = is_hovered);
                    }));
                    vec![cx.text("hover me")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let inside = Point::new(
        Px(pressable_bounds.origin.x.0 + 1.0),
        Px(pressable_bounds.origin.y.0 + 1.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: inside,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(true));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(pressable_bounds.origin.x.0 + 200.0), Px(2.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(true));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(pressable_bounds.origin.x.0 + 200.0), Px(2.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&hovered), Some(false));
}

#[test]
fn pressable_on_activate_hook_runs_on_keyboard_activation() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let activated = app.models_mut().insert(false);

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-on-activate-hook-keyboard",
        |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    let activated = activated.clone();
                    cx.pressable_on_activate(Arc::new(move |host, _cx, reason| {
                        assert_eq!(reason, ActivateReason::Keyboard);
                        let _ = host
                            .models_mut()
                            .update(&activated, |v: &mut bool| *v = true);
                    }));
                    vec![cx.text("activate")]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&activated), Some(false));

    let pressable_node = ui.children(root)[0];
    ui.set_focus(Some(pressable_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyUp {
            key: fret_core::KeyCode::Enter,
            modifiers: Modifiers::default(),
        },
    );

    assert_eq!(app.models().get_copied(&activated), Some(true));
}

#[test]
fn pressable_pointer_click_focuses_pressable_even_when_not_in_tab_order() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(40.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-pointer-focuses-even-when-not-tab-stop",
        |cx| {
            let mut props = crate::element::PressableProps::default();
            props.layout.size.width = crate::element::Length::Fill;
            props.layout.size.height = crate::element::Length::Fill;
            props.focusable = false;

            vec![cx.pressable(props, |cx, _state| vec![cx.text("pressable")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let position = Point::new(
        Px(pressable_bounds.origin.x.0 + 4.0),
        Px(pressable_bounds.origin.y.0 + 4.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), None);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.focus(),
        Some(pressable_node),
        "expected pointer activation to focus the pressable even when it is excluded from Tab traversal"
    );
}

#[test]
fn pressable_pointer_up_does_not_steal_focus_from_text_input_descendant() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(160.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "pressable-text-input-descendant-focus",
        |cx| {
            let mut pressable_props = crate::element::PressableProps::default();
            pressable_props.layout.size.width = crate::element::Length::Fill;
            pressable_props.layout.size.height = crate::element::Length::Fill;
            pressable_props.focusable = true;

            let mut region_props = crate::element::TextInputRegionProps::default();
            region_props.layout.size.width = crate::element::Length::Fill;
            region_props.layout.size.height = crate::element::Length::Fill;
            region_props.a11y_label = Some(Arc::from("Editor"));

            vec![cx.pressable(pressable_props, move |cx, _state| {
                vec![cx.text_input_region(region_props, |_cx| Vec::<AnyElement>::new())]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let pressable_node = ui.children(root)[0];
    let region_node = ui.children(pressable_node)[0];
    let region_bounds = ui.debug_node_bounds(region_node).expect("region bounds");
    let position = Point::new(
        Px(region_bounds.origin.x.0 + 4.0),
        Px(region_bounds.origin.y.0 + 4.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.focus(),
        Some(region_node),
        "expected text input descendant to take focus on pointer down"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        ui.focus(),
        Some(region_node),
        "expected ancestor pressable not to steal focus on pointer up"
    );

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("window text input snapshot");
    assert!(snapshot.focus_is_text_input);
}

#[test]
fn pressable_semantics_checked_is_exposed() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-pressable-checked",
        |cx| {
            vec![cx.pressable(
                crate::element::PressableProps {
                    enabled: true,
                    a11y: crate::element::PressableA11y {
                        role: Some(fret_core::SemanticsRole::Checkbox),
                        label: Some(Arc::from("checked")),
                        checked: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |cx, _state| vec![cx.text("x")],
            )]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == fret_core::SemanticsRole::Checkbox && n.label.as_deref() == Some("checked")
        })
        .expect("expected checkbox semantics node");

    assert_eq!(node.flags.checked, Some(true));
    assert!(node.actions.invoke, "expected checkbox to be invokable");
}
