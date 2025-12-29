use fret_core::geometry::Rect;
use fret_core::{Color, NodeId, PanelKey, RenderTargetId, Scene, ViewportFit, ViewportMapping};
use fret_ui::UiHost;

mod consts;
mod hit_test;
mod layout;
mod paint;
mod prelude_core;
mod prelude_runtime;
mod prelude_ui;
mod services;
mod split_stabilize;
mod types;
mod viewport;

mod manager;
mod space;

pub use manager::{ActivatePanelOptions, DockManager};
pub use services::{DockPanelContentService, DockViewportOverlayHooksService};
pub use space::DockSpace;

pub struct DockPanel {
    pub title: String,
    pub color: Color,
    pub viewport: Option<ViewportPanel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportPanel {
    pub target: RenderTargetId,
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
    pub context_menu_enabled: bool,
}

/// App/editor-owned viewport overlays (gizmos, marquee, selection, etc.).
///
/// Docking UI is policy-heavy already, but viewport overlay *shapes* are editor/app-specific
/// (ADR 0027 / ADR 0049). This hook keeps docking focused on "viewport embedding" only.
pub trait DockViewportOverlayHooks: Send + Sync + 'static {
    fn paint(
        &self,
        theme: fret_ui::ThemeSnapshot,
        window: fret_core::AppWindowId,
        panel: &PanelKey,
        viewport: ViewportPanel,
        mapping: ViewportMapping,
        draw_rect: Rect,
        scene: &mut Scene,
    );
}

pub fn create_dock_space_node<H: UiHost>(
    ui: &mut fret_ui::UiTree<H>,
    window: fret_core::AppWindowId,
) -> NodeId {
    use fret_ui::retained_bridge::UiTreeRetainedExt as _;
    ui.create_node_retained(DockSpace::new(window))
}

#[cfg(test)]
mod tests;
