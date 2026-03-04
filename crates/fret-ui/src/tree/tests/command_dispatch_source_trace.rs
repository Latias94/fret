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
