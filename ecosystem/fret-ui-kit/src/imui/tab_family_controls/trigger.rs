use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::{AnyElement, PressableA11y, PressableProps};
use fret_ui::{ElementContext, UiHost};

use super::{BuiltTabItem, visual};
use crate::imui::ResponseExt;
use crate::primitives::tabs;

mod behavior;

pub(super) struct BuiltTabTrigger {
    pub(super) element: AnyElement,
    pub(super) response: ResponseExt,
}

pub(super) fn render_tab_trigger<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    selected_model: &Model<Option<Arc<str>>>,
    item: &BuiltTabItem,
    selected: bool,
    pos_in_set: u32,
    set_size: u32,
) -> BuiltTabTrigger {
    let mut response = ResponseExt::default();
    let label = item.label.clone();
    let test_id = item.test_id.clone();
    let selected_model = selected_model.clone();
    let tab_id = item.id.clone();
    let enabled = item.enabled;
    let activate_shortcut = item.activate_shortcut;
    let shortcut_repeat = item.shortcut_repeat;

    let element = cx.keyed(("tab-trigger", item.id.clone()), |cx| {
        let response = &mut response;
        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled;
        props.a11y = PressableA11y {
            test_id: test_id.clone(),
            ..tabs::tab_a11y_with_collection(
                Some(label.clone()),
                selected,
                Some(pos_in_set),
                Some(set_size),
            )
        };

        cx.pressable_with_id(props, move |cx, state, element_id| {
            behavior::install_tab_trigger_behavior(
                cx,
                element_id,
                state,
                behavior::TabTriggerBehaviorInput {
                    selected_model: selected_model.clone(),
                    tab_id: tab_id.clone(),
                    enabled,
                    activate_shortcut,
                    shortcut_repeat,
                },
                response,
            );

            vec![visual::tab_trigger_visual(
                cx,
                label.clone(),
                selected,
                enabled,
                state,
            )]
        })
    });

    BuiltTabTrigger { element, response }
}
