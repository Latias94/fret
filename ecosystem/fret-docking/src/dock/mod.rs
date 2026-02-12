use fret_core::dock::DropZone;
use fret_core::geometry::{Px, Rect, Size};
use fret_core::{
    AppWindowId, Color, DockNodeId, NodeId, PanelKey, RenderTargetId, Scene, ViewportFit,
    ViewportMapping,
};
use fret_ui::UiHost;

mod consts;
mod hit_test;
mod layout;
mod paint;
mod panel_registry;
mod prelude_core;
mod prelude_runtime;
mod prelude_ui;
mod services;
mod tab_bar_geometry;
mod tab_overflow;
mod types;
mod viewport;

mod manager;
mod space;

pub use manager::{ActivatePanelOptions, DockManager};
pub use panel_registry::{
    DockPanelRegistry, DockPanelRegistryService, render_and_bind_dock_panels,
    render_cached_panel_root,
};
pub use services::{
    DockPanelContentService, DockViewportOverlayHooksService, DockingPolicyService,
};
pub use space::DockSpace;

pub struct DockPanel {
    pub title: String,
    pub color: Color,
    pub viewport: Option<ViewportPanel>,
}

/// Docking policy hooks for editor-grade constraints.
///
/// This trait is intentionally owned by the docking layer (not `fret-ui`). It is the primary
/// extension point for "hard to change" editor constraints such as minimum sizes, no-drop zones,
/// group locking, and "no tear-off" policies.
pub trait DockingPolicy: Send + Sync + 'static {
    /// Minimum content size for a panel when hosted inside docking tabs.
    ///
    /// This is used to clamp splitter drags so panels (especially viewports) do not collapse into
    /// unusable sizes.
    ///
    /// The size is expressed in **logical px**.
    fn panel_min_content_size(&self, _panel: &PanelKey, _info: Option<&DockPanel>) -> Option<Size> {
        None
    }

    /// Whether a dock drop target is allowed.
    ///
    /// Docking UI resolves an explicit drop target (tab bar center insert, inner hint-pad, or
    /// outer hint-pad) before committing a `DockOp`. This hook allows editors/apps to mask out
    /// specific drop zones (e.g. disallow edge docking on certain groups, disable top/bottom
    /// docking in a single-pane mode, etc.).
    fn allow_dock_drop_target(
        &self,
        _window: AppWindowId,
        _layout_root: DockNodeId,
        _tabs: DockNodeId,
        _zone: DropZone,
        _outer: bool,
    ) -> bool {
        true
    }

    /// Whether a dock drag is allowed to request an OS-window tear-off when leaving window bounds.
    ///
    /// Docking has two tear-off paths:
    /// - Debounced "stable OOB" request while hovering outside the window.
    /// - Drop-time resolution when the pointer is outside window bounds.
    ///
    /// This hook gates both.
    fn allow_tear_off(
        &self,
        _source_window: AppWindowId,
        _panel: &PanelKey,
        _info: Option<&DockPanel>,
    ) -> bool {
        true
    }

    /// Whether a panel tab is allowed to start a dock drag (tear-off / docking previews).
    ///
    /// This is the "group locking" escape hatch: editors can prevent dragging certain panels
    /// (or entire groups) without changing core docking ops.
    fn allow_panel_drag(
        &self,
        _window: AppWindowId,
        _panel: &PanelKey,
        _info: Option<&DockPanel>,
    ) -> bool {
        true
    }

    /// Whether a tab-stack (tabs node) is allowed to start a "tabs group" drag.
    fn allow_tabs_group_drag(&self, _window: AppWindowId, _tabs: DockNodeId) -> bool {
        true
    }
}

pub(crate) fn default_viewport_min_content_size() -> Size {
    // Default editor feel: viewports should not collapse to tiny sizes during splitter drags.
    //
    // Apps can override this via `DockingPolicyService`.
    Size::new(Px(240.0), Px(160.0))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockViewportLayout {
    pub content_rect: Rect,
    pub mapping: ViewportMapping,
    pub draw_rect: Rect,
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
    /// Preferred overlay hook: provides the full viewport layout bundle recorded by docking.
    ///
    /// This method is non-breaking: the default implementation forwards to `paint(...)`.
    #[allow(clippy::too_many_arguments)]
    fn paint_with_layout(
        &self,
        theme: fret_ui::ThemeSnapshot,
        window: fret_core::AppWindowId,
        panel: &PanelKey,
        viewport: ViewportPanel,
        layout: DockViewportLayout,
        scene: &mut Scene,
    ) {
        self.paint(
            theme,
            window,
            panel,
            viewport,
            layout.mapping,
            layout.draw_rect,
            scene,
        );
    }

    /// Legacy overlay hook: prefer `paint_with_layout(...)` for new code.
    ///
    /// Default implementation is a no-op to keep implementations minimal when the overlay hook is
    /// unused.
    #[allow(clippy::too_many_arguments)]
    fn paint(
        &self,
        _theme: fret_ui::ThemeSnapshot,
        _window: fret_core::AppWindowId,
        _panel: &PanelKey,
        _viewport: ViewportPanel,
        _mapping: ViewportMapping,
        _draw_rect: Rect,
        _scene: &mut Scene,
    ) {
    }
}

pub fn create_dock_space_node<H: UiHost>(
    ui: &mut fret_ui::UiTree<H>,
    window: fret_core::AppWindowId,
) -> NodeId {
    // Integration contract:
    // - Create one DockSpace node per window and keep it alive (do not conditionally omit it).
    // - Call `render_and_bind_dock_panels(...)` every frame before `layout_all`/`paint_all`.
    // This matches ADR 0013's "dock host keep-alive / early submission" guidance.
    use fret_ui::retained_bridge::UiTreeRetainedExt as _;
    ui.create_node_retained(DockSpace::new(window))
}

pub fn create_dock_space_node_with_test_id<H: UiHost>(
    ui: &mut fret_ui::UiTree<H>,
    window: fret_core::AppWindowId,
    test_id: &'static str,
) -> NodeId {
    use fret_ui::retained_bridge::UiTreeRetainedExt as _;
    ui.create_node_retained(DockSpace::new(window).with_semantics_test_id(test_id))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockSpaceMount {
    pub root: NodeId,
    pub dock_space: NodeId,
}

/// Create a dock space node and mount it as the UI root.
///
/// This helper exists to prevent integration bugs where a dock space node is created but never
/// mounted into the tree (which would break hit testing and internal drag routing).
pub fn mount_dock_space<H: UiHost>(
    ui: &mut fret_ui::UiTree<H>,
    window: fret_core::AppWindowId,
) -> DockSpaceMount {
    let dock_space = create_dock_space_node(ui, window);
    ui.set_root(dock_space);
    DockSpaceMount {
        root: dock_space,
        dock_space,
    }
}

/// `mount_dock_space(...)` variant that also sets a semantics test id.
pub fn mount_dock_space_with_test_id<H: UiHost>(
    ui: &mut fret_ui::UiTree<H>,
    window: fret_core::AppWindowId,
    test_id: &'static str,
) -> DockSpaceMount {
    let dock_space = create_dock_space_node_with_test_id(ui, window, test_id);
    ui.set_root(dock_space);
    DockSpaceMount {
        root: dock_space,
        dock_space,
    }
}

#[cfg(test)]
mod tests;
