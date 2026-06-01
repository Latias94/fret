use std::sync::Arc;

use fret_ui::UiHost;

use super::super::{ResponseExt, SelectableOptions, UiWriterImUiFacadeExt};

pub(in crate::imui::selectable_controls) fn selectable_with_visible_label<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    options: SelectableOptions,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::super::imui_is_disabled(cx);
        let focusable = enabled && options.focusable;
        let selected = options.selected;
        let highlighted = enabled && options.highlighted;
        let close_popup = options.close_popup.clone();
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let props = super::props::selectable_pressable_props(
            &label, &options, enabled, focusable, selected,
        );

        cx.pressable_with_id(props, move |cx, state, id| {
            super::behavior::install_selectable_behavior(
                cx,
                id,
                state,
                super::behavior::SelectableBehaviorOptions {
                    enabled,
                    focusable,
                    close_popup: close_popup.clone(),
                    activate_shortcut,
                    shortcut_repeat,
                },
                response,
            );

            vec![super::visual::selectable_row_element(
                cx,
                label.clone(),
                enabled,
                selected,
                highlighted,
                state,
            )]
        })
    });

    ui.add(element);
    response
}
