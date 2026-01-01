use fret_core::{AppWindowId, NodeId, Rect, UiServices};
use fret_ui::element::AnyElement;
use fret_ui::{ElementCx, UiHost, UiTree};

pub use fret_ui::action::DismissReason;

/// Render a full-window dismissible root that provides Escape + outside-press dismissal hooks.
///
/// This is a small wrapper over `fret-ui`'s `render_dismissible_root_with_hooks(...)` so component
/// crates can depend on `fret-components-ui` as the stable policy surface (ADR 0067).
pub fn render_dismissible_root_with_hooks<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn UiServices,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    render: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> NodeId {
    fret_ui::declarative::render_dismissible_root_with_hooks(
        ui, app, services, window, bounds, root_name, render,
    )
}
