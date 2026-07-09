use fret_core::{AppWindowId, PanelKey, WindowAnchor};
use fret_runtime::{PlatformCapabilities, UiHost};

use crate::dock::DockManager;

use super::query::panel_location_in_window;
use super::{
    DockSurface, DockSurfaceChange, DockSurfaceDriver, DockSurfaceViewportCloseOutcome,
    DockSurfaceViewportError, DockSurfaceViewportOpenOutcome, DockSurfaceViewportOpenStatus,
    DockSurfaceViewportPlatformReadiness, DockSurfaceViewportReadiness,
    DockSurfaceViewportReadinessStatus, DockSurfaceViewportUnsupportedReason,
};

#[derive(Debug, Clone, Copy)]
pub struct DockSurfaceViewportSession {
    pub(super) surface: DockSurface,
}

impl DockSurfaceViewportSession {
    pub fn check_open_readiness<H: UiHost>(
        &self,
        app: &H,
        panel: &PanelKey,
    ) -> DockSurfaceViewportReadiness {
        let source_window = self
            .surface
            .panel_location(app, panel)
            .map(|location| location.window)
            .unwrap_or(self.surface.main_window);
        self.check_open_readiness_from_window(app, source_window, panel)
    }

    pub fn readiness<H: UiHost>(&self, app: &H, panel: &PanelKey) -> DockSurfaceViewportReadiness {
        self.check_open_readiness(app, panel)
    }

    pub fn check_open_readiness_from_window<H: UiHost>(
        &self,
        app: &H,
        source_window: AppWindowId,
        panel: &PanelKey,
    ) -> DockSurfaceViewportReadiness {
        let (platform, unsupported_reasons) =
            viewport_platform_readiness(app.global::<PlatformCapabilities>());

        let status = match app.global::<DockManager>() {
            None => DockSurfaceViewportReadinessStatus::DockManagerUnavailable,
            Some(dock)
                if panel_location_in_window(&dock.workspace.graph, source_window, panel)
                    .is_none() =>
            {
                DockSurfaceViewportReadinessStatus::PanelNotOpen
            }
            Some(_) if unsupported_reasons.is_empty() => {
                DockSurfaceViewportReadinessStatus::Openable
            }
            Some(_) => DockSurfaceViewportReadinessStatus::InWindowFallback,
        };

        DockSurfaceViewportReadiness {
            panel: panel.clone(),
            source_window,
            status,
            platform,
            unsupported_reasons,
        }
    }

    pub fn readiness_from_window<H: UiHost>(
        &self,
        app: &H,
        source_window: AppWindowId,
        panel: &PanelKey,
    ) -> DockSurfaceViewportReadiness {
        self.check_open_readiness_from_window(app, source_window, panel)
    }

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
        let driver = DockSurfaceDriver::new(self.surface);
        let command_cursor = driver.runtime_command_cursor(app);
        let requested =
            driver.request_float_panel_to_new_window(app, source_window, panel.clone(), anchor);
        if !requested {
            return Err(DockSurfaceViewportError::OpenFailed {
                source_window,
                panel: panel.clone(),
            });
        }

        let window_requests = if supported {
            driver.flush_runtime_commands_since_to_effects(app, command_cursor)
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
        self.before_close_window_into(app, window, self.surface.main_window)
    }

    pub fn before_close_window_into<H: UiHost>(
        &self,
        app: &mut H,
        window: AppWindowId,
        target_window: AppWindowId,
    ) -> Result<DockSurfaceViewportCloseOutcome, DockSurfaceViewportError> {
        if app.global::<DockManager>().is_none() {
            return Err(DockSurfaceViewportError::DockManagerUnavailable);
        }
        let before_panels = app
            .global::<DockManager>()
            .map(|dock| dock.workspace.graph.collect_panels_in_window(window))
            .unwrap_or_default();
        self.surface
            .host_lifecycle()
            .before_close_window_into(app, window, target_window);
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

fn viewport_platform_readiness(
    caps: Option<&PlatformCapabilities>,
) -> (
    DockSurfaceViewportPlatformReadiness,
    Vec<DockSurfaceViewportUnsupportedReason>,
) {
    let Some(caps) = caps else {
        return (
            DockSurfaceViewportPlatformReadiness {
                platform_capabilities_available: false,
                multi_window: false,
                window_tear_off: false,
                window_hover_detection: fret_runtime::WindowHoverDetectionQuality::None,
            },
            vec![DockSurfaceViewportUnsupportedReason::PlatformCapabilitiesUnavailable],
        );
    };

    let platform = DockSurfaceViewportPlatformReadiness {
        platform_capabilities_available: true,
        multi_window: caps.ui.multi_window,
        window_tear_off: caps.ui.window_tear_off,
        window_hover_detection: caps.ui.window_hover_detection,
    };
    let mut reasons = Vec::new();
    if !platform.multi_window {
        reasons.push(DockSurfaceViewportUnsupportedReason::MultiWindowDisabled);
    }
    if !platform.window_tear_off {
        reasons.push(DockSurfaceViewportUnsupportedReason::WindowTearOffDisabled);
    }
    if platform.window_hover_detection == fret_runtime::WindowHoverDetectionQuality::None {
        reasons.push(DockSurfaceViewportUnsupportedReason::WindowHoverDetectionUnavailable);
    }

    (platform, reasons)
}
