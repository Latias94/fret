use super::*;
use crate::widget::{CommandAvailability, CommandAvailabilityCx};

#[derive(Debug)]
struct AvailabilityWidget {
    command: CommandId,
    result: CommandAvailability,
}

impl<H: UiHost> Widget<H> for AvailabilityWidget {
    fn command_availability(
        &self,
        _cx: &mut CommandAvailabilityCx<'_, H>,
        command: &CommandId,
    ) -> CommandAvailability {
        if command.as_str() == self.command.as_str() {
            self.result
        } else {
            CommandAvailability::NotHandled
        }
    }
}

#[test]
fn command_availability_walks_focus_chain() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let cmd = CommandId::from("test.cmd");

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let parent = ui.create_node(AvailabilityWidget {
        command: cmd.clone(),
        result: CommandAvailability::Available,
    });
    let child = ui.create_node(AvailabilityWidget {
        command: cmd.clone(),
        result: CommandAvailability::NotHandled,
    });
    ui.add_child(root, parent);
    ui.add_child(parent, child);
    ui.set_focus(Some(child));

    assert_eq!(
        ui.command_availability(&mut app, &cmd),
        CommandAvailability::Available
    );
    assert!(ui.is_command_available(&mut app, &cmd));
}

#[test]
fn command_availability_blocked_stops_walk() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let cmd = CommandId::from("test.cmd");

    let root = ui.create_node(TestStack);
    ui.set_root(root);

    let parent = ui.create_node(AvailabilityWidget {
        command: cmd.clone(),
        result: CommandAvailability::Available,
    });
    let child = ui.create_node(AvailabilityWidget {
        command: cmd.clone(),
        result: CommandAvailability::Blocked,
    });
    ui.add_child(root, parent);
    ui.add_child(parent, child);
    ui.set_focus(Some(child));

    assert_eq!(
        ui.command_availability(&mut app, &cmd),
        CommandAvailability::Blocked
    );
    assert!(!ui.is_command_available(&mut app, &cmd));
}

#[test]
fn command_availability_falls_back_to_default_root_when_focus_in_other_layer() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let cmd = CommandId::from("test.cmd");

    let base_root = ui.create_node(AvailabilityWidget {
        command: cmd.clone(),
        result: CommandAvailability::Available,
    });
    ui.set_root(base_root);

    let overlay_root = ui.create_node(TestStack);
    let overlay_leaf = ui.create_node(AvailabilityWidget {
        command: cmd.clone(),
        result: CommandAvailability::NotHandled,
    });
    ui.add_child(overlay_root, overlay_leaf);
    ui.push_overlay_root(overlay_root, false);
    ui.set_focus(Some(overlay_leaf));

    assert_eq!(
        ui.command_availability(&mut app, &cmd),
        CommandAvailability::Available
    );
}

#[test]
fn dispatch_command_falls_back_to_default_root_when_focus_in_other_layer() {
    #[derive(Debug)]
    struct HandleCommandWidget {
        command: CommandId,
        calls: Arc<AtomicUsize>,
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
            if command.as_str() != self.command.as_str() {
                return false;
            }
            self.calls.fetch_add(1, Ordering::SeqCst);
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

    let cmd = CommandId::from("test.cmd");
    let calls = Arc::new(AtomicUsize::new(0));

    let base_root = ui.create_node(HandleCommandWidget {
        command: cmd.clone(),
        calls: calls.clone(),
    });
    ui.set_root(base_root);

    let overlay_root = ui.create_node(TestStack);
    let overlay_leaf = ui.create_node(TestStack);
    ui.add_child(overlay_root, overlay_leaf);
    ui.push_overlay_root(overlay_root, false);
    ui.set_focus(Some(overlay_leaf));

    let mut services = FakeUiServices;

    assert!(ui.dispatch_command(&mut app, &mut services, &cmd));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
