use fret_runtime::{KeyChord, Model};
use fret_ui::action::{ActivateReason, UiActionHostExt as _};
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::{KEY_CLICKED, ResponseExt, item_behavior, mark_lifecycle_instant_if_inactive};
use super::keyboard::{SelectableKeyboardOptions, install_selectable_keyboard};

pub(super) struct SelectableBehaviorOptions {
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) close_popup: Option<Model<bool>>,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_selectable_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    options: SelectableBehaviorOptions,
    response: &mut ResponseExt,
) {
    let behavior = item_behavior::install_pressable_item_behavior_with_options(
        cx,
        id,
        item_behavior::PressableItemBehaviorOptions {
            report_pointer_click: true,
        },
    );
    let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

    if options.enabled {
        let close_popup_for_activate = options.close_popup.clone();
        cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
            if reason == ActivateReason::Keyboard {
                mark_lifecycle_instant_if_inactive(host, acx, &lifecycle_model_for_activate, false);
            }
            if let Some(open) = close_popup_for_activate.as_ref() {
                let _ = host.update_model(open, |v| *v = false);
            }
            host.record_transient_event(acx, KEY_CLICKED);
            host.notify(acx);
        }));

        install_selectable_keyboard(
            cx,
            id,
            options.focusable,
            behavior.lifecycle_model.clone(),
            SelectableKeyboardOptions {
                close_popup: options.close_popup.clone(),
                activate_shortcut: options.activate_shortcut,
                shortcut_repeat: options.shortcut_repeat,
            },
        );
    }

    let clicked = cx.take_transient_for(id, KEY_CLICKED);
    item_behavior::populate_pressable_item_response(
        cx,
        id,
        state,
        &behavior,
        item_behavior::PressableItemResponseInput {
            enabled: options.enabled,
            clicked,
            changed: false,
            lifecycle_edited: false,
        },
        response,
    );
}
