use std::sync::Arc;

use fret_ui::UiHost;

use crate::imui::label_identity::parse_label_identity;
use crate::imui::{ImUiFacade, UiWriterImUiFacadeExt};

pub(super) fn with_menu_item_label_identity<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized, R>(
    ui: &mut W,
    label: Arc<str>,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>, Arc<str>) -> R,
) -> R {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("menu-item-label", identity), |ui| f(ui, visible_label))
}
