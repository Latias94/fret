use fret_app::{App, CommandId};
use fret_core::{Event, KeyCode, Modifiers, NodeId, UiServices};

use crate::UiTree;

pub fn invoke(ui: &mut UiTree, app: &mut App, services: &mut dyn UiServices, target: NodeId) {
    ui.set_focus(Some(target));
    ui.dispatch_event(
        app,
        services,
        &Event::KeyDown {
            key: KeyCode::Space,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );
    ui.dispatch_event(
        app,
        services,
        &Event::KeyUp {
            key: KeyCode::Space,
            modifiers: Modifiers::default(),
        },
    );
}

pub fn set_value_text(
    ui: &mut UiTree,
    app: &mut App,
    services: &mut dyn UiServices,
    target: NodeId,
    value: &str,
) {
    ui.set_focus(Some(target));
    let _ = ui.dispatch_command(app, services, &CommandId::from("edit.select_all"));
    ui.dispatch_event(app, services, &Event::TextInput(value.to_string()));
}

pub fn set_value_numeric(
    ui: &mut UiTree,
    app: &mut App,
    services: &mut dyn UiServices,
    target: NodeId,
    value: f64,
) {
    set_value_text(ui, app, services, target, &value.to_string());
}

pub fn set_text_selection(
    ui: &mut UiTree,
    app: &mut App,
    services: &mut dyn UiServices,
    target: NodeId,
    anchor: u32,
    focus: u32,
) {
    ui.set_focus(Some(target));
    ui.dispatch_event(app, services, &Event::SetTextSelection { anchor, focus });
}

pub fn replace_selected_text(
    ui: &mut UiTree,
    app: &mut App,
    services: &mut dyn UiServices,
    target: NodeId,
    value: &str,
) {
    ui.set_focus(Some(target));
    ui.dispatch_event(app, services, &Event::TextInput(value.to_string()));
}
