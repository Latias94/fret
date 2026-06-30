use super::*;
use crate::widget::{CommandAvailability, CommandAvailabilityCx};

#[derive(Debug)]
struct HandleCommandWidget {
    command: CommandId,
}

impl<H: UiHost> Widget<H> for HandleCommandWidget {
    fn command_availability(
        &self,
        _cx: &mut CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> CommandAvailability {
        if command.as_str() == self.command.as_str() {
            CommandAvailability::Available
        } else {
            CommandAvailability::NotHandled
        }
    }

    fn command(&mut self, _cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
        command.as_str() == self.command.as_str()
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}

#[test]
fn dispatch_command_records_programmatic_source_by_default() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let cmd = CommandId::from("test.cmd");

    let root = ui.create_node(HandleCommandWidget {
        command: cmd.clone(),
    });
    ui.set_root(root);
    ui.set_focus(Some(root));

    let mut services = FakeUiServices;
    assert!(ui.dispatch_command(&mut app, &mut services, &cmd));

    let store = app
        .global::<fret_runtime::WindowCommandDispatchDiagnosticsStore>()
        .expect("dispatch must record diagnostics");
    let decisions = store.snapshot_since(window, 0, 10);
    let last = decisions.last().expect("expected at least one decision");

    assert_eq!(last.command.as_str(), cmd.as_str());
    assert_eq!(
        last.source.kind,
        fret_runtime::CommandDispatchSourceKindV1::Programmatic
    );
    assert_eq!(last.source.element, None);
}

#[test]
fn dispatch_command_consumes_pending_pointer_source_metadata() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let cmd = CommandId::from("test.cmd");

    let root = ui.create_node(HandleCommandWidget {
        command: cmd.clone(),
    });
    ui.set_root(root);
    ui.set_focus(Some(root));

    let expected_element = 42u64;
    app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |svc, app| {
            svc.record(
                window,
                app.tick_id(),
                cmd.clone(),
                fret_runtime::CommandDispatchSourceV1 {
                    kind: fret_runtime::CommandDispatchSourceKindV1::Pointer,
                    element: Some(expected_element),
                    test_id: None,
                },
            );
        },
    );

    let mut services = FakeUiServices;
    assert!(ui.dispatch_command(&mut app, &mut services, &cmd));

    let store = app
        .global::<fret_runtime::WindowCommandDispatchDiagnosticsStore>()
        .expect("dispatch must record diagnostics");
    let decisions = store.snapshot_since(window, 0, 10);
    let last = decisions.last().expect("expected at least one decision");

    assert_eq!(last.command.as_str(), cmd.as_str());
    assert_eq!(
        last.source.kind,
        fret_runtime::CommandDispatchSourceKindV1::Pointer
    );
    assert_eq!(last.source.element, Some(expected_element));

    let consumed = app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |svc, app| svc.consume(window, app.tick_id(), &cmd),
    );
    assert_eq!(consumed, None);
}

#[test]
fn dispatch_command_bubbles_from_pending_source_element_when_focus_is_none() {
    use crate::elements::NodeEntry;

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let cmd = CommandId::from("test.cmd");

    let root = ui.create_node(HandleCommandWidget {
        command: CommandId::from("test.root"),
    });
    let handler = ui.create_node(HandleCommandWidget {
        command: cmd.clone(),
    });
    let source_element = crate::elements::GlobalElementId(42);
    let leaf = ui.create_node_for_element(
        source_element,
        HandleCommandWidget {
            command: CommandId::from("test.leaf"),
        },
    );
    ui.add_child(root, handler);
    ui.add_child(handler, leaf);
    ui.set_root(root);
    ui.set_focus(None);

    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            source_element,
            NodeEntry {
                node: leaf,
                last_seen_frame: frame_id,
                root: source_element,
            },
        );
    });

    app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |svc, app| {
            svc.record(
                window,
                app.tick_id(),
                cmd.clone(),
                fret_runtime::CommandDispatchSourceV1 {
                    kind: fret_runtime::CommandDispatchSourceKindV1::Pointer,
                    element: Some(source_element.0),
                    test_id: None,
                },
            );
        },
    );

    let mut services = FakeUiServices;
    assert!(
        ui.dispatch_command(&mut app, &mut services, &cmd),
        "expected pending-source element bubbling to reach an ancestor handler"
    );
}

#[test]
fn dispatch_command_prefers_pending_source_element_over_stale_focus() {
    use crate::elements::NodeEntry;

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let cmd = CommandId::from("test.cmd");

    let root = ui.create_node(HandleCommandWidget {
        command: CommandId::from("test.root"),
    });
    let stale_focus = ui.create_node(HandleCommandWidget {
        command: CommandId::from("test.stale_focus"),
    });
    let handler = ui.create_node(HandleCommandWidget {
        command: cmd.clone(),
    });
    let source_element = crate::elements::GlobalElementId(42);
    let leaf = ui.create_node_for_element(
        source_element,
        HandleCommandWidget {
            command: CommandId::from("test.leaf"),
        },
    );
    ui.add_child(root, stale_focus);
    ui.add_child(root, handler);
    ui.add_child(handler, leaf);
    ui.set_root(root);
    ui.set_focus(Some(stale_focus));

    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            source_element,
            NodeEntry {
                node: leaf,
                last_seen_frame: frame_id,
                root: source_element,
            },
        );
    });

    app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |svc, app| {
            svc.record(
                window,
                app.tick_id(),
                cmd.clone(),
                fret_runtime::CommandDispatchSourceV1 {
                    kind: fret_runtime::CommandDispatchSourceKindV1::Pointer,
                    element: Some(source_element.0),
                    test_id: None,
                },
            );
        },
    );

    let mut services = FakeUiServices;
    assert!(
        ui.dispatch_command(&mut app, &mut services, &cmd),
        "expected pointer-triggered command dispatch to prefer the pending source element over stale focus"
    );
}

#[test]
fn dispatch_command_source_element_ignores_stale_detached_node_entry() {
    use crate::elements::NodeEntry;

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let cmd = CommandId::from("test.cmd");

    let root = ui.create_node(HandleCommandWidget {
        command: CommandId::from("test.root"),
    });
    let handler = ui.create_node(HandleCommandWidget {
        command: cmd.clone(),
    });
    let source_element = crate::elements::GlobalElementId(4242);
    let live_leaf = ui.create_node_for_element(
        source_element,
        HandleCommandWidget {
            command: CommandId::from("test.live"),
        },
    );
    let stale_detached = ui.create_node_for_element(
        source_element,
        HandleCommandWidget {
            command: CommandId::from("test.stale"),
        },
    );
    ui.add_child(root, handler);
    ui.add_child(handler, live_leaf);
    ui.set_root(root);
    ui.set_focus(None);

    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            source_element,
            NodeEntry {
                node: stale_detached,
                last_seen_frame: frame_id,
                root: source_element,
            },
        );
    });

    app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |svc, app| {
            svc.record(
                window,
                app.tick_id(),
                cmd.clone(),
                fret_runtime::CommandDispatchSourceV1 {
                    kind: fret_runtime::CommandDispatchSourceKindV1::Pointer,
                    element: Some(source_element.0),
                    test_id: None,
                },
            );
        },
    );

    let mut services = FakeUiServices;
    assert!(
        ui.dispatch_command(&mut app, &mut services, &cmd),
        "expected pending-source command dispatch to fall back from a stale detached node_entry to the live attached element node"
    );
}

#[test]
fn dispatch_command_bubble_uses_dispatch_snapshot_parent_not_retained_parent() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let cmd = CommandId::from("test.cmd");

    let root = ui.create_node(HandleCommandWidget {
        command: CommandId::from("test.root"),
    });
    let leaf = ui.create_node(HandleCommandWidget {
        command: CommandId::from("test.leaf"),
    });
    let detached_handler_element = crate::elements::GlobalElementId(0xD17A);
    let detached_handler = ui.create_node_for_element(
        detached_handler_element,
        HandleCommandWidget {
            command: cmd.clone(),
        },
    );

    ui.set_root(root);
    ui.add_child(root, leaf);
    ui.set_focus(Some(leaf));

    let (_active_roots, barrier_root) = ui.active_input_layers();
    let (active_focus_roots, focus_barrier_root) = ui.active_focus_layers();
    let barrier_root = focus_barrier_root.or(barrier_root);
    let snapshot = ui.cached_dispatch_snapshot_for_layer_roots(
        app.frame_id(),
        &active_focus_roots,
        barrier_root,
    );
    assert!(snapshot.pre.get(leaf).is_some());
    assert!(snapshot.pre.get(detached_handler).is_none());

    ui.test_set_node_parent(leaf, Some(detached_handler));

    let mut services = FakeUiServices;
    assert!(
        !ui.dispatch_command(&mut app, &mut services, &cmd),
        "command dispatch must not bubble through retained parents that are outside the dispatch snapshot"
    );

    let store = app
        .global::<fret_runtime::WindowCommandDispatchDiagnosticsStore>()
        .expect("dispatch must record diagnostics");
    let decisions = store.snapshot_since(window, 0, 10);
    let last = decisions.last().expect("expected at least one decision");
    assert!(!last.handled);
    assert_eq!(last.handled_by_element, None);
}

#[test]
fn dispatch_command_ignores_pending_source_node_outside_dispatch_snapshot() {
    use crate::elements::NodeEntry;

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let cmd = CommandId::from("test.cmd");
    let source_element = crate::elements::GlobalElementId(0x515E);

    let underlay_source = ui.create_node_for_element(
        source_element,
        HandleCommandWidget {
            command: cmd.clone(),
        },
    );
    ui.set_root(underlay_source);

    let overlay_root = ui.create_node(HandleCommandWidget {
        command: CommandId::from("test.overlay"),
    });
    let overlay_layer = ui.push_overlay_root_with_options(
        overlay_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: false,
            hit_testable: false,
        },
    );
    ui.set_layer_blocks_underlay_focus(overlay_layer, true);
    ui.set_focus(None);

    let frame_id = app.frame_id();
    crate::elements::with_window_state(&mut app, window, |st| {
        st.set_node_entry(
            source_element,
            NodeEntry {
                node: underlay_source,
                last_seen_frame: frame_id,
                root: source_element,
            },
        );
    });

    app.with_global_mut(
        fret_runtime::WindowPendingCommandDispatchSourceService::default,
        |svc, app| {
            svc.record(
                window,
                app.tick_id(),
                cmd.clone(),
                fret_runtime::CommandDispatchSourceV1 {
                    kind: fret_runtime::CommandDispatchSourceKindV1::Pointer,
                    element: Some(source_element.0),
                    test_id: None,
                },
            );
        },
    );

    let mut services = FakeUiServices;
    assert!(
        !ui.dispatch_command(&mut app, &mut services, &cmd),
        "pending source elements outside the current dispatch snapshot must not route commands into an inactive underlay"
    );

    let store = app
        .global::<fret_runtime::WindowCommandDispatchDiagnosticsStore>()
        .expect("dispatch must record diagnostics");
    let decisions = store.snapshot_since(window, 0, 10);
    let last = decisions.last().expect("expected at least one decision");
    assert!(!last.handled);
    assert_eq!(last.source.element, Some(source_element.0));
    assert_eq!(last.handled_by_element, None);
}
