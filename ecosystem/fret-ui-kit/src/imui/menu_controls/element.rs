use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::ActionId;
use fret_ui::element::{AnyElement, PressableState};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::imui::{MenuItemOptions, ResponseExt};

use super::interaction;

mod visual_row;

pub(super) fn menu_item_element_with_pressable_hook_inner<H: UiHost, F>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    options: MenuItemOptions,
    role: SemanticsRole,
    checked: Option<bool>,
    action: Option<ActionId>,
    pressable_hook: F,
    response: &mut ResponseExt,
) -> AnyElement
where
    F: Clone
        + for<'cx> Fn(&mut fret_ui::ElementContext<'cx, H>, PressableState, GlobalElementId, bool),
{
    let pressable_hook = pressable_hook.clone();
    let visual_row =
        visual_row::MenuItemVisualRow::from_options(label.clone(), &options, role, checked);
    let interaction =
        interaction::resolve_menu_item_interaction(cx, &label, &options, role, checked, action);
    let runtime = interaction.runtime;

    cx.pressable_with_id(interaction.props, move |cx, state, id| {
        let pressable_hook = pressable_hook.clone();
        let behavior = interaction::install_menu_item_interaction(cx, id, &runtime);

        pressable_hook(cx, state, id, runtime.enabled);

        interaction::populate_menu_item_response(
            cx,
            id,
            state,
            &behavior,
            runtime.enabled,
            response,
        );

        let visuals = visual_row::render_menu_item_visual_row(cx, visual_row);

        vec![visuals]
    })
}
