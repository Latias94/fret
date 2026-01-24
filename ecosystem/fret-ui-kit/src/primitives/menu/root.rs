//! Menu root helpers (Radix-aligned outcomes).
//!
//! In Radix, `MenuRoot` provides shared context for menu content and nested submenus.
//! In Fret, the "open/portal/overlay" concerns live in wrapper components (DropdownMenu, etc),
//! but we still centralize Menu-specific policy wiring here:
//! - ensuring submenu models exist within a menu root scope
//! - installing a timer handler for submenu focus/close delays
//! - producing a DismissableLayer pointer-move observer for submenu grace intent

use fret_ui::action::{OnDismissRequest, OnDismissiblePointerMove};
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
    // Radix menu-like overlays can be "modal" (the default) or non-modal.
    //
    // In practice this controls whether outside pointer interactions are allowed while the menu is
    // open:
    // - modal: outside pointer events are blocked and outside presses are not click-through.
    // - non-modal: outside presses are click-through (the underlay can receive the click).
    let mut req = OverlayRequest::dismissible_popover(id, trigger, open, presence, children);
    req.consume_outside_pointer_events = modal;
    req.disable_outside_pointer_events = modal;
    req
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
    let models = sub::ensure_models_for(cx, timer_handler_element);
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
    sub::sync_root_open_for(cx, timer_handler_element, is_open);
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

/// Build a shadcn/Radix-aligned menu overlay request that routes dismissals through an optional
/// dismiss handler (Radix `DismissableLayer` "preventDefault" outcome).
pub fn dismissible_menu_request_with_dismiss_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
    root_name: String,
    content_focus: Option<GlobalElementId>,
    on_dismiss_request: Option<OnDismissRequest>,
    dismissible_on_pointer_move: Option<OnDismissiblePointerMove>,
) -> OverlayRequest {
    dismissible_menu_request_with_modal_and_dismiss_handler(
        cx,
        id,
        trigger,
        open,
        presence,
        children,
        root_name,
        content_focus,
        on_dismiss_request,
        dismissible_on_pointer_move,
        true,
    )
}

/// Build a shadcn/Radix-aligned menu overlay request with explicit modal behavior.
///
/// In Radix, the `modal` flag controls `disableOutsidePointerEvents`. In Fret we approximate this
/// behavior by:
/// - blocking underlay pointer interaction while open, and
/// - controlling whether outside-press dismissal is click-through.
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
    dismissible_menu_request_with_modal_and_dismiss_handler(
        cx,
        id,
        trigger,
        open,
        presence,
        children,
        root_name,
        content_focus,
        None,
        dismissible_on_pointer_move,
        modal,
    )
}

/// Build a shadcn/Radix-aligned menu overlay request with explicit modal behavior and an optional
/// dismiss handler.
pub fn dismissible_menu_request_with_modal_and_dismiss_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
    root_name: String,
    content_focus: Option<GlobalElementId>,
    on_dismiss_request: Option<OnDismissRequest>,
    dismissible_on_pointer_move: Option<OnDismissiblePointerMove>,
    modal: bool,
) -> OverlayRequest {
    let mut request = base_menu_overlay_request(id, trigger, open, presence, children, modal);
    request.root_name = Some(root_name);
    request.dismissible_on_dismiss_request = on_dismiss_request;
    request.dismissible_on_pointer_move = dismissible_on_pointer_move;
    if !fret_ui::input_modality::is_keyboard(cx.app, Some(cx.window)) {
        request.initial_focus = content_focus;
    }
    request
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::action::DismissReason;

    #[test]
    fn menu_modal_controls_underlay_pointer_blocking_and_click_through() {
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
        assert!(req.disable_outside_pointer_events);

        let req = base_menu_overlay_request(
            GlobalElementId(1),
            GlobalElementId(2),
            open,
            OverlayPresence::hidden(),
            Vec::new(),
            false,
        );
        assert!(!req.consume_outside_pointer_events);
        assert!(!req.disable_outside_pointer_events);
    }

    #[test]
    fn menu_request_can_install_dismiss_handler() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let handler: OnDismissRequest = Arc::new(|_host, _cx, _reason: DismissReason| {});

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let req = dismissible_menu_request_with_modal_and_dismiss_handler(
                cx,
                GlobalElementId(1),
                GlobalElementId(2),
                open.clone(),
                OverlayPresence::hidden(),
                Vec::new(),
                "menu".to_string(),
                None,
                Some(handler.clone()),
                None,
                true,
            );
            assert!(req.dismissible_on_dismiss_request.is_some());
        });
    }
}
