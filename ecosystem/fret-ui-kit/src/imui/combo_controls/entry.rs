use std::sync::Arc;

use fret_ui::UiHost;

use super::super::label_identity::parse_label_identity;
use super::super::{ComboOptions, ComboResponse, ImUiFacade, UiWriterImUiFacadeExt};
use super::{state, trigger};

pub(super) fn combo_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    preview: Arc<str>,
    options: ComboOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> ComboResponse {
    let parts = parse_label_identity(label.as_ref());
    let label = Arc::<str>::from(parts.visible);
    let enabled = state::combo_enabled(ui, options.enabled);
    let open_before = state::combo_popup_open(ui, id);
    let trigger_options = trigger::ComboTriggerOptions {
        enabled,
        focusable: options.focusable,
        a11y_label: options.a11y_label.clone(),
        test_id: options.test_id.clone(),
        activate_shortcut: options.activate_shortcut,
        shortcut_repeat: options.shortcut_repeat,
        open: open_before,
    };
    let popup_options = options.popup;

    let mut trigger = trigger::combo_trigger(ui, id, label, preview, trigger_options);

    state::toggle_popup_from_trigger(ui, id, enabled, open_before, &trigger);

    let popup_opened = super::super::popup_overlay::begin_popup_menu_with_options(
        ui,
        id,
        trigger.id(),
        popup_options,
        false,
        f,
    );
    state::close_disabled_popup(ui, id, enabled, popup_opened);

    let open_after = state::combo_popup_open(ui, id);
    let toggled = state::combo_toggled(ui, &trigger, open_after);
    state::apply_trigger_open_response(&mut trigger, toggled, open_after);

    ComboResponse {
        trigger,
        open: open_after,
        toggled,
    }
}
