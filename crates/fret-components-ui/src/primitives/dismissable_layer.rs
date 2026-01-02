//! DismissableLayer (Radix-aligned outcomes).
//!
//! In the DOM, Radix's DismissableLayer composes Escape and outside-interaction dismissal hooks.
//! In Fret, the runtime substrate provides those mechanisms via:
//!
//! - Escape routing: `fret-ui` event dispatch.
//! - Outside-press observer pass: ADR 0069 (observer phase pointer events).
//!
//! This module provides a stable, Radix-named primitive surface for component-layer policy.

use std::sync::Arc;

use fret_core::{AppWindowId, Rect, UiServices};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost, UiTree};

pub use fret_ui::action::{ActionCx, DismissReason, OnDismissRequest, UiActionHost};
pub use fret_ui::action::{OnDismissiblePointerMove, PointerMoveCx};

/// Render a full-window dismissable root that provides Escape + outside-press dismissal hooks.
///
/// This is a Radix-aligned naming alias for `render_dismissible_root_with_hooks`.
#[allow(clippy::too_many_arguments)]
pub fn render_dismissable_root_with_hooks<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> fret_core::NodeId {
    crate::declarative::dismissible::render_dismissible_root_with_hooks(
        ui, app, services, window, bounds, root_name, render,
    )
}

/// Installs an `on_dismiss_request` handler for the current dismissable root.
///
/// This is a naming-aligned wrapper around `ElementContext::dismissible_on_dismiss_request`.
pub fn on_dismiss_request<H: UiHost>(cx: &mut ElementContext<'_, H>, handler: OnDismissRequest) {
    cx.dismissible_on_dismiss_request(handler);
}

/// Installs an `on_pointer_move` observer for the current dismissable root.
///
/// This is intended for overlay policy code (e.g. submenu safe-hover corridors) that needs pointer
/// movement even when the overlay content is click-through.
pub fn on_pointer_move<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    handler: OnDismissiblePointerMove,
) {
    cx.dismissible_on_pointer_move(handler);
}

/// Convenience builder for an `OnDismissRequest` handler.
pub fn handler(
    f: impl Fn(&mut dyn UiActionHost, ActionCx, DismissReason) + 'static,
) -> OnDismissRequest {
    Arc::new(f)
}

/// Convenience builder for an `OnDismissiblePointerMove` handler.
pub fn pointer_move_handler(
    f: impl Fn(&mut dyn UiActionHost, ActionCx, PointerMoveCx) -> bool + 'static,
) -> OnDismissiblePointerMove {
    Arc::new(f)
}
