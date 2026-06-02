use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::BuiltTabItem;
use crate::imui::{TabBarOptions, TabTriggerResponse};

mod element;
mod triggers;

pub(super) struct BuiltTabList {
    pub(super) element: AnyElement,
    pub(super) selected_trigger_id: Option<GlobalElementId>,
    pub(super) first_focusable: Option<GlobalElementId>,
    pub(super) trigger_responses: Vec<TabTriggerResponse>,
}

pub(super) fn render_tab_list<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected_model: &Model<Option<Arc<str>>>,
    selected: Option<&str>,
    items: &[BuiltTabItem],
    options: &TabBarOptions,
) -> BuiltTabList {
    let built = triggers::build_tab_triggers(cx, selected_model, selected, items);
    let element = element::tab_list_element(cx, built.triggers, options);

    BuiltTabList {
        element,
        selected_trigger_id: built.selected_trigger_id,
        first_focusable: built.first_focusable,
        trigger_responses: built.trigger_responses,
    }
}
