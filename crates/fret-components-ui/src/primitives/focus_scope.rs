//! FocusScope (Radix-aligned outcomes).
//!
//! In Radix, FocusScope composes focus trapping/looping and (optionally) auto-focus / restore.
//! In Fret, the runtime provides the focus traversal mechanism, and this primitive provides a
//! stable, Radix-named entry point for component-layer policy.

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

pub use fret_ui::element::FocusScopeProps;

/// Convenience helper for building a trapped focus scope (Tab/Shift+Tab loops within the subtree).
#[track_caller]
pub fn focus_trap<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    cx.focus_scope(
        FocusScopeProps {
            trap_focus: true,
            ..Default::default()
        },
        f,
    )
}

/// Like `focus_trap`, but also exposes the scope element ID.
#[track_caller]
pub fn focus_trap_with_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>, fret_ui::elements::GlobalElementId) -> Vec<AnyElement>,
) -> AnyElement {
    cx.focus_scope_with_id(
        FocusScopeProps {
            trap_focus: true,
            ..Default::default()
        },
        f,
    )
}
