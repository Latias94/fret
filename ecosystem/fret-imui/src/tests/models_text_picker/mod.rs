use super::*;
use fret_ui_kit::imui::InputTextOptions;
use fret_ui_kit::imui::InputTextPickerFilter;
use fret_ui_kit::imui::InputTextPickerOptions;

fn picker_option_active(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
    test_id: &str,
) -> bool {
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let option = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .unwrap_or_else(|| panic!("expected semantics node with test_id {test_id:?}"));
    snap.nodes
        .iter()
        .any(|node| node.active_descendant == Some(option.id))
}

mod completion_keyboard;
mod completion_popup;
mod empty_keyboard;
mod history_keyboard;
mod history_popup;
