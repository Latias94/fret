use fret_core::{AppWindowId, DockLayout, DockPanelLocation, DockPanelLocationKind, PanelKey};
use fret_runtime::WindowHoverDetectionQuality;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSurfaceChange {
    Changed,
    Unchanged,
}

impl DockSurfaceChange {
    pub fn changed(self) -> bool {
        matches!(self, Self::Changed)
    }
}

impl From<bool> for DockSurfaceChange {
    fn from(changed: bool) -> Self {
        if changed {
            Self::Changed
        } else {
            Self::Unchanged
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSurfacePanelPlacement {
    Docked,
    Floating,
}

impl From<DockPanelLocationKind> for DockSurfacePanelPlacement {
    fn from(placement: DockPanelLocationKind) -> Self {
        match placement {
            DockPanelLocationKind::Docked => Self::Docked,
            DockPanelLocationKind::Floating => Self::Floating,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockSurfacePanelLocation {
    pub window: AppWindowId,
    pub placement: DockSurfacePanelPlacement,
    pub tab_index: usize,
    pub tab_count: usize,
    pub active: bool,
}

impl From<DockPanelLocation> for DockSurfacePanelLocation {
    fn from(location: DockPanelLocation) -> Self {
        Self {
            window: location.window,
            placement: location.placement.into(),
            tab_index: location.tab_index,
            tab_count: location.tab_count,
            active: location.active,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockSurfacePanelSnapshot {
    pub key: PanelKey,
    pub title: String,
    pub descriptor_only: bool,
    pub location: Option<DockSurfacePanelLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockSurfacePanelOutcome {
    pub panel: PanelKey,
    pub change: DockSurfaceChange,
    pub location: Option<DockSurfacePanelLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockSurfacePanelError {
    DockManagerUnavailable,
    DuplicatePanelKey { panel: PanelKey },
    PanelNotRegistered { panel: PanelKey },
    PanelNotOpen { panel: PanelKey },
}

#[derive(Debug, Clone)]
pub struct DockSurfaceSnapshot {
    pub layout: DockLayout,
    pub panels: Vec<DockSurfacePanelSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSurfaceViewportOpenStatus {
    WindowCreateQueued,
    InWindowFallback,
    AlreadyPending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSurfaceViewportReadinessStatus {
    Openable,
    InWindowFallback,
    PanelNotOpen,
    DockManagerUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockSurfaceViewportUnsupportedReason {
    PlatformCapabilitiesUnavailable,
    MultiWindowDisabled,
    WindowTearOffDisabled,
    WindowHoverDetectionUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockSurfaceViewportPlatformReadiness {
    pub platform_capabilities_available: bool,
    pub multi_window: bool,
    pub window_tear_off: bool,
    pub window_hover_detection: WindowHoverDetectionQuality,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockSurfaceViewportReadiness {
    pub panel: PanelKey,
    pub source_window: AppWindowId,
    pub status: DockSurfaceViewportReadinessStatus,
    pub platform: DockSurfaceViewportPlatformReadiness,
    pub unsupported_reasons: Vec<DockSurfaceViewportUnsupportedReason>,
}

impl DockSurfaceViewportReadiness {
    pub fn can_open_platform_viewport(&self) -> bool {
        matches!(self.status, DockSurfaceViewportReadinessStatus::Openable)
    }

    pub fn will_use_in_window_fallback(&self) -> bool {
        matches!(
            self.status,
            DockSurfaceViewportReadinessStatus::InWindowFallback
        )
    }

    pub fn has_unsupported_platform_reason(
        &self,
        reason: DockSurfaceViewportUnsupportedReason,
    ) -> bool {
        self.unsupported_reasons.contains(&reason)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DockSurfaceViewportOpenOutcome {
    pub panel: PanelKey,
    pub source_window: AppWindowId,
    pub status: DockSurfaceViewportOpenStatus,
    pub change: DockSurfaceChange,
    pub window_requests: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockSurfaceViewportCloseOutcome {
    pub window: AppWindowId,
    pub change: DockSurfaceChange,
    pub window_requests: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockSurfaceViewportError {
    DockManagerUnavailable,
    PanelNotOpen {
        source_window: AppWindowId,
        panel: PanelKey,
    },
    OpenFailed {
        source_window: AppWindowId,
        panel: PanelKey,
    },
}
