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

    let root = ui.create_node(TestStack::default());
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

    let root = ui.create_node(TestStack::default());
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
