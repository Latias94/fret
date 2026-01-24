//! Alert dialog helpers (Radix `@radix-ui/react-alert-dialog` outcomes).
//!
//! Upstream AlertDialog is a constrained Dialog variant:
//! - always modal,
//! - prevents outside interactions from dismissing,
//! - prefers focusing the `Cancel` action on open.
//!
//! In Fret, modal dismissal via outside press is modeled at the recipe layer (e.g. the overlay
//! barrier click handler). This module focuses on the Radix-specific focus preference: choosing
//! the cancel action as the default initial focus target when present.
//!
//! For parity with Radix `FocusScope`, alert dialogs also allow customizing open/close auto focus
//! via `AlertDialogOptions` (forwarded into `DialogOptions`).

use std::collections::HashMap;

use fret_runtime::Model;
use fret_runtime::ModelId;
use fret_ui::action::{OnCloseAutoFocus, OnOpenAutoFocus};
use fret_ui::element::{AnyElement, LayoutStyle};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::primitives::dialog as dialog_prim;
use crate::primitives::dialog::DialogOptions;
use crate::primitives::trigger_a11y;
use crate::{OverlayPresence, OverlayRequest};

/// Stable per-overlay root naming convention for alert dialogs.
pub fn alert_dialog_root_name(id: GlobalElementId) -> String {
    dialog_prim::dialog_root_name(id)
}

/// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
///
/// AlertDialog itself is a constrained Dialog variant. This helper exists to standardize how
/// recipes derive the open model (`open` / `defaultOpen`) before applying alert-dialog-specific
/// focus preferences.
pub fn alert_dialog_use_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled_open: Option<Model<bool>>,
    default_open: impl FnOnce() -> bool,
) -> crate::primitives::controllable_state::ControllableModel<bool> {
    crate::primitives::open_state::open_use_model(cx, controlled_open, default_open)
}

/// A Radix-shaped `AlertDialog` root configuration surface.
///
/// Upstream AlertDialog is a constrained Dialog variant: always modal, and prefers focusing the
/// cancel action by default. This root helper standardizes:
/// - controlled/uncontrolled open modeling (`open` / `defaultOpen`)
/// - initial focus preference (`PreferCancel` by default)
#[derive(Debug, Clone, Default)]
pub struct AlertDialogRoot {
    open: Option<Model<bool>>,
    default_open: bool,
    options: AlertDialogOptions,
}

impl AlertDialogRoot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the controlled `open` model (`Some`) or selects uncontrolled mode (`None`).
    pub fn open(mut self, open: Option<Model<bool>>) -> Self {
        self.open = open;
        self
    }

    /// Sets the uncontrolled initial open value (Radix `defaultOpen`).
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn initial_focus(mut self, initial_focus: AlertDialogInitialFocus) -> Self {
        self.options = self.options.initial_focus(initial_focus);
        self
    }

    pub fn options(&self) -> AlertDialogOptions {
        self.options.clone()
    }

    /// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
    pub fn use_open_model<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
    ) -> crate::primitives::controllable_state::ControllableModel<bool> {
        alert_dialog_use_open_model(cx, self.open.clone(), || self.default_open)
    }

    pub fn open_model<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Model<bool> {
        self.use_open_model(cx).model()
    }

    pub fn open_id<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> ModelId {
        self.open_model(cx).id()
    }

    pub fn is_open<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> bool {
        let open_model = self.open_model(cx);
        cx.watch_model(&open_model)
            .layout()
            .copied()
            .unwrap_or(false)
    }

    pub fn dialog_options<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> DialogOptions {
        let open_id = self.open_id(cx);
        dialog_options_for_alert_dialog(cx, open_id, self.options.clone())
    }

    pub fn modal_request<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        id: GlobalElementId,
        trigger: GlobalElementId,
        presence: OverlayPresence,
        children: Vec<AnyElement>,
    ) -> OverlayRequest {
        let open = self.open_model(cx);
        let options = self.dialog_options(cx);

        dialog_prim::modal_dialog_request_with_options(
            id, trigger, open, presence, options, children,
        )
    }
}

#[derive(Default)]
struct AlertDialogCancelRegistry {
    by_open: HashMap<ModelId, GlobalElementId>,
}

/// Records a `Cancel` element for the given open model id.
///
/// This is a best-effort mechanism: callers should re-register on each frame while the alert
/// dialog is open so stale entries are naturally overwritten.
pub fn register_cancel_for_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open_id: ModelId,
    element: GlobalElementId,
) {
    cx.app
        .with_global_mut_untracked(AlertDialogCancelRegistry::default, |reg, _app| {
            reg.by_open.entry(open_id).or_insert(element);
        });
}

/// Clears the stored cancel element for the given open model id.
pub fn clear_cancel_for_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>, open_id: ModelId) {
    cx.app
        .with_global_mut_untracked(AlertDialogCancelRegistry::default, |reg, _app| {
            reg.by_open.remove(&open_id);
        });
}

/// Returns the preferred initial focus element for this alert dialog (the registered cancel
/// action), if any.
pub fn cancel_element_for_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open_id: ModelId,
) -> Option<GlobalElementId> {
    cx.app
        .with_global_mut_untracked(AlertDialogCancelRegistry::default, |reg, _app| {
            reg.by_open.get(&open_id).copied()
        })
}

#[derive(Debug, Clone, Copy)]
pub enum AlertDialogInitialFocus {
    None,
    PreferCancel,
    Element(GlobalElementId),
}

impl Default for AlertDialogInitialFocus {
    fn default() -> Self {
        Self::PreferCancel
    }
}

#[derive(Clone)]
pub struct AlertDialogOptions {
    pub initial_focus: AlertDialogInitialFocus,
    pub on_open_auto_focus: Option<OnOpenAutoFocus>,
    pub on_close_auto_focus: Option<OnCloseAutoFocus>,
}

impl std::fmt::Debug for AlertDialogOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlertDialogOptions")
            .field("initial_focus", &self.initial_focus)
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .finish()
    }
}

impl Default for AlertDialogOptions {
    fn default() -> Self {
        Self {
            initial_focus: AlertDialogInitialFocus::default(),
            on_open_auto_focus: None,
            on_close_auto_focus: None,
        }
    }
}

impl AlertDialogOptions {
    pub fn initial_focus(mut self, initial_focus: AlertDialogInitialFocus) -> Self {
        self.initial_focus = initial_focus;
        self
    }

    pub fn on_open_auto_focus(mut self, hook: Option<OnOpenAutoFocus>) -> Self {
        self.on_open_auto_focus = hook;
        self
    }

    pub fn on_close_auto_focus(mut self, hook: Option<OnCloseAutoFocus>) -> Self {
        self.on_close_auto_focus = hook;
        self
    }
}

/// Converts alert-dialog options into dialog options (modal, non-dismissable by outside press).
pub fn dialog_options_for_alert_dialog<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open_id: ModelId,
    options: AlertDialogOptions,
) -> DialogOptions {
    let initial_focus = match options.initial_focus {
        AlertDialogInitialFocus::None => None,
        AlertDialogInitialFocus::Element(id) => Some(id),
        AlertDialogInitialFocus::PreferCancel => cancel_element_for_open_model(cx, open_id),
    };

    DialogOptions::default()
        .dismiss_on_overlay_press(false)
        .initial_focus(initial_focus)
        .on_open_auto_focus(options.on_open_auto_focus.clone())
        .on_close_auto_focus(options.on_close_auto_focus.clone())
}

/// Layout used for a Radix-like alert dialog modal barrier element.
///
/// This is a re-export of the shared modal barrier layout from `primitives::dialog`.
pub fn alert_dialog_modal_barrier_layout() -> LayoutStyle {
    dialog_prim::modal_barrier_layout()
}

/// Builds a Radix-style alert-dialog modal barrier (non-dismissable by outside press).
pub fn alert_dialog_modal_barrier<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    children: Vec<AnyElement>,
) -> AnyElement {
    dialog_prim::modal_barrier(cx, open, false, children)
}

/// Convenience helper to assemble alert dialog overlay children in a Radix-like order: barrier then
/// content.
pub fn alert_dialog_modal_layer_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    barrier_children: Vec<AnyElement>,
    content: AnyElement,
) -> Vec<AnyElement> {
    vec![
        alert_dialog_modal_barrier(cx, open, barrier_children),
        content,
    ]
}

/// Stamps Radix-like trigger relationships:
/// - `expanded` mirrors `aria-expanded`
/// - `controls_element` mirrors `aria-controls` (by element id).
pub fn apply_alert_dialog_trigger_a11y(
    trigger: AnyElement,
    expanded: bool,
    content_element: Option<GlobalElementId>,
) -> AnyElement {
    trigger_a11y::apply_trigger_controls_expanded(trigger, Some(expanded), content_element)
}

/// Builds an overlay request for a Radix-style modal alert dialog.
pub fn alert_dialog_modal_request_with_options(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    options: DialogOptions,
    children: Vec<AnyElement>,
) -> OverlayRequest {
    dialog_prim::modal_dialog_request_with_options(id, trigger, open, presence, options, children)
}

/// Requests a Radix-style modal alert dialog overlay for the current window.
pub fn request_alert_dialog<H: UiHost>(cx: &mut ElementContext<'_, H>, request: OverlayRequest) {
    dialog_prim::request_modal_dialog(cx, request);
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn alert_dialog_root_open_model_uses_controlled_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(true);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let root = AlertDialogRoot::new()
                .open(Some(controlled.clone()))
                .default_open(false);
            assert_eq!(root.open_model(cx), controlled);
        });
    }

    #[test]
    fn alert_dialog_root_options_builder_updates_options() {
        let root = AlertDialogRoot::new().initial_focus(AlertDialogInitialFocus::None);
        let options = root.options();
        assert!(matches!(
            options.initial_focus,
            AlertDialogInitialFocus::None
        ));
    }

    #[test]
    fn registry_prefers_first_cancel_and_can_be_cleared() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let open = app.models_mut().insert(false);
        let open_id = open.id();
        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            register_cancel_for_open_model(cx, open_id, GlobalElementId(0xaaa));
            register_cancel_for_open_model(cx, open_id, GlobalElementId(0xbbb));
            assert_eq!(
                cancel_element_for_open_model(cx, open_id),
                Some(GlobalElementId(0xaaa))
            );
            clear_cancel_for_open_model(cx, open_id);
            assert_eq!(cancel_element_for_open_model(cx, open_id), None);
        });
    }

    #[test]
    fn dialog_options_for_alert_dialog_prefers_cancel() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let open = app.models_mut().insert(false);
        let open_id = open.id();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            register_cancel_for_open_model(cx, open_id, GlobalElementId(0xaaa));

            let opts = dialog_options_for_alert_dialog(cx, open_id, AlertDialogOptions::default());
            assert!(!opts.dismiss_on_overlay_press);
            assert_eq!(opts.initial_focus, Some(GlobalElementId(0xaaa)));
        });
    }
}
