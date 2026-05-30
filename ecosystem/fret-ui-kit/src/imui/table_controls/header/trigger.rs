use std::hash::Hash;
use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{
    AnyElement, Length, PressableA11y, PressableKeyActivation, PressableProps, PressableState,
};
use fret_ui::{ElementContext, UiHost};

use crate::imui::{ResponseExt, imui_is_disabled};

pub(super) use visual::sortable_header_visual;

mod behavior;
mod visual;

pub(super) struct BuiltHeaderTrigger {
    pub(super) element: AnyElement,
    pub(super) trigger: ResponseExt,
}

pub(super) fn header_trigger_surface<H, K, F>(
    cx: &mut ElementContext<'_, H>,
    key: K,
    a11y_label: Option<Arc<str>>,
    activates_on_primary: bool,
    render: F,
) -> BuiltHeaderTrigger
where
    H: UiHost,
    K: Hash + Eq + Clone + 'static,
    F: Fn(&mut ElementContext<'_, H>, bool, PressableState) -> Vec<AnyElement> + 'static,
{
    let mut trigger = ResponseExt::default();
    let trigger_element = cx.keyed(key, |cx| {
        let trigger = &mut trigger;
        let enabled = !imui_is_disabled(cx);
        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled;
        props.layout.size.width = Length::Fill;
        props.layout.flex.grow = 1.0;
        props.layout.flex.shrink = 1.0;
        if !activates_on_primary {
            props.key_activation = PressableKeyActivation::None;
        }
        props.a11y = PressableA11y {
            role: Some(if activates_on_primary {
                SemanticsRole::Button
            } else {
                SemanticsRole::Group
            }),
            label: a11y_label.clone(),
            ..Default::default()
        };

        cx.pressable_with_id(props, move |cx, state, element_id| {
            behavior::install_header_trigger_behavior(
                cx,
                element_id,
                state,
                enabled,
                activates_on_primary,
                trigger,
            );

            render(cx, enabled, state)
        })
    });

    BuiltHeaderTrigger {
        element: trigger_element,
        trigger,
    }
}
