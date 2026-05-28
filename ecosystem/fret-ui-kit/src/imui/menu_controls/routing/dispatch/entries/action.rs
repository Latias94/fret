use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::ActionId;
use fret_ui::UiHost;

use crate::imui::{MenuItemOptions, ResponseExt, UiWriterImUiFacadeExt};

pub(in crate::imui) fn menu_item_action_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    action: ActionId,
    options: MenuItemOptions,
) -> ResponseExt {
    super::menu_item_impl(
        ui,
        label,
        options,
        SemanticsRole::MenuItem,
        None,
        Some(action),
    )
}
