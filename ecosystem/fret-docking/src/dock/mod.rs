use fret_core::dock::DropZone;
use fret_core::geometry::{Px, Rect, Size};
use fret_core::{
    AppWindowId, Color, DockNodeId, PanelKey, RenderTargetId, Scene, ViewportFit, ViewportMapping,
};

mod consts;
mod declarative;
mod diagnostics;
mod drop_resolve;
mod hit_test;
mod host_frame;
mod layout;
mod paint;
mod prelude_core;
mod prelude_runtime;
mod services;
mod split_geometry;
mod tab_bar_drop_target;
mod tab_bar_geometry;
mod tab_bar_kernel;
mod tab_overflow;
mod types;
mod viewport;

mod manager;

pub use declarative::DockPanelElementRegistry;
pub(crate) use declarative::clear_declarative_dock_interactions_for_window;
#[cfg(feature = "imui")]
pub use declarative::imui_dock_space_element;
pub(crate) use declarative::{
    DockPanelElementRegistryService, DockSpaceElementOptions, dock_space_element_from_registry,
};
#[cfg(test)]
pub(crate) use declarative::{
    declarative_dock_interaction_exists_for_window, seed_declarative_dock_interaction_for_window,
};
#[cfg(test)]
pub(crate) use declarative::{dock_panel_element, dock_space_element};
pub use manager::{
    ActivatePanelOptions, DockManager, DockPanelCatalog, DockPanelCatalogError, DockWorkspace,
};
#[cfg(test)]
pub(crate) use services::DockPanelContentService;
pub(crate) use services::{DockViewportOverlayHooksService, DockingPolicyService};
pub(crate) use types::{DockPanelDragPayload, DockTabsDragPayload};

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

    /// Whether tear-off (new OS window) is allowed once docking is already in a multi-window
    /// session.
    ///
    /// By default, docking avoids creating additional OS windows after the first tear-off. This
    /// keeps scripted playback deterministic and prevents "chains" of empty one-panel windows.
    ///
    /// Editors/apps may opt into chained tear-offs by overriding this hook (and/or enabling it
    /// per dock space via `DockSpaceElementOptions::allow_multi_window_tear_off`).
    fn allow_multi_window_tear_off(
        &self,
        _source_window: AppWindowId,
        _panel: &PanelKey,
        _info: Option<&DockPanel>,
    ) -> bool {
        false
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
    /// Provides the full viewport layout bundle recorded by docking.
    #[allow(clippy::too_many_arguments)]
    fn paint_with_layout(
        &self,
        theme: fret_ui::ThemeSnapshot,
        window: fret_core::AppWindowId,
        panel: &PanelKey,
        viewport: ViewportPanel,
        layout: DockViewportLayout,
        scene: &mut Scene,
    );
}

#[cfg(test)]
mod tests;
