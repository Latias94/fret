use std::sync::Arc;

use fret_runtime::KeyChord;
use fret_ui::UiHost;

use crate::imui::{ResponseExt, UiWriterImUiFacadeExt};

use super::super::menu_state::{self, BeginMenuState};

pub(in crate::imui::menu_family_controls) struct BeginMenuTriggerInput {
    pub(in crate::imui::menu_family_controls) label: Arc<str>,
    pub(in crate::imui::menu_family_controls) enabled: bool,
    pub(in crate::imui::menu_family_controls) test_id: Option<Arc<str>>,
    pub(in crate::imui::menu_family_controls) activate_shortcut: Option<KeyChord>,
    pub(in crate::imui::menu_family_controls) shortcut_repeat: bool,
}

pub(in crate::imui::menu_family_controls) struct BeginMenuTriggerFlow {
    pub(in crate::imui::menu_family_controls) trigger: ResponseExt,
    pub(in crate::imui::menu_family_controls) open_menu_before: Option<Arc<str>>,
}

pub(in crate::imui::menu_family_controls) fn run_begin_menu_trigger_flow<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
    input: BeginMenuTriggerInput,
) -> BeginMenuTriggerFlow {
    let trigger = ui.push_id(format!("{id}.trigger"), |ui| {
        super::super::trigger::menu_trigger_with_options(
            ui,
            Arc::from(id),
            input.label.clone(),
            state.open_before,
            state.row_open.clone(),
            state.menubar_policy.clone(),
            input.enabled,
            input.test_id.clone(),
            input.activate_shortcut,
            input.shortcut_repeat,
        )
    });

    let open_after_trigger = state.read_row_open(ui);
    menu_state::sync_open_menu_for_active_trigger(
        ui,
        id,
        state,
        open_after_trigger,
        trigger.clicked(),
        trigger.id(),
    );

    let open_menu_before = state.read_menubar_open_menu(ui);
    menu_state::reconcile_menubar_after_trigger(ui, id, state, open_after_trigger, trigger.id());

    if input.enabled && trigger.clicked() {
        menu_state::toggle_menu_on_trigger_click(ui, id, state);
    }

    BeginMenuTriggerFlow {
        trigger,
        open_menu_before,
    }
}
