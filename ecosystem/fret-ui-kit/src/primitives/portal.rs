//! Portal primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/portal/src/portal.tsx`
//!
//! In Radix/DOM, `Portal` renders its subtree into a different DOM container (defaulting to
//! `document.body` when mounted).
//!
//! In Fret, the analogous layering mechanism is the per-window overlay root stack (ADR 0011,
//! ADR 0067). Overlay roots are installed by `OverlayController` and are named via stable root-name
//! helpers. This module is a thin, Radix-named facade over those naming conventions so component
//! authors can depend on `primitives::*` without reaching into the overlay controller directly.

use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::OverlayController;
use crate::primitives::portal_inherited;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalKind {
    Popover,
    Modal,
    Tooltip,
    Hover,
    ToastLayer,
}

pub fn portal_root_name(kind: PortalKind, id: GlobalElementId) -> String {
    match kind {
        PortalKind::Popover => OverlayController::popover_root_name(id),
        PortalKind::Modal => OverlayController::modal_root_name(id),
        PortalKind::Tooltip => OverlayController::tooltip_root_name(id),
        PortalKind::Hover => OverlayController::hover_overlay_root_name(id),
        PortalKind::ToastLayer => OverlayController::toast_layer_root_name(id),
    }
}

/// Run `f` inside a stable portal root name scope.
///
/// This is a convenience wrapper over `ElementContext::with_root_name(...)`.
#[track_caller]
pub fn with_portal_root_name<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    kind: PortalKind,
    id: GlobalElementId,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let inherited = portal_inherited::PortalInherited::capture(cx);
    let root_name = portal_root_name(kind, id);
    portal_inherited::with_root_name_inheriting(cx, &root_name, inherited, f)
}
