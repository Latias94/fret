//! MenuSubContent helpers (Radix-aligned outcomes).
//!
//! Radix submenu content coordinates focus transfer:
//! - when a submenu opens via keyboard, focus moves into the submenu
//! - ArrowLeft closes the submenu and restores focus to its trigger
//!
//! These helpers keep wrappers from duplicating the per-item wiring.

use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::menu::sub;

/// Wire submenu-content behavior for a single submenu item.
///
/// Intended to be called inside the submenu panel pressable closure.
pub fn wire_item<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    item_id: GlobalElementId,
    disabled: bool,
    models: &sub::MenuSubmenuModels,
) {
    sub::focus_first_available_on_open(cx, models, item_id, disabled);
    cx.key_on_key_down_for(
        item_id,
        sub::submenu_item_arrow_left_handler(models.clone()),
    );
}
