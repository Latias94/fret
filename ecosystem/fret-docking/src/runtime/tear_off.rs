use std::collections::{HashMap, HashSet};

use fret_core::{AppWindowId, DockNode, DockNodeId, DockOp, PanelKey, PointerId, WindowAnchor};
use fret_runtime::{
    CreateWindowKind, CreateWindowRequest, Effect, PlatformCapabilities, TickId, UiHost,
    WindowRequest, WindowRole,
};

use crate::DockManager;

use super::commands::{self, DockRuntimeCommand};

#[derive(Debug, Clone, Copy)]
pub(super) enum DockTearOffCompletion {
    Proceed,
    CancelAndCloseWindow,
}

#[derive(Default)]
pub(super) struct DockFloatingOsWindowRegistry {
    windows: HashSet<AppWindowId>,
}

impl DockFloatingOsWindowRegistry {
    pub(super) fn register(&mut self, window: AppWindowId) {
        self.windows.insert(window);
    }

    pub(super) fn remove(&mut self, window: AppWindowId) {
        self.windows.remove(&window);
    }

    fn contains(&self, window: AppWindowId) -> bool {
        self.windows.contains(&window)
    }

    pub(super) fn windows(&self) -> impl Iterator<Item = AppWindowId> + '_ {
        self.windows.iter().copied()
    }
}

pub(crate) fn is_dock_floating_os_window<H: UiHost>(app: &H, window: AppWindowId) -> bool {
    app.global::<DockFloatingOsWindowRegistry>()
        .is_some_and(|reg| reg.contains(window))
}

pub(super) fn dock_tear_off_supported(caps: Option<&PlatformCapabilities>) -> bool {
    caps.is_some_and(|caps| {
        caps.ui.multi_window
            && caps.ui.window_tear_off
            && caps.ui.window_hover_detection != fret_runtime::WindowHoverDetectionQuality::None
    })
}

pub(super) fn push_dock_floating_window_create<H: UiHost>(
    app: &mut H,
    source_window: AppWindowId,
    panel: PanelKey,
    anchor: Option<WindowAnchor>,
) {
    let request = dock_floating_window_create_request(app, source_window, panel, anchor);
    app.push_effect(Effect::Window(WindowRequest::Create(request)));
}

pub(super) fn queue_dock_floating_window_create<H: UiHost>(
    app: &mut H,
    source_window: AppWindowId,
    panel: PanelKey,
    anchor: Option<WindowAnchor>,
) {
    let request = dock_floating_window_create_request(app, source_window, panel, anchor);
    commands::push_runtime_command(app, DockRuntimeCommand::CreateWindow(request));
}

fn dock_floating_window_create_request<H: UiHost>(
    app: &H,
    source_window: AppWindowId,
    panel: PanelKey,
    anchor: Option<WindowAnchor>,
) -> CreateWindowRequest {
    let caps = app
        .global::<PlatformCapabilities>()
        .cloned()
        .unwrap_or_default();
    CreateWindowRequest {
        kind: CreateWindowKind::DockFloating {
            source_window,
            panel,
        },
        anchor,
        role: WindowRole::Auxiliary,
        style: fret_window_style_profiles::tool_window_profile_v1(&caps).style,
    }
}

#[derive(Debug, Clone)]
pub(super) struct DockTearOffPending {
    source_window: AppWindowId,
    pub(super) kind: DockTearOffKind,
    pub(super) pointer_id: Option<PointerId>,
    requested_at: TickId,
    canceled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DockTearOffKind {
    Panel,
    Tabs { source_tabs: DockNodeId },
}

/// Small runtime-layer state machine to keep tear-off window creation idempotent.
///
/// This intentionally lives outside `fret-core` (graph stays pure) and outside the UI widget
/// (covers duplicate ops emitted by runners/drivers or other app code).
#[derive(Default)]
pub(super) struct DockTearOffMachine {
    pending_by_panel: HashMap<PanelKey, DockTearOffPending>,
}

impl DockTearOffMachine {
    // If a create request fails (e.g. backend error), we may never receive `window_created`.
    // Use a TTL so a later tear-off attempt can recover.
    const PENDING_TTL_TICKS: u64 = 600;

    fn prune_expired(&mut self, now: TickId) {
        self.pending_by_panel.retain(|_, pending| {
            let age = now.0.saturating_sub(pending.requested_at.0);
            age <= Self::PENDING_TTL_TICKS
        });
    }

    pub(super) fn register_request(
        &mut self,
        now: TickId,
        source_window: AppWindowId,
        panel: &PanelKey,
        kind: DockTearOffKind,
        pointer_id: Option<PointerId>,
    ) -> bool {
        self.prune_expired(now);
        match self.pending_by_panel.get(panel) {
            Some(_) => false,
            None => {
                self.pending_by_panel.insert(
                    panel.clone(),
                    DockTearOffPending {
                        source_window,
                        kind,
                        pointer_id,
                        requested_at: now,
                        canceled: false,
                    },
                );
                true
            }
        }
    }

    fn cancel_for_panel(&mut self, panel: &PanelKey) {
        if let Some(pending) = self.pending_by_panel.get_mut(panel) {
            pending.canceled = true;
        }
    }

    pub(super) fn prune_and_cancel_for_op(&mut self, now: TickId, dock: &DockManager, op: &DockOp) {
        self.prune_expired(now);
        match op {
            DockOp::ClosePanel { panel, .. }
            | DockOp::MovePanel { panel, .. }
            | DockOp::MovePanelToEmptyDockSpace { panel, .. }
            | DockOp::FloatPanelToWindow { panel, .. }
            | DockOp::FloatPanelInWindow { panel, .. } => {
                self.cancel_for_panel(panel);
            }
            DockOp::MoveTabs { source_tabs, .. }
            | DockOp::MoveTabsToEmptyDockSpace { source_tabs, .. }
            | DockOp::FloatTabsInWindow { source_tabs, .. } => {
                self.cancel_for_tabs_node(dock, *source_tabs);
            }
            DockOp::MoveWindowToEmptyDockSpace { source_window, .. }
            | DockOp::MergeWindowInto { source_window, .. } => {
                self.cancel_for_window(dock, *source_window);
            }
            _ => {}
        }
    }

    fn cancel_for_window(&mut self, dock: &DockManager, source_window: AppWindowId) {
        for panel in dock.graph.collect_panels_in_window(source_window) {
            self.cancel_for_panel(&panel);
        }
    }

    fn cancel_for_tabs_node(&mut self, dock: &DockManager, source_tabs: DockNodeId) {
        if let Some(DockNode::Tabs { tabs, .. }) = dock.graph.node(source_tabs) {
            for panel in tabs {
                self.cancel_for_panel(panel);
            }
        }
    }

    pub(super) fn complete_for_create_request(
        &mut self,
        request: &CreateWindowRequest,
        now: TickId,
    ) -> (DockTearOffCompletion, Option<DockTearOffPending>) {
        self.prune_expired(now);
        let CreateWindowKind::DockFloating {
            source_window,
            panel,
        } = &request.kind
        else {
            return (DockTearOffCompletion::Proceed, None);
        };

        let Some(pending) = self.pending_by_panel.remove(panel) else {
            // If we can't correlate the request, default to proceeding; callers may still apply the
            // graph update if the panel exists.
            return (DockTearOffCompletion::Proceed, None);
        };

        if pending.canceled || pending.source_window != *source_window {
            return (DockTearOffCompletion::CancelAndCloseWindow, Some(pending));
        }

        (DockTearOffCompletion::Proceed, Some(pending))
    }
}
