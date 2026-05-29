use fret_core::{Px, Rect, Size};
use fret_ui::UiHost;

use super::super::{ImUiFacade, PopupMenuOptions, ResponseExt, UiWriterImUiFacadeExt};

pub(in crate::imui) fn begin_popup_context_menu_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    trigger: ResponseExt,
    options: PopupMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    if trigger.context_menu_requested() {
        let anchor = trigger
            .context_menu_anchor()
            .map(|p| Rect::new(p, Size::new(Px(1.0), Px(1.0))))
            .or(trigger.rect());
        if let Some(anchor) = anchor {
            super::state::open_popup_at(ui, id, anchor);
        }
    }

    super::begin_popup_menu_with_options(ui, id, trigger.id(), options, false, f)
}
