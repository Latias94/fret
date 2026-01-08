//! Menu root helpers (Radix-aligned outcomes).
//!
//! In Radix, `MenuRoot` provides shared context for menu content and nested submenus.
//! In Fret, the "open/portal/overlay" concerns live in wrapper components (DropdownMenu, etc),
//! but we still centralize Menu-specific policy wiring here:
//! - ensuring submenu models exist within a menu root scope
//! - installing a timer handler for submenu focus/close delays
//! - producing a DismissableLayer pointer-move observer for submenu grace intent

use fret_ui::action::OnDismissiblePointerMove;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use fret_runtime::Model;

use crate::primitives::dismissable_layer;
use crate::primitives::menu::sub;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

fn base_menu_overlay_request(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
    modal: bool,
) -> OverlayRequest {
    if modal {
        OverlayRequest::dismissible_menu(id, trigger, open, presence, children)
    } else {
        OverlayRequest::dismissible_popover(id, trigger, open, presence, children)
    }
}

/// A stable per-overlay root name for menu-like popovers.
///
/// This is the root naming convention used by shadcn menu wrappers (DropdownMenu, ContextMenu,
/// Menubar) and is safe to share as a Radix-aligned default.
pub fn menu_overlay_root_name(id: GlobalElementId) -> String {
    OverlayController::popover_root_name(id)
}

/// Ensure submenu models exist and install the menu-root timer handler.
///
/// Call this inside the overlay root scope (e.g. `cx.with_root_name(...)`), so the models are
/// scoped to that root.
pub fn ensure_submenu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    timer_handler_element: GlobalElementId,
    cfg: sub::MenuSubmenuConfig,
) -> sub::MenuSubmenuModels {
    let models = sub::ensure_models(cx);
    sub::install_timer_handler(cx, timer_handler_element, models.clone(), cfg);
    models
}

/// Sync root open state and ensure submenu models exist.
///
/// This is a convenience wrapper used by menu wrappers (`DropdownMenu`, `Menubar`, etc) so they
/// don't have to remember to call both `sub::sync_root_open` and `ensure_submenu` inside the
/// overlay root scope.
pub fn sync_root_open_and_ensure_submenu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    is_open: bool,
    timer_handler_element: GlobalElementId,
    cfg: sub::MenuSubmenuConfig,
) -> sub::MenuSubmenuModels {
    sub::sync_root_open(cx, is_open);
    ensure_submenu(cx, timer_handler_element, cfg)
}

/// Sync root open state and ensure submenu models exist inside a named overlay root.
#[track_caller]
pub fn with_root_name_sync_root_open_and_ensure_submenu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_name: &str,
    is_open: bool,
    cfg: sub::MenuSubmenuConfig,
) -> sub::MenuSubmenuModels {
    cx.with_root_name(root_name, |cx| {
        sync_root_open_and_ensure_submenu(cx, is_open, cx.root_id(), cfg)
    })
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

/// Build a shadcn/Radix-aligned menu overlay request.
///
/// Policy:
/// - Uses non-click-through outside press (`OverlayRequest::dismissible_menu`, ADR 0069).
/// - Gates initial focus by last input modality (ADR 0095):
///   - keyboard: allow entry focus (first focusable descendant)
///   - pointer: focus the content container and prevent entry focus
pub fn dismissible_menu_request<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
    root_name: String,
    content_focus: Option<GlobalElementId>,
    dismissible_on_pointer_move: Option<OnDismissiblePointerMove>,
) -> OverlayRequest {
    dismissible_menu_request_with_modal(
        cx,
        id,
        trigger,
        open,
        presence,
        children,
        root_name,
        content_focus,
        dismissible_on_pointer_move,
        true,
    )
}

/// Build a shadcn/Radix-aligned menu overlay request with explicit modal behavior.
///
/// In Radix, the `modal` flag controls `disableOutsidePointerEvents`. In Fret we approximate this
/// behavior by controlling whether outside-press dismissal consumes the pointer event (preventing
/// click-through on the underlay).
pub fn dismissible_menu_request_with_modal<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
    root_name: String,
    content_focus: Option<GlobalElementId>,
    dismissible_on_pointer_move: Option<OnDismissiblePointerMove>,
    modal: bool,
) -> OverlayRequest {
    let mut request = base_menu_overlay_request(id, trigger, open, presence, children, modal);
    request.root_name = Some(root_name);
    request.dismissible_on_pointer_move = dismissible_on_pointer_move;
    if !fret_ui::input_modality::is_keyboard(cx.app, Some(cx.window)) {
        request.initial_focus = content_focus;
    }
    request
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;

    #[test]
    fn menu_modal_controls_click_through_on_outside_press() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let req = base_menu_overlay_request(
            GlobalElementId(1),
            GlobalElementId(2),
            open.clone(),
            OverlayPresence::hidden(),
            Vec::new(),
            true,
        );
        assert!(req.consume_outside_pointer_events);

        let req = base_menu_overlay_request(
            GlobalElementId(1),
            GlobalElementId(2),
            open,
            OverlayPresence::hidden(),
            Vec::new(),
            false,
        );
        assert!(!req.consume_outside_pointer_events);
    }
}
