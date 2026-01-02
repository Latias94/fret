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
