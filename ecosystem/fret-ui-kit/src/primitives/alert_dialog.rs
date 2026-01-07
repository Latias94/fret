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

use std::collections::HashMap;

use fret_runtime::Model;
use fret_runtime::ModelId;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::primitives::dialog as dialog_prim;
use crate::primitives::dialog::DialogOptions;
use crate::{OverlayPresence, OverlayRequest};

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
        self.options
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
        dialog_options_for_alert_dialog(cx, open_id, self.options)
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
        .with_global_mut(AlertDialogCancelRegistry::default, |reg, _app| {
            reg.by_open.entry(open_id).or_insert(element);
        });
}

/// Clears the stored cancel element for the given open model id.
pub fn clear_cancel_for_open_model<H: UiHost>(cx: &mut ElementContext<'_, H>, open_id: ModelId) {
    cx.app
        .with_global_mut(AlertDialogCancelRegistry::default, |reg, _app| {
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
        .with_global_mut(AlertDialogCancelRegistry::default, |reg, _app| {
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

#[derive(Debug, Clone, Copy, Default)]
pub struct AlertDialogOptions {
    pub initial_focus: AlertDialogInitialFocus,
}

impl AlertDialogOptions {
    pub fn initial_focus(mut self, initial_focus: AlertDialogInitialFocus) -> Self {
        self.initial_focus = initial_focus;
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
}

/// Builds a Radix-style alert-dialog modal barrier (non-dismissable by outside press).
pub fn alert_dialog_modal_barrier<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    children: Vec<AnyElement>,
) -> AnyElement {
    dialog_prim::modal_barrier(cx, open, false, children)
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
