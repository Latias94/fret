use fret_core::{AppWindowId, DockLayout, DockPanelLocation, DockPanelPlacement, PanelKey};

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

impl From<DockPanelPlacement> for DockSurfacePanelPlacement {
    fn from(placement: DockPanelPlacement) -> Self {
        match placement {
            DockPanelPlacement::Docked => Self::Docked,
            DockPanelPlacement::Floating => Self::Floating,
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
