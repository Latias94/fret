use super::*;
struct KeyCounter {
    hits: fret_runtime::Model<u32>,
}

impl KeyCounter {
    fn new(hits: fret_runtime::Model<u32>) -> Self {
        Self { hits }
    }
}

impl<H: UiHost> Widget<H> for KeyCounter {
    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if matches!(event, Event::KeyDown { .. }) {
            let _ = cx
                .app
                .models_mut()
                .update(&self.hits, |v: &mut u32| *v += 1);
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn key_events_route_to_focus_barrier_root_when_unfocused() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let underlay_hits = app.models_mut().insert(0u32);
    let overlay_hits = app.models_mut().insert(0u32);

    let base_root = ui.create_node(KeyCounter::new(underlay_hits.clone()));
    ui.set_root(base_root);

    let overlay_root = ui.create_node(KeyCounter::new(overlay_hits.clone()));
    let overlay_layer = ui.push_overlay_root_with_options(
        overlay_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: false,
            hit_testable: false,
        },
    );
    ui.set_layer_blocks_underlay_focus(overlay_layer, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // With a focus barrier active and no focused node, key events must not route into the underlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::KeyA,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(
        app.models().get_copied(&underlay_hits).unwrap_or_default(),
        0,
        "expected underlay to not receive key events while a focus barrier is active"
    );
    assert_eq!(
        app.models().get_copied(&overlay_hits).unwrap_or_default(),
        1,
        "expected overlay barrier root to receive key events while unfocused"
    );
}

#[test]
fn shortcuts_use_focus_barrier_key_context_scope_when_input_barrier_is_off() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let overlay_command = CommandId::from("test.overlay_shortcut_scope");
    let underlay_command = CommandId::from("test.underlay_shortcut_scope");
    app.register_command(
        overlay_command.clone(),
        fret_runtime::CommandMeta::new("Overlay Shortcut Scope")
            .with_scope(fret_runtime::CommandScope::App),
    );
    app.register_command(
        underlay_command.clone(),
        fret_runtime::CommandMeta::new("Underlay Shortcut Scope")
            .with_scope(fret_runtime::CommandScope::App),
    );

    let mut keymap = Keymap::empty();
    keymap.push_binding(fret_runtime::keymap::Binding {
        platform: fret_runtime::PlatformFilter::All,
        sequence: vec![fret_runtime::KeyChord::new(
            fret_core::KeyCode::KeyK,
            fret_core::Modifiers::default(),
        )],
        when: Some(fret_runtime::WhenExpr::parse("keyctx.underlay").expect("underlay when expr")),
        command: Some(underlay_command.clone()),
    });
    keymap.push_binding(fret_runtime::keymap::Binding {
        platform: fret_runtime::PlatformFilter::All,
        sequence: vec![fret_runtime::KeyChord::new(
            fret_core::KeyCode::KeyK,
            fret_core::Modifiers::default(),
        )],
        when: Some(fret_runtime::WhenExpr::parse("keyctx.overlay").expect("overlay when expr")),
        command: Some(overlay_command.clone()),
    });
    app.set_global(KeymapService { keymap });

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let base_root = ui.create_node(TestStack);
    ui.set_root(base_root);

    let overlay_root = ui.create_node(TestStack);
    let overlay_layer = ui.push_overlay_root_with_options(
        overlay_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: false,
            hit_testable: false,
        },
    );
    ui.set_layer_blocks_underlay_focus(overlay_layer, true);

    crate::declarative::frame::with_window_frame_mut(&mut app, window, |window_frame| {
        window_frame.instances.insert(
            base_root,
            crate::declarative::frame::ElementRecord {
                element: crate::elements::GlobalElementId(1),
                instance: crate::declarative::frame::ElementInstance::Stack(
                    crate::element::StackProps::default(),
                ),
                inherited_foreground: None,
                inherited_text_style: None,
                semantics_decoration: None,
                key_context: Some(Arc::<str>::from("underlay")),
            },
        );
        window_frame.instances.insert(
            overlay_root,
            crate::declarative::frame::ElementRecord {
                element: crate::elements::GlobalElementId(2),
                instance: crate::declarative::frame::ElementInstance::Stack(
                    crate::element::StackProps::default(),
                ),
                inherited_foreground: None,
                inherited_text_style: None,
                semantics_decoration: None,
                key_context: Some(Arc::<str>::from("overlay")),
            },
        );
    });

    ui.publish_window_runtime_snapshots(&mut app);

    let initial_key_contexts = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    assert_eq!(initial_key_contexts, vec![Arc::<str>::from("overlay")]);

    let mut services = FakeUiServices;
    let _ = app.take_effects();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::KeyK,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, Effect::Command { command, .. } if command == &overlay_command)),
        "shortcut routing should resolve against the focus barrier key-context stack"
    );
    assert!(
        effects
            .iter()
            .all(|effect| !matches!(effect, Effect::Command { command, .. } if command == &underlay_command)),
        "shortcut routing must not fall back to the underlay key-context stack while a focus barrier is active"
    );

    let key_contexts_after_shortcut = app
        .global::<fret_runtime::WindowKeyContextStackService>()
        .and_then(|svc| svc.snapshot(window))
        .map(|v| v.to_vec())
        .unwrap_or_default();
    assert_eq!(
        key_contexts_after_shortcut,
        vec![Arc::<str>::from("overlay")],
        "dispatch-time key-context publication must stay aligned with the focus barrier scope when the shortcut path returns early"
    );
}
