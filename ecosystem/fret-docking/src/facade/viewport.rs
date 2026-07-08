use fret_core::{AppWindowId, PanelKey, WindowAnchor};
use fret_runtime::{PlatformCapabilities, UiHost};

use crate::dock::DockManager;

use super::query::panel_location_in_window;
use super::{
    DockSurface, DockSurfaceChange, DockSurfaceViewportCloseOutcome, DockSurfaceViewportError,
    DockSurfaceViewportOpenOutcome, DockSurfaceViewportOpenStatus,
};

#[derive(Debug, Clone, Copy)]
pub struct DockSurfaceViewportSession {
    pub(super) surface: DockSurface,
}

impl DockSurfaceViewportSession {
    pub fn open_panel<H: UiHost>(
        &self,
        app: &mut H,
        panel: &PanelKey,
        anchor: Option<WindowAnchor>,
    ) -> Result<DockSurfaceViewportOpenOutcome, DockSurfaceViewportError> {
        let source_window = self
            .surface
            .panel_location(app, panel)
            .map(|location| location.window)
            .unwrap_or(self.surface.main_window);
        self.open_panel_from_window(app, source_window, panel, anchor)
    }

    pub fn open_panel_from_window<H: UiHost>(
        &self,
        app: &mut H,
        source_window: AppWindowId,
        panel: &PanelKey,
        anchor: Option<WindowAnchor>,
    ) -> Result<DockSurfaceViewportOpenOutcome, DockSurfaceViewportError> {
        let Some(dock) = app.global::<DockManager>() else {
            return Err(DockSurfaceViewportError::DockManagerUnavailable);
        };
        if panel_location_in_window(&dock.workspace.graph, source_window, panel).is_none() {
            return Err(DockSurfaceViewportError::PanelNotOpen {
                source_window,
                panel: panel.clone(),
            });
        }

        let supported =
            crate::runtime::dock_tear_off_supported(app.global::<PlatformCapabilities>());
        let driver = self.surface.driver();
        let command_baseline = driver.runtime_command_count(app);
        let requested =
            driver.request_float_panel_to_new_window(app, source_window, panel.clone(), anchor);
        if !requested {
            return Err(DockSurfaceViewportError::OpenFailed {
                source_window,
                panel: panel.clone(),
            });
        }

        let window_requests = if supported {
            driver.flush_runtime_commands_since_to_effects(app, command_baseline)
        } else {
            0
        };
        let status = if supported {
            if window_requests > 0 {
                DockSurfaceViewportOpenStatus::WindowCreateQueued
            } else {
                DockSurfaceViewportOpenStatus::AlreadyPending
            }
        } else {
            DockSurfaceViewportOpenStatus::InWindowFallback
        };
        Ok(DockSurfaceViewportOpenOutcome {
            panel: panel.clone(),
            source_window,
            status,
            change: DockSurfaceChange::from(!supported),
            window_requests,
        })
    }

    pub fn before_close_window<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
    ) -> Result<DockSurfaceViewportCloseOutcome, DockSurfaceViewportError> {
        if app.global::<DockManager>().is_none() {
            return Err(DockSurfaceViewportError::DockManagerUnavailable);
        }
        let before_panels = app
            .global::<DockManager>()
            .map(|dock| dock.workspace.graph.collect_panels_in_window(window))
            .unwrap_or_default();
        self.surface.driver().before_close_window(app, window);
        let after_panels = app
            .global::<DockManager>()
            .map(|dock| dock.workspace.graph.collect_panels_in_window(window))
            .unwrap_or_default();
        Ok(DockSurfaceViewportCloseOutcome {
            window,
            change: DockSurfaceChange::from(before_panels != after_panels),
            window_requests: 0,
        })
    }
}
