use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::BuiltTabItem;
use crate::imui::TabTriggerResponse;

pub(super) struct BuiltTabTriggers {
    pub(super) triggers: Vec<AnyElement>,
    pub(super) selected_trigger_id: Option<GlobalElementId>,
    pub(super) first_focusable: Option<GlobalElementId>,
    pub(super) trigger_responses: Vec<TabTriggerResponse>,
}

pub(super) fn build_tab_triggers<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected_model: &Model<Option<Arc<str>>>,
    selected: Option<&str>,
    items: &[BuiltTabItem],
) -> BuiltTabTriggers {
    let set_size = items.len().min(u32::MAX as usize) as u32;
    let mut selected_trigger_id = None;
    let mut first_focusable = None;
    let mut trigger_responses = Vec::with_capacity(items.len());

    let triggers = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let is_selected = selected == Some(item.id.as_ref());
            let built = super::super::super::trigger::render_tab_trigger(
                cx,
                selected_model,
                item,
                is_selected,
                index.min(u32::MAX as usize - 1) as u32 + 1,
                set_size,
            );
            if first_focusable.is_none() && item.enabled {
                first_focusable = built.response.id();
            }
            if is_selected {
                selected_trigger_id = built.response.id();
            }
            trigger_responses.push(TabTriggerResponse {
                id: item.id.clone(),
                selected: is_selected,
                trigger: built.response,
            });
            built.element
        })
        .collect::<Vec<_>>();

    BuiltTabTriggers {
        triggers,
        selected_trigger_id,
        first_focusable,
        trigger_responses,
    }
}
