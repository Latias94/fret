//! Menu root helpers (Radix-aligned outcomes).
//!
//! In Radix, `MenuRoot` provides shared context for menu content and nested submenus.
//! In Fret, the "open/portal/overlay" concerns live in wrapper components (DropdownMenu, etc),
//! but we still centralize Menu-specific policy wiring here:
//! - ensuring submenu models exist within a menu root scope
//! - installing a timer handler for submenu focus/close delays
//! - producing a DismissableLayer pointer-move observer for submenu grace intent

use fret_ui::action::OnDismissiblePointerMove;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::dismissable_layer;
use crate::primitives::menu::sub;

/// Ensure submenu models exist and install the menu-root timer handler.
///
/// Call this inside the overlay root scope (e.g. `cx.with_root_name(...)`), so the models are
/// scoped to that root.
pub fn ensure_submenu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    timer_handler_element: GlobalElementId,
) -> sub::MenuSubmenuModels {
    let models = sub::ensure_models(cx);
    sub::install_timer_handler(cx, timer_handler_element, models.clone());
    models
}

/// Build a DismissableLayer pointer-move observer that drives submenu grace intent.
pub fn submenu_pointer_move_handler(
    models: sub::MenuSubmenuModels,
    cfg: sub::MenuSubmenuConfig,
) -> OnDismissiblePointerMove {
    dismissable_layer::pointer_move_handler(move |host, acx, mv| {
        let _ = sub::handle_dismissible_pointer_move(host, acx, mv, &models, cfg);
        false
    })
}
