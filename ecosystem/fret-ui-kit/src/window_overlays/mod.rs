//! Window-scoped overlay manager (policy layer).
//!
//! This is a small component-layer orchestration helper that installs `UiTree` overlay roots
//! (ADR 0067) and coordinates dismissal + focus restore rules (ADR 0069).

mod frame;
mod names;
mod render;
mod requests;
mod state;
mod toast;

#[cfg(test)]
mod tests;

pub use frame::{
    begin_frame, request_dismissible_popover_for_window, request_hover_overlay_for_window,
    request_modal_for_window, request_toast_layer_for_window, request_tooltip_for_window,
};

#[cfg(feature = "unstable-internals")]
pub use frame::{
    request_dismissible_popover, request_hover_overlay, request_modal, request_toast_layer,
    request_tooltip,
};
pub use names::{
    hover_overlay_root_name, modal_root_name, popover_root_name, toast_layer_root_name,
    tooltip_root_name,
};
pub use render::render;
pub use requests::{
    DismissiblePopoverRequest, HoverOverlayRequest, ModalRequest, ToastLayerRequest, TooltipRequest,
};
pub use toast::{
    DEFAULT_MAX_TOASTS, ToastAction, ToastId, ToastPosition, ToastRequest, ToastStore,
    ToastVariant, dismiss_toast_action, toast_action, toast_store,
};
