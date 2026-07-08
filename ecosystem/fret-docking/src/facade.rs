use std::sync::Arc;

use fret_core::{
    AppWindowId, DockLayout, DockLayoutValidationError, DockOp, DockWindowPlacement, PanelKey,
};
use fret_runtime::UiHost;
use fret_ui::ElementContext;
use fret_ui::element::AnyElement;

use crate::dock::{
    DockManager, DockPanel, DockPanelElementRegistry, DockPanelElementRegistryService,
    DockSpaceElementOptions, DockViewportOverlayHooks, DockViewportOverlayHooksService,
    DockingPolicy, DockingPolicyService, dock_space_element_from_registry,
};
pub type DockHostOptions = DockSpaceElementOptions;

mod driver;
mod query;
#[cfg(test)]
mod tests;
mod types;
mod viewport;

pub use driver::DockSurfaceDriver;
pub use types::{
    DockSurfaceChange, DockSurfacePanelError, DockSurfacePanelLocation, DockSurfacePanelOutcome,
    DockSurfacePanelPlacement, DockSurfacePanelSnapshot, DockSurfaceSnapshot,
    DockSurfaceViewportCloseOutcome, DockSurfaceViewportError, DockSurfaceViewportOpenOutcome,
    DockSurfaceViewportOpenStatus,
};
pub use viewport::DockSurfaceViewportSession;

use query::{panel_location, panel_snapshot, registered_panel_snapshots, selected_panel_in_window};

/// App-facing docking surface.
///
/// `DockSurface` is the preferred ordinary entry point for applications. It keeps common app code
/// on facade operations while lower-level manager access stays behind explicit advanced modules.
#[derive(Debug, Clone, Copy)]
pub struct DockSurface {
    pub(super) main_window: AppWindowId,
}

impl DockSurface {
    pub fn new(main_window: AppWindowId) -> Self {
        Self { main_window }
    }

    pub fn main_window(&self) -> AppWindowId {
        self.main_window
    }

    pub fn register_panel<H: UiHost>(&self, app: &mut H, key: PanelKey, panel: DockPanel) {
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(key, panel);
        });
    }

    pub fn ensure_panel<H: UiHost>(
        &self,
        app: &mut H,
        key: &PanelKey,
        make: impl FnOnce() -> DockPanel,
    ) {
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.ensure_panel(key, make);
        });
    }

    pub fn has_window_root<H: UiHost>(&self, app: &H, window: AppWindowId) -> bool {
        app.global::<DockManager>()
            .is_some_and(|dock| dock.workspace.graph.window_root(window).is_some())
    }

    pub fn import_layout_for_windows<H: UiHost>(
        &self,
        app: &mut H,
        layout: &DockLayout,
        windows: &[(AppWindowId, String)],
    ) -> bool {
        self.try_import_layout_for_windows(app, layout, windows)
            .unwrap_or(false)
    }

    pub fn try_import_layout_for_windows<H: UiHost>(
        &self,
        app: &mut H,
        layout: &DockLayout,
        windows: &[(AppWindowId, String)],
    ) -> Result<bool, DockLayoutValidationError> {
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.workspace
                .import_layout_for_windows_checked(layout, windows)
        })
    }

    pub fn import_layout_for_windows_with_fallback_floatings<H: UiHost>(
        &self,
        app: &mut H,
        layout: &DockLayout,
        windows: &[(AppWindowId, String)],
        fallback_window: AppWindowId,
    ) -> bool {
        self.try_import_layout_for_windows_with_fallback_floatings(
            app,
            layout,
            windows,
            fallback_window,
        )
        .unwrap_or(false)
    }

    pub fn try_import_layout_for_windows_with_fallback_floatings<H: UiHost>(
        &self,
        app: &mut H,
        layout: &DockLayout,
        windows: &[(AppWindowId, String)],
        fallback_window: AppWindowId,
    ) -> Result<bool, DockLayoutValidationError> {
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.workspace
                .import_layout_for_windows_with_fallback_floatings_checked(
                    layout,
                    windows,
                    fallback_window,
                )
        })
    }

    pub fn export_layout<H: UiHost>(
        &self,
        app: &H,
        windows: &[(AppWindowId, String)],
    ) -> Option<DockLayout> {
        app.global::<DockManager>()
            .map(|dock| dock.workspace.graph.export_layout(windows))
    }

    pub fn export_layout_with_placement<H: UiHost>(
        &self,
        app: &H,
        windows: &[(AppWindowId, String)],
        placement: impl Fn(AppWindowId) -> Option<DockWindowPlacement>,
    ) -> Option<DockLayout> {
        app.global::<DockManager>().map(|dock| {
            dock.workspace
                .graph
                .export_layout_with_placement(windows, placement)
        })
    }

    pub fn snapshot<H: UiHost>(
        &self,
        app: &H,
        windows: &[(AppWindowId, String)],
    ) -> Option<DockSurfaceSnapshot> {
        self.snapshot_with_placement(app, windows, |_| None)
    }

    pub fn snapshot_with_placement<H: UiHost>(
        &self,
        app: &H,
        windows: &[(AppWindowId, String)],
        placement: impl FnMut(AppWindowId) -> Option<DockWindowPlacement>,
    ) -> Option<DockSurfaceSnapshot> {
        app.global::<DockManager>().map(|dock| DockSurfaceSnapshot {
            layout: dock
                .workspace
                .graph
                .export_layout_with_placement(windows, placement),
            panels: registered_panel_snapshots(dock),
        })
    }

    pub fn registered_panels<H: UiHost>(&self, app: &H) -> Vec<DockSurfacePanelSnapshot> {
        self.try_registered_panels(app).unwrap_or_default()
    }

    pub fn try_registered_panels<H: UiHost>(
        &self,
        app: &H,
    ) -> Result<Vec<DockSurfacePanelSnapshot>, DockSurfacePanelError> {
        app.global::<DockManager>()
            .map(registered_panel_snapshots)
            .ok_or(DockSurfacePanelError::DockManagerUnavailable)
    }

    pub fn panels_in_window<H: UiHost>(
        &self,
        app: &H,
        window: AppWindowId,
    ) -> Vec<DockSurfacePanelSnapshot> {
        self.try_panels_in_window(app, window).unwrap_or_default()
    }

    pub fn try_panels_in_window<H: UiHost>(
        &self,
        app: &H,
        window: AppWindowId,
    ) -> Result<Vec<DockSurfacePanelSnapshot>, DockSurfacePanelError> {
        let Some(dock) = app.global::<DockManager>() else {
            return Err(DockSurfacePanelError::DockManagerUnavailable);
        };
        Ok(dock
            .workspace
            .graph
            .collect_panels_in_window(window)
            .into_iter()
            .map(|panel| panel_snapshot(dock, panel))
            .collect())
    }

    pub fn selected_panel_in_window<H: UiHost>(
        &self,
        app: &H,
        window: AppWindowId,
    ) -> Option<PanelKey> {
        self.try_selected_panel_in_window(app, window)
            .ok()
            .flatten()
    }

    pub fn try_selected_panel_in_window<H: UiHost>(
        &self,
        app: &H,
        window: AppWindowId,
    ) -> Result<Option<PanelKey>, DockSurfacePanelError> {
        let Some(dock) = app.global::<DockManager>() else {
            return Err(DockSurfacePanelError::DockManagerUnavailable);
        };
        Ok(selected_panel_in_window(&dock.workspace.graph, window))
    }

    pub fn panel_location<H: UiHost>(
        &self,
        app: &H,
        panel: &PanelKey,
    ) -> Option<DockSurfacePanelLocation> {
        self.try_panel_location(app, panel).ok().flatten()
    }

    pub fn try_panel_location<H: UiHost>(
        &self,
        app: &H,
        panel: &PanelKey,
    ) -> Result<Option<DockSurfacePanelLocation>, DockSurfacePanelError> {
        let Some(dock) = app.global::<DockManager>() else {
            return Err(DockSurfacePanelError::DockManagerUnavailable);
        };
        Ok(panel_location(dock, panel))
    }

    pub fn open_panel<H: UiHost>(
        &self,
        app: &mut H,
        panel: &PanelKey,
    ) -> Result<DockSurfacePanelOutcome, DockSurfacePanelError> {
        self.open_panel_in_window(app, self.main_window, panel)
    }

    pub fn open_panel_in_window<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        panel: &PanelKey,
    ) -> Result<DockSurfacePanelOutcome, DockSurfacePanelError> {
        let Some(dock) = app.global::<DockManager>() else {
            return Err(DockSurfacePanelError::DockManagerUnavailable);
        };
        if dock.workspace.panel(panel).is_none() {
            return Err(DockSurfacePanelError::PanelNotRegistered {
                panel: panel.clone(),
            });
        }
        let before = panel_location(dock, panel);

        self.driver().on_dock_op(
            app,
            DockOp::OpenPanel {
                window,
                panel: panel.clone(),
            },
        );
        self.driver().flush_runtime_commands_to_effects(app);
        Ok(self.panel_outcome_from_before(app, panel.clone(), before))
    }

    pub fn select_panel<H: UiHost>(
        &self,
        app: &mut H,
        panel: &PanelKey,
    ) -> Result<DockSurfacePanelOutcome, DockSurfacePanelError> {
        let Some((_, op)) = app
            .global::<DockManager>()
            .and_then(|dock| dock.activate_panel_tab_best_effort([self.main_window], panel))
        else {
            if app.global::<DockManager>().is_none() {
                return Err(DockSurfacePanelError::DockManagerUnavailable);
            }
            return Err(DockSurfacePanelError::PanelNotOpen {
                panel: panel.clone(),
            });
        };

        let before = self.panel_location(app, panel);
        self.driver().on_dock_op(app, op);
        self.driver().flush_runtime_commands_to_effects(app);
        Ok(self.panel_outcome_from_before(app, panel.clone(), before))
    }

    pub fn close_panel<H: UiHost>(
        &self,
        app: &mut H,
        panel: &PanelKey,
    ) -> Result<DockSurfacePanelOutcome, DockSurfacePanelError> {
        let Some(location) = self.panel_location(app, panel) else {
            if app.global::<DockManager>().is_none() {
                return Err(DockSurfacePanelError::DockManagerUnavailable);
            }
            return Err(DockSurfacePanelError::PanelNotOpen {
                panel: panel.clone(),
            });
        };

        let close_window = location.window;
        let before = Some(location);
        self.driver().on_dock_op(
            app,
            DockOp::ClosePanel {
                window: close_window,
                panel: panel.clone(),
            },
        );
        self.driver().flush_runtime_commands_to_effects(app);
        Ok(self.panel_outcome_from_before(app, panel.clone(), before))
    }

    pub fn install_panel_registry<H: UiHost + 'static>(
        &self,
        app: &mut H,
        registry: Arc<dyn DockPanelElementRegistry<H>>,
    ) {
        app.with_global_mut(
            DockPanelElementRegistryService::<H>::default,
            |service, _app| {
                service.set(registry);
            },
        );
    }

    pub fn install_policy<H: UiHost>(&self, app: &mut H, policy: Arc<dyn DockingPolicy>) {
        app.with_global_mut(DockingPolicyService::default, |service, _app| {
            service.set(policy);
        });
    }

    pub fn install_viewport_overlay_hooks<H: UiHost>(
        &self,
        app: &mut H,
        hooks: Arc<dyn DockViewportOverlayHooks>,
    ) {
        app.with_global_mut(DockViewportOverlayHooksService::default, |service, _app| {
            service.set(hooks);
        });
    }

    pub fn host<H>(
        &self,
        cx: &mut ElementContext<'_, H>,
        window: AppWindowId,
        options: DockHostOptions,
    ) -> AnyElement
    where
        H: UiHost + 'static,
    {
        dock_space_element_from_registry(cx, window, options)
    }

    /// Returns the explicit driver-tier API for runtime and host integration callbacks.
    pub fn driver(&self) -> DockSurfaceDriver {
        DockSurfaceDriver { surface: *self }
    }

    pub fn viewports(&self) -> DockSurfaceViewportSession {
        DockSurfaceViewportSession { surface: *self }
    }

    fn panel_outcome_from_before<H: UiHost>(
        &self,
        app: &H,
        panel: PanelKey,
        before: Option<DockSurfacePanelLocation>,
    ) -> DockSurfacePanelOutcome {
        let location = self.panel_location(app, &panel);
        DockSurfacePanelOutcome {
            panel,
            change: DockSurfaceChange::from(before != location),
            location,
        }
    }
}
