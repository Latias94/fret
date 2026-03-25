use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ClipboardReadHookState {
    pending: Option<fret_core::ClipboardToken>,
    reads: u32,
    failures: u32,
    last_token: Option<fret_core::ClipboardToken>,
    last_text: Option<String>,
    last_error_kind: Option<fret_core::ClipboardAccessErrorKind>,
    last_error_message: Option<String>,
}

impl ClipboardReadHookState {
    fn pending(token: fret_core::ClipboardToken) -> Self {
        Self {
            pending: Some(token),
            ..Default::default()
        }
    }
}

#[derive(Clone)]
struct ClipboardReadHookRegion {
    label: &'static str,
    state: fret_runtime::Model<ClipboardReadHookState>,
}

fn record_clipboard_read_text(
    host: &mut dyn crate::action::UiActionHost,
    state: &fret_runtime::Model<ClipboardReadHookState>,
    token: fret_core::ClipboardToken,
    text: &str,
) -> bool {
    let mut handled = false;
    let _ = host
        .models_mut()
        .update(state, |hook_state: &mut ClipboardReadHookState| {
            if hook_state.pending != Some(token) {
                return;
            }

            handled = true;
            hook_state.pending = None;
            hook_state.reads = hook_state.reads.saturating_add(1);
            hook_state.last_token = Some(token);
            hook_state.last_text = Some(text.to_string());
            hook_state.last_error_kind = None;
            hook_state.last_error_message = None;
        });
    handled
}

fn record_clipboard_read_failed(
    host: &mut dyn crate::action::UiActionHost,
    state: &fret_runtime::Model<ClipboardReadHookState>,
    token: fret_core::ClipboardToken,
    error: &fret_core::ClipboardAccessError,
) -> bool {
    let mut handled = false;
    let _ = host
        .models_mut()
        .update(state, |hook_state: &mut ClipboardReadHookState| {
            if hook_state.pending != Some(token) {
                return;
            }

            handled = true;
            hook_state.pending = None;
            hook_state.failures = hook_state.failures.saturating_add(1);
            hook_state.last_token = Some(token);
            hook_state.last_text = None;
            hook_state.last_error_kind = Some(error.kind);
            hook_state.last_error_message = error.message.clone();
        });
    handled
}

fn render_clipboard_read_hook_regions(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    name: &'static str,
    regions: Vec<ClipboardReadHookRegion>,
) -> NodeId {
    render_root(ui, app, services, window, bounds, name, move |cx| {
        vec![cx.column(crate::element::ColumnProps::default(), {
            move |cx| {
                let mut children = Vec::new();
                for region in &regions {
                    let label = region.label;
                    let state = region.state.clone();
                    let mut props = crate::element::TextInputRegionProps::default();
                    props.layout.size.width = crate::element::Length::Fill;
                    props.layout.size.height = crate::element::Length::Px(Px(20.0));
                    props.a11y_label = Some(Arc::<str>::from(label));
                    props.a11y_value = Some(Arc::<str>::from(label));
                    children.push(cx.text_input_region(props, move |cx| {
                        let state_for_read = state.clone();
                        cx.text_input_region_on_clipboard_read_text(Arc::new(
                            move |host, _action_cx, token, text| {
                                record_clipboard_read_text(host, &state_for_read, token, text)
                            },
                        ));
                        let state_for_failure = state.clone();
                        cx.text_input_region_on_clipboard_read_failed(Arc::new(
                            move |host, _action_cx, token, error| {
                                record_clipboard_read_failed(host, &state_for_failure, token, error)
                            },
                        ));
                        vec![cx.text(label)]
                    }));
                }
                children
            }
        })]
    })
}

fn dispatch_clipboard_read_text(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    token: fret_core::ClipboardToken,
    text: &str,
) {
    ui.dispatch_event(
        app,
        services,
        &fret_core::Event::ClipboardReadText {
            token,
            text: text.to_string(),
        },
    );
}

fn dispatch_clipboard_read_failed(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    token: fret_core::ClipboardToken,
    error: fret_core::ClipboardAccessError,
) {
    ui.dispatch_event(
        app,
        services,
        &fret_core::Event::ClipboardReadFailed { token, error },
    );
}

#[test]
fn text_input_cut_updates_model_and_availability() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert("hello".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-cut-updates-model",
        |cx| vec![cx.text_input(crate::element::TextInputProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    let copy = CommandId::from("text.copy");
    let cut = CommandId::from("text.cut");
    let select_all = CommandId::from("text.select_all");

    assert!(
        ui.is_command_available(&mut app, &select_all),
        "expected text.select_all to be available for focused text input"
    );
    assert!(
        !ui.is_command_available(&mut app, &copy),
        "expected text.copy to be unavailable without a selection"
    );
    assert!(
        !ui.is_command_available(&mut app, &cut),
        "expected text.cut to be unavailable without a selection"
    );

    assert!(
        ui.dispatch_command(&mut app, &mut services, &select_all),
        "expected text.select_all to be handled by text input"
    );
    assert!(
        ui.is_command_available(&mut app, &copy),
        "expected text.copy to be available after select_all"
    );
    assert!(
        ui.is_command_available(&mut app, &cut),
        "expected text.cut to be available after select_all"
    );

    assert!(
        ui.dispatch_command(&mut app, &mut services, &cut),
        "expected text.cut to be handled by text input"
    );
    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some(""),
        "expected cut to update the bound model"
    );
    assert!(
        app.take_effects()
            .iter()
            .any(|e| matches!(e, fret_runtime::Effect::ClipboardWriteText { .. })),
        "expected text.cut to emit ClipboardWriteText"
    );
}

#[test]
fn text_input_paste_requests_clipboard_text_when_editable() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert("hello".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-paste-clipboard-get",
        |cx| vec![cx.text_input(crate::element::TextInputProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    let paste = CommandId::from("text.paste");
    assert!(
        ui.is_command_available(&mut app, &paste),
        "expected text.paste to be available for focused editable text input"
    );
    assert!(
        ui.dispatch_command(&mut app, &mut services, &paste),
        "expected text.paste to be handled by text input"
    );

    assert!(
        app.take_effects()
            .iter()
            .any(|e| matches!(e, fret_runtime::Effect::ClipboardReadText { .. })),
        "expected text.paste to request ClipboardReadText"
    );
}

#[test]
fn text_input_region_clipboard_read_text_hook_handles_matching_token() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let token = fret_core::ClipboardToken(101);
    let hook_state = app
        .models_mut()
        .insert(ClipboardReadHookState::pending(token));

    let root = render_clipboard_read_hook_regions(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-region-clipboard-read-match",
        vec![ClipboardReadHookRegion {
            label: "editor",
            state: hook_state.clone(),
        }],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    dispatch_clipboard_read_text(&mut ui, &mut app, &mut services, token, "hello");

    assert_eq!(
        app.models().get_cloned(&hook_state),
        Some(ClipboardReadHookState {
            pending: None,
            reads: 1,
            failures: 0,
            last_token: Some(token),
            last_text: Some("hello".to_string()),
            last_error_kind: None,
            last_error_message: None,
        })
    );
}

#[test]
fn text_input_region_clipboard_read_hooks_ignore_non_matching_token() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let pending_token = fret_core::ClipboardToken(202);
    let hook_state = app
        .models_mut()
        .insert(ClipboardReadHookState::pending(pending_token));

    let root = render_clipboard_read_hook_regions(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-region-clipboard-read-non-matching",
        vec![ClipboardReadHookRegion {
            label: "editor",
            state: hook_state.clone(),
        }],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    dispatch_clipboard_read_text(
        &mut ui,
        &mut app,
        &mut services,
        fret_core::ClipboardToken(999),
        "ignored",
    );

    assert_eq!(
        app.models().get_cloned(&hook_state),
        Some(ClipboardReadHookState::pending(pending_token))
    );
}

#[test]
fn text_input_region_clipboard_read_failed_hook_handles_matching_token() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let token = fret_core::ClipboardToken(303);
    let hook_state = app
        .models_mut()
        .insert(ClipboardReadHookState::pending(token));

    let root = render_clipboard_read_hook_regions(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-region-clipboard-read-failure",
        vec![ClipboardReadHookRegion {
            label: "editor",
            state: hook_state.clone(),
        }],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    dispatch_clipboard_read_failed(
        &mut ui,
        &mut app,
        &mut services,
        token,
        fret_core::ClipboardAccessError {
            kind: fret_core::ClipboardAccessErrorKind::Unavailable,
            message: Some("clipboard unavailable".to_string()),
        },
    );

    assert_eq!(
        app.models().get_cloned(&hook_state),
        Some(ClipboardReadHookState {
            pending: None,
            reads: 0,
            failures: 1,
            last_token: Some(token),
            last_text: None,
            last_error_kind: Some(fret_core::ClipboardAccessErrorKind::Unavailable),
            last_error_message: Some("clipboard unavailable".to_string()),
        })
    );
}

#[test]
fn text_input_region_clipboard_read_hooks_route_multiple_pending_tokens_across_regions() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut services = FakeTextService::default();

    let first_token = fret_core::ClipboardToken(401);
    let second_token = fret_core::ClipboardToken(402);
    let first_state = app
        .models_mut()
        .insert(ClipboardReadHookState::pending(first_token));
    let second_state = app
        .models_mut()
        .insert(ClipboardReadHookState::pending(second_token));

    let root = render_clipboard_read_hook_regions(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-region-clipboard-read-multiple-pending",
        vec![
            ClipboardReadHookRegion {
                label: "editor-a",
                state: first_state.clone(),
            },
            ClipboardReadHookRegion {
                label: "editor-b",
                state: second_state.clone(),
            },
        ],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    dispatch_clipboard_read_text(&mut ui, &mut app, &mut services, second_token, "beta");
    dispatch_clipboard_read_text(&mut ui, &mut app, &mut services, first_token, "alpha");

    assert_eq!(
        app.models().get_cloned(&first_state),
        Some(ClipboardReadHookState {
            pending: None,
            reads: 1,
            failures: 0,
            last_token: Some(first_token),
            last_text: Some("alpha".to_string()),
            last_error_kind: None,
            last_error_message: None,
        })
    );
    assert_eq!(
        app.models().get_cloned(&second_state),
        Some(ClipboardReadHookState {
            pending: None,
            reads: 1,
            failures: 0,
            last_token: Some(second_token),
            last_text: Some("beta".to_string()),
            last_error_kind: None,
            last_error_message: None,
        })
    );
}

#[test]
fn text_input_key_hooks_can_intercept_navigation_keys() {
    use fret_core::{Event, KeyCode, Modifiers};

    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert("hello".to_string());
    let opened = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let opened_for_hook = opened.clone();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-key-hooks-intercept",
        move |cx| {
            vec![cx.text_input_with_id_props(|cx, id| {
                let opened = opened_for_hook.clone();
                cx.key_add_on_key_down_for(
                    id,
                    Arc::new(move |host, action_cx, down| {
                        if down.key != KeyCode::ArrowDown {
                            return false;
                        }
                        let _ = host.models_mut().update(&opened, |v| *v = true);
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );
                crate::element::TextInputProps::new(model.clone())
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowDown,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert!(
        app.models().get_copied(&opened).unwrap_or(false),
        "expected key hook to run for focused text input"
    );
}

#[test]
fn text_input_middle_click_pastes_primary_selection_when_enabled() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::TextInteractionSettings {
        linux_primary_selection: true,
        ..Default::default()
    });
    let mut caps = fret_runtime::PlatformCapabilities::default();
    caps.clipboard.text.read = true;
    caps.clipboard.text.write = true;
    caps.clipboard.primary_text = true;
    app.set_global(caps);

    let model = app.models_mut().insert(String::new());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-primary-selection-middle-click",
        |cx| vec![cx.text_input(crate::element::TextInputProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    let input_bounds = ui.debug_node_bounds(input_node).expect("input bounds");
    let pos = Point::new(
        Px(input_bounds.origin.x.0 + 10.0),
        Px(input_bounds.origin.y.0 + 10.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: MouseButton::Middle,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let effects = app.take_effects();
    let Some(token) = effects.iter().find_map(|e| match e {
        fret_runtime::Effect::PrimarySelectionGetText { token, .. } => Some(*token),
        _ => None,
    }) else {
        panic!("expected middle click to request PrimarySelectionGetText");
    };

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::PrimarySelectionText {
            token,
            text: "hello".to_string(),
        },
    );

    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some("hello"),
        "expected primary selection paste to insert text into the bound model"
    );
}

#[test]
fn text_input_select_all_is_blocked_when_empty() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let model = app.models_mut().insert(String::new());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-select-all-empty",
        |cx| vec![cx.text_input(crate::element::TextInputProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    let select_all = CommandId::from("text.select_all");
    let edit_select_all = CommandId::from("edit.select_all");
    let clear = CommandId::from("text.clear");
    let edit_copy = CommandId::from("edit.copy");
    let edit_cut = CommandId::from("edit.cut");
    let unknown = CommandId::from("text.unknown");

    assert!(
        !ui.is_command_available(&mut app, &select_all),
        "expected text.select_all to be unavailable for empty text input"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_select_all),
        "expected edit.select_all to be unavailable for empty text input"
    );
    assert!(
        !ui.is_command_available(&mut app, &clear),
        "expected text.clear to be unavailable for empty text input"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_copy),
        "expected edit.copy to be unavailable without a selection"
    );
    assert!(
        !ui.is_command_available(&mut app, &edit_cut),
        "expected edit.cut to be unavailable without a selection"
    );
    assert!(
        !ui.is_command_available(&mut app, &unknown),
        "expected unknown text.* commands to be NotHandled for availability"
    );
}

#[test]
fn text_input_supports_edit_select_all_and_copy() {
    let mut app = TestHost::new();
    let mut caps = fret_runtime::PlatformCapabilities::default();
    caps.clipboard.text.read = true;
    caps.clipboard.text.write = true;
    app.set_global(caps);

    let model = app.models_mut().insert("hello".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-edit-select-all-copy",
        |cx| vec![cx.text_input(crate::element::TextInputProps::new(model.clone()))],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    ui.set_focus(Some(input_node));

    let select_all = CommandId::from("edit.select_all");
    assert!(
        ui.dispatch_command(&mut app, &mut services, &select_all),
        "expected edit.select_all to be handled by text input"
    );

    let copy = CommandId::from("edit.copy");
    assert!(
        ui.is_command_available(&mut app, &copy),
        "expected edit.copy to be available after select_all"
    );
    assert!(
        ui.dispatch_command(&mut app, &mut services, &copy),
        "expected edit.copy to be handled by text input"
    );
    assert!(
        app.take_effects().iter().any(
            |e| matches!(e, fret_runtime::Effect::ClipboardWriteText { text, .. } if text == "hello")
        ),
        "expected edit.copy to emit ClipboardWriteText for the selected text"
    );
}

#[test]
fn text_input_double_click_respects_window_text_boundary_mode_under_render_transform() {
    fn selection_for_mode(mode: fret_runtime::TextBoundaryMode) -> Option<(u32, u32)> {
        let mut app = TestHost::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());
        app.with_global_mut_untracked(
            fret_runtime::WindowTextBoundaryModeService::default,
            |svc, _app| {
                svc.set_base_mode(AppWindowId::default(), mode);
            },
        );

        let model = app.models_mut().insert("can't".to_string());

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(180.0), Px(60.0)));
        let mut services = FakeTextService::default();

        let transform = Transform2D::translation(Point::new(Px(40.0), Px(10.0)));
        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "text-input-double-click-boundary-mode-transform",
            |cx| {
                vec![cx.render_transform(transform, |cx| {
                    let mut props = crate::element::TextInputProps::new(model.clone());
                    props.layout.size.width = Length::Px(Px(120.0));
                    props.layout.size.height = Length::Px(Px(32.0));
                    vec![cx.text_input(props)]
                })]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let transform_node = ui.children(root)[0];
        let input_node = ui.children(transform_node)[0];
        let input_bounds = ui
            .debug_node_visual_bounds(input_node)
            .expect("input bounds");
        let pos = Point::new(
            Px(input_bounds.origin.x.0 + 5.0),
            Px(input_bounds.origin.y.0 + 5.0),
        );
        assert_eq!(
            ui.debug_hit_test(pos).hit,
            Some(input_node),
            "expected the translated hit-test position to target the text input"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert_eq!(ui.focus(), Some(input_node));
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: pos,
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 2,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert_eq!(ui.focus(), Some(input_node));

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let snapshot = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window))
            .cloned()
            .expect("expected a window text input snapshot");
        assert!(snapshot.focus_is_text_input);
        snapshot.selection_utf16
    }

    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::UnicodeWord),
        Some((0, 5)),
        "UnicodeWord should select the whole word"
    );
    assert_eq!(
        selection_for_mode(fret_runtime::TextBoundaryMode::Identifier),
        Some((0, 3)),
        "Identifier should stop at the apostrophe"
    );
}

#[test]
fn text_input_double_click_respects_window_text_boundary_mode_under_scroll_offset() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());
    app.with_global_mut_untracked(
        fret_runtime::WindowTextBoundaryModeService::default,
        |svc, _app| {
            svc.set_base_mode(
                AppWindowId::default(),
                fret_runtime::TextBoundaryMode::Identifier,
            );
        },
    );

    let model = app.models_mut().insert("can't".to_string());

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(180.0), Px(60.0)));
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-double-click-boundary-mode-scroll",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0).into(),
                            ..Default::default()
                        },
                        |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();
                            for _ in 0..40 {
                                let mut row_layout = crate::element::LayoutStyle::default();
                                row_layout.size.height = Length::Px(Px(18.0));
                                out.push(cx.container(
                                    crate::element::ContainerProps {
                                        layout: row_layout,
                                        ..Default::default()
                                    },
                                    |cx| vec![cx.text("filler")],
                                ));
                            }

                            let mut props = crate::element::TextInputProps::new(model.clone());
                            props.layout.size.width = Length::Px(Px(120.0));
                            props.layout.size.height = Length::Px(Px(32.0));
                            out.push(cx.text_input(props));

                            out
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let input_node = *ui
        .children(column_node)
        .last()
        .expect("expected input as last child");
    let input_bounds = ui.debug_node_bounds(input_node).expect("input bounds");

    scroll_handle.set_offset(Point::new(Px(0.0), input_bounds.origin.y));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let input_bounds = ui
        .debug_node_visual_bounds(input_node)
        .expect("input bounds after scroll");
    let pos = Point::new(
        Px(input_bounds.origin.x.0 + 5.0),
        Px(input_bounds.origin.y.0 + 5.0),
    );
    assert_eq!(
        ui.debug_hit_test(pos).hit,
        Some(input_node),
        "expected the scrolled hit-test position to target the text input"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(input_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(input_node));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot");
    assert!(snapshot.focus_is_text_input);
    let selection_utf16 = snapshot.selection_utf16;

    assert_eq!(
        selection_utf16,
        Some((0, 3)),
        "Identifier mode should stop at the apostrophe"
    );
}

#[test]
fn text_input_ctrl_arrow_word_navigation_respects_window_text_boundary_mode() {
    fn caret_positions_for_mode(mode: fret_runtime::TextBoundaryMode) -> (u32, u32) {
        let mut app = TestHost::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());
        app.with_global_mut_untracked(
            fret_runtime::WindowTextBoundaryModeService::default,
            |svc, _app| {
                svc.set_base_mode(AppWindowId::default(), mode);
            },
        );

        let model = app.models_mut().insert("can't".to_string());

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(60.0)));
        let mut services = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "text-input-ctrl-arrow-boundary-mode",
            |cx| {
                let mut props = crate::element::TextInputProps::new(model.clone());
                props.layout.size.width = Length::Px(Px(160.0));
                props.layout.size.height = Length::Px(Px(32.0));
                vec![cx.text_input(props)]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let input_node = ui.children(root)[0];
        ui.set_focus(Some(input_node));

        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let move_home = CommandId::from("text.move_home");
        assert!(
            ui.dispatch_command(&mut app, &mut services, &move_home),
            "expected text.move_home to be handled by text input"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowRight,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let snapshot = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window))
            .cloned()
            .expect("expected a window text input snapshot");
        assert!(snapshot.focus_is_text_input);
        let (anchor_u16, focus_u16) = snapshot.selection_utf16.expect("selection");
        assert_eq!(
            anchor_u16, focus_u16,
            "expected a collapsed selection after move"
        );
        let caret_right = focus_u16;

        let move_end = CommandId::from("text.move_end");
        assert!(
            ui.dispatch_command(&mut app, &mut services, &move_end),
            "expected text.move_end to be handled by text input"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowLeft,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );
        let mut scene = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let snapshot = app
            .global::<fret_runtime::WindowTextInputSnapshotService>()
            .and_then(|svc| svc.snapshot(window))
            .cloned()
            .expect("expected a window text input snapshot");
        assert!(snapshot.focus_is_text_input);
        let (anchor_u16, focus_u16) = snapshot.selection_utf16.expect("selection");
        assert_eq!(
            anchor_u16, focus_u16,
            "expected a collapsed selection after move"
        );
        let caret_left = focus_u16;

        (caret_right, caret_left)
    }

    assert_eq!(
        caret_positions_for_mode(fret_runtime::TextBoundaryMode::UnicodeWord),
        (5, 0),
        "UnicodeWord should treat \"can't\" as a single word"
    );
    assert_eq!(
        caret_positions_for_mode(fret_runtime::TextBoundaryMode::Identifier),
        (3, 4),
        "Identifier should split \"can't\" around the apostrophe"
    );
}

#[test]
fn text_input_ctrl_backspace_delete_word_respects_window_text_boundary_mode() {
    fn text_after_key(
        mode: fret_runtime::TextBoundaryMode,
        prep_command: &'static str,
        key: fret_core::KeyCode,
    ) -> String {
        let mut app = TestHost::new();
        app.set_global(fret_runtime::PlatformCapabilities::default());
        app.with_global_mut_untracked(
            fret_runtime::WindowTextBoundaryModeService::default,
            |svc, _app| {
                svc.set_base_mode(AppWindowId::default(), mode);
            },
        );

        let model = app.models_mut().insert("can't".to_string());

        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(220.0), Px(60.0)));
        let mut services = FakeTextService::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "text-input-ctrl-word-delete-boundary-mode",
            |cx| {
                let mut props = crate::element::TextInputProps::new(model.clone());
                props.layout.size.width = Length::Px(Px(160.0));
                props.layout.size.height = Length::Px(Px(32.0));
                vec![cx.text_input(props)]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let input_node = ui.children(root)[0];
        ui.set_focus(Some(input_node));

        let prep_command = CommandId::from(prep_command);
        assert!(
            ui.dispatch_command(&mut app, &mut services, &prep_command),
            "expected {prep_command:?} to be handled by text input"
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                repeat: false,
            },
        );

        app.models()
            .get_cloned(&model)
            .unwrap_or_default()
            .to_string()
    }

    assert_eq!(
        text_after_key(
            fret_runtime::TextBoundaryMode::UnicodeWord,
            "text.move_end",
            fret_core::KeyCode::Backspace,
        ),
        "",
        "UnicodeWord should delete the whole word on Ctrl+Backspace"
    );
    assert_eq!(
        text_after_key(
            fret_runtime::TextBoundaryMode::Identifier,
            "text.move_end",
            fret_core::KeyCode::Backspace,
        ),
        "can'",
        "Identifier should delete only the last identifier segment on Ctrl+Backspace"
    );
    assert_eq!(
        text_after_key(
            fret_runtime::TextBoundaryMode::UnicodeWord,
            "text.move_home",
            fret_core::KeyCode::Delete,
        ),
        "",
        "UnicodeWord should delete the whole word on Ctrl+Delete"
    );
    assert_eq!(
        text_after_key(
            fret_runtime::TextBoundaryMode::Identifier,
            "text.move_home",
            fret_core::KeyCode::Delete,
        ),
        "'t",
        "Identifier should delete only the first identifier segment on Ctrl+Delete"
    );
}

#[test]
fn text_input_double_click_cancels_ime_preedit() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let text = "hello world".to_string();
    let base_len_utf16: u32 = text.encode_utf16().count().try_into().unwrap();
    let model = app.models_mut().insert(text);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(180.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-double-click-cancels-ime-preedit",
        |cx| {
            let mut props = crate::element::TextInputProps::new(model.clone());
            props.layout.size.width = Length::Px(Px(160.0));
            props.layout.size.height = Length::Px(Px(32.0));
            vec![cx.text_input(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    let input_bounds = ui
        .debug_node_visual_bounds(input_node)
        .expect("input bounds");
    let pos = Point::new(
        Px(input_bounds.origin.x.0 + 5.0),
        Px(input_bounds.origin.y.0 + 5.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(input_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Ime(fret_core::ImeEvent::Preedit {
            text: "X".to_string(),
            cursor: Some((0, 1)),
        }),
    );
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after preedit");
    assert!(snapshot.focus_is_text_input);
    assert!(snapshot.is_composing);
    assert!(snapshot.marked_utf16.is_some());
    assert_eq!(
        snapshot.text_len_utf16,
        base_len_utf16 + 1,
        "expected composed text length to include the preedit"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 2,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after double click");
    assert!(snapshot.focus_is_text_input);
    assert!(!snapshot.is_composing);
    assert_eq!(snapshot.marked_utf16, None);
    assert_eq!(snapshot.text_len_utf16, base_len_utf16);
}

#[test]
fn text_input_triple_click_selects_logical_line_under_render_transform() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let text = "hello world".to_string();
    let text_len_utf16: u32 = text.encode_utf16().count().try_into().unwrap();
    let model = app.models_mut().insert(text);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(180.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let transform = Transform2D::translation(Point::new(Px(40.0), Px(10.0)));
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-triple-click-select-line-transform",
        |cx| {
            vec![cx.render_transform(transform, |cx| {
                let mut props = crate::element::TextInputProps::new(model.clone());
                props.layout.size.width = Length::Px(Px(160.0));
                props.layout.size.height = Length::Px(Px(32.0));
                vec![cx.text_input(props)]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let transform_node = ui.children(root)[0];
    let input_node = ui.children(transform_node)[0];
    let input_bounds = ui
        .debug_node_visual_bounds(input_node)
        .expect("input bounds");
    let pos = Point::new(
        Px(input_bounds.origin.x.0 + 5.0),
        Px(input_bounds.origin.y.0 + 5.0),
    );
    assert_eq!(
        ui.debug_hit_test(pos).hit,
        Some(input_node),
        "expected the translated hit-test position to target the text input"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(input_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 3,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(input_node));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot");
    assert!(snapshot.focus_is_text_input);
    assert_eq!(
        snapshot.selection_utf16,
        Some((0, text_len_utf16)),
        "triple click should select the logical line"
    );
}

#[test]
fn text_input_triple_click_selects_logical_line_under_scroll_offset() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let text = "hello world".to_string();
    let text_len_utf16: u32 = text.encode_utf16().count().try_into().unwrap();
    let model = app.models_mut().insert(text);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(180.0), Px(60.0)));
    let mut services = FakeTextService::default();
    let scroll_handle = crate::scroll::ScrollHandle::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-triple-click-select-line-scroll",
        |cx| {
            let mut scroll_layout = crate::element::LayoutStyle::default();
            scroll_layout.size.width = Length::Fill;
            scroll_layout.size.height = Length::Fill;
            scroll_layout.overflow = crate::element::Overflow::Clip;

            vec![cx.scroll(
                crate::element::ScrollProps {
                    layout: scroll_layout,
                    axis: crate::element::ScrollAxis::Y,
                    scroll_handle: Some(scroll_handle.clone()),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.column(
                        crate::element::ColumnProps {
                            gap: Px(0.0).into(),
                            ..Default::default()
                        },
                        |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();
                            for _ in 0..40 {
                                let mut row_layout = crate::element::LayoutStyle::default();
                                row_layout.size.height = Length::Px(Px(18.0));
                                out.push(cx.container(
                                    crate::element::ContainerProps {
                                        layout: row_layout,
                                        ..Default::default()
                                    },
                                    |cx| vec![cx.text("filler")],
                                ));
                            }

                            let mut props = crate::element::TextInputProps::new(model.clone());
                            props.layout.size.width = Length::Px(Px(120.0));
                            props.layout.size.height = Length::Px(Px(32.0));
                            out.push(cx.text_input(props));

                            out
                        },
                    )]
                },
            )]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll_node = ui.children(root)[0];
    let column_node = ui.children(scroll_node)[0];
    let input_node = *ui
        .children(column_node)
        .last()
        .expect("expected input as last child");
    let input_bounds = ui.debug_node_bounds(input_node).expect("input bounds");

    scroll_handle.set_offset(Point::new(Px(0.0), input_bounds.origin.y));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let input_bounds = ui
        .debug_node_visual_bounds(input_node)
        .expect("input bounds after scroll");
    let pos = Point::new(
        Px(input_bounds.origin.x.0 + 5.0),
        Px(input_bounds.origin.y.0 + 5.0),
    );
    assert_eq!(
        ui.debug_hit_test(pos).hit,
        Some(input_node),
        "expected the scrolled hit-test position to target the text input"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(input_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 3,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(input_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot");
    assert!(snapshot.focus_is_text_input);
    assert_eq!(
        snapshot.selection_utf16,
        Some((0, text_len_utf16)),
        "triple click should select the logical line"
    );
}

#[test]
fn text_input_triple_click_cancels_ime_preedit() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let text = "hello world".to_string();
    let base_len_utf16: u32 = text.encode_utf16().count().try_into().unwrap();
    let model = app.models_mut().insert(text);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(180.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "text-input-triple-click-cancels-ime-preedit",
        |cx| {
            let mut props = crate::element::TextInputProps::new(model.clone());
            props.layout.size.width = Length::Px(Px(160.0));
            props.layout.size.height = Length::Px(Px(32.0));
            vec![cx.text_input(props)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_node = ui.children(root)[0];
    let input_bounds = ui
        .debug_node_visual_bounds(input_node)
        .expect("input bounds");
    let pos = Point::new(
        Px(input_bounds.origin.x.0 + 5.0),
        Px(input_bounds.origin.y.0 + 5.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(input_node));
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Ime(fret_core::ImeEvent::Preedit {
            text: "X".to_string(),
            cursor: Some((0, 1)),
        }),
    );
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after preedit");
    assert!(snapshot.focus_is_text_input);
    assert!(snapshot.is_composing);
    assert!(snapshot.marked_utf16.is_some());
    assert_eq!(
        snapshot.text_len_utf16,
        base_len_utf16 + 1,
        "expected composed text length to include the preedit"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 3,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot after triple click");
    assert!(snapshot.focus_is_text_input);
    assert!(!snapshot.is_composing);
    assert_eq!(snapshot.marked_utf16, None);
    assert_eq!(snapshot.text_len_utf16, base_len_utf16);
    assert_eq!(snapshot.selection_utf16, Some((0, base_len_utf16)));
}

#[test]
fn text_input_semantics_controls_element_is_exposed() {
    use std::cell::Cell;

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert("hello".to_string());

    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "a11y-text-input-controls",
        |cx| {
            let listbox_id_out: Cell<Option<crate::elements::GlobalElementId>> = Cell::new(None);
            let listbox = cx.semantics_with_id(
                crate::element::SemanticsProps {
                    role: fret_core::SemanticsRole::ListBox,
                    test_id: Some(Arc::from("listbox")),
                    ..Default::default()
                },
                |_cx, id| {
                    listbox_id_out.set(Some(id));
                    Vec::new()
                },
            );

            let mut props = crate::element::TextInputProps::new(model.clone());
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;
            props.test_id = Some(Arc::from("combo"));
            props.a11y_role = Some(fret_core::SemanticsRole::ComboBox);
            props.controls_element = listbox_id_out.get().map(|id| id.0);

            vec![cx.text_input(props), listbox]
        },
    );
    ui.set_root(root);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let combo = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("combo"))
        .expect("expected combobox semantics node");
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("listbox"))
        .expect("expected listbox semantics node");

    assert!(
        combo.controls.contains(&listbox.id),
        "expected combobox to control the listbox"
    );
}
