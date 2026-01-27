//! Menu root helpers (Radix-aligned outcomes).
//!
//! In Radix, `MenuRoot` provides shared context for menu content and nested submenus.
//! In Fret, the "open/portal/overlay" concerns live in wrapper components (DropdownMenu, etc),
//! but we still centralize Menu-specific policy wiring here:
//! - ensuring submenu models exist within a menu root scope
//! - installing a timer handler for submenu focus/close delays
//! - producing a DismissableLayer pointer-move observer for submenu grace intent

use fret_ui::action::{
    OnCloseAutoFocus, OnDismissRequest, OnDismissiblePointerMove, OnOpenAutoFocus,
};
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use fret_runtime::Model;

use crate::primitives::dismissable_layer;
use crate::primitives::menu::sub;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

/// Menu initial focus targets (Radix `onOpenAutoFocus` outcomes).
///
/// When menu overlays open, Radix distinguishes between pointer-open and keyboard-open:
/// - Pointer-open: focus the content container and prevent “entry focus”.
/// - Keyboard-open: allow entry focus (typically the first enabled menu item).
///
/// In Fret, we encode this as a pair of optional element targets and choose between them based on
/// the last observed input modality (ADR 0095).
#[derive(Debug, Default, Clone, Copy)]
pub struct MenuInitialFocusTargets {
    pub keyboard_entry_focus: Option<GlobalElementId>,
    pub pointer_content_focus: Option<GlobalElementId>,
}

impl MenuInitialFocusTargets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn keyboard_entry_focus(mut self, id: Option<GlobalElementId>) -> Self {
        self.keyboard_entry_focus = id;
        self
    }

    pub fn pointer_content_focus(mut self, id: Option<GlobalElementId>) -> Self {
        self.pointer_content_focus = id;
        self
    }
}

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
        sub::handle_dismissible_pointer_move(host, acx, mv, &models, cfg)
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
    initial_focus: MenuInitialFocusTargets,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
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
        initial_focus,
        on_open_auto_focus,
        on_close_auto_focus,
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
    initial_focus: MenuInitialFocusTargets,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
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
        initial_focus,
        on_open_auto_focus,
        on_close_auto_focus,
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
    initial_focus: MenuInitialFocusTargets,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
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
        initial_focus,
        on_open_auto_focus,
        on_close_auto_focus,
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
    initial_focus: MenuInitialFocusTargets,
    on_open_auto_focus: Option<OnOpenAutoFocus>,
    on_close_auto_focus: Option<OnCloseAutoFocus>,
    on_dismiss_request: Option<OnDismissRequest>,
    dismissible_on_pointer_move: Option<OnDismissiblePointerMove>,
    modal: bool,
) -> OverlayRequest {
    let mut request = base_menu_overlay_request(id, trigger, open, presence, children, modal);
    request.root_name = Some(root_name);
    request.dismissible_on_dismiss_request = on_dismiss_request;
    request.dismissible_on_pointer_move = dismissible_on_pointer_move;
    request.on_open_auto_focus = on_open_auto_focus;
    request.on_close_auto_focus = on_close_auto_focus;

    let keyboard = fret_ui::input_modality::is_keyboard(cx.app, Some(cx.window));
    request.initial_focus = if keyboard {
        initial_focus.keyboard_entry_focus
    } else {
        initial_focus.pointer_content_focus
    };
    request
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, KeyCode, Modifiers, MouseButtons, Point, PointerEvent, PointerId,
        PointerType, Px, Rect, Size,
    };

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
        let handler: OnDismissRequest =
            Arc::new(|_host, _cx, _req: &mut fret_ui::action::DismissRequestCx| {});

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let req = dismissible_menu_request_with_modal_and_dismiss_handler(
                cx,
                GlobalElementId(1),
                GlobalElementId(2),
                open.clone(),
                OverlayPresence::hidden(),
                Vec::new(),
                "menu".to_string(),
                MenuInitialFocusTargets::new(),
                None,
                None,
                Some(handler.clone()),
                None,
                true,
            );
            assert!(req.dismissible_on_dismiss_request.is_some());
        });
    }

    #[test]
    fn menu_request_gates_initial_focus_by_modality() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let window = AppWindowId::default();
        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));

        let pointer_focus = GlobalElementId(0x111);
        let keyboard_focus = GlobalElementId(0x222);

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            // Pointer modality: choose pointer content focus.
            fret_ui::input_modality::update_for_event(
                cx.app,
                window,
                &Event::Pointer(PointerEvent::Move {
                    position: Point::new(Px(1.0), Px(2.0)),
                    buttons: MouseButtons::default(),
                    modifiers: Modifiers::default(),
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );

            let req = dismissible_menu_request(
                cx,
                GlobalElementId(1),
                GlobalElementId(2),
                open.clone(),
                OverlayPresence::hidden(),
                Vec::new(),
                "menu".to_string(),
                MenuInitialFocusTargets::new()
                    .pointer_content_focus(Some(pointer_focus))
                    .keyboard_entry_focus(Some(keyboard_focus)),
                None,
                None,
                None,
            );
            assert_eq!(req.initial_focus, Some(pointer_focus));

            // Keyboard modality: choose keyboard entry focus.
            fret_ui::input_modality::update_for_event(
                cx.app,
                window,
                &Event::KeyDown {
                    key: KeyCode::KeyA,
                    modifiers: Modifiers::default(),
                    repeat: false,
                },
            );
            let req = dismissible_menu_request(
                cx,
                GlobalElementId(1),
                GlobalElementId(2),
                open.clone(),
                OverlayPresence::hidden(),
                Vec::new(),
                "menu".to_string(),
                MenuInitialFocusTargets::new()
                    .pointer_content_focus(Some(pointer_focus))
                    .keyboard_entry_focus(Some(keyboard_focus)),
                None,
                None,
                None,
            );
            assert_eq!(req.initial_focus, Some(keyboard_focus));
        });
    }
}
