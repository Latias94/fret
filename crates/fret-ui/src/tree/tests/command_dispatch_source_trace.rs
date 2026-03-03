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
        cx.available
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
