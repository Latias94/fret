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

use fret_runtime::ModelId;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

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
}
