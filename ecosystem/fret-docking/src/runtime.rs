//! App/runner integration helpers for docking.
//!
//! The docking UI emits high-level `DockOp` transactions via `Effect::Dock(...)` (ADR 0013).
//! These ops must be applied by the app/runner layer:
//! - apply graph mutations (`DockGraph::apply_op`)
//! - translate `RequestFloatPanelToNewWindow` into a `WindowRequest::Create`
//! - complete the float by updating the graph once the OS window exists

use fret_core::{AppWindowId, DockOp};
use fret_runtime::{
    CreateWindowKind, CreateWindowRequest, Effect, PlatformCapabilities, UiHost, WindowRequest,
};

use crate::DockManager;
use crate::invalidation::DockInvalidationService;

#[derive(Debug, Clone, Copy)]
enum DockTearOffCompletion {
    Proceed,
    CancelAndCloseWindow,
}

#[derive(Default)]
struct DockFloatingOsWindowRegistry {
    windows: std::collections::HashSet<AppWindowId>,
}

impl DockFloatingOsWindowRegistry {
    fn register(&mut self, window: AppWindowId) {
        self.windows.insert(window);
    }

    fn remove(&mut self, window: AppWindowId) {
        self.windows.remove(&window);
    }

    fn contains(&self, window: AppWindowId) -> bool {
        self.windows.contains(&window)
    }
}

#[derive(Debug, Clone)]
struct DockTearOffPending {
    source_window: AppWindowId,
    requested_at: fret_runtime::TickId,
    canceled: bool,
}

/// Small runtime-layer state machine to keep tear-off window creation idempotent.
///
/// This intentionally lives outside `fret-core` (graph stays pure) and outside the UI widget
/// (covers duplicate ops emitted by runners/drivers or other app code).
#[derive(Default)]
struct DockTearOffMachine {
    pending_by_panel: std::collections::HashMap<fret_core::PanelKey, DockTearOffPending>,
}

impl DockTearOffMachine {
    // If a create request fails (e.g. backend error), we may never receive `window_created`.
    // Use a TTL so a later tear-off attempt can recover.
    const PENDING_TTL_TICKS: u64 = 600;

    fn prune_expired(&mut self, now: fret_runtime::TickId) {
        self.pending_by_panel.retain(|_, pending| {
            let age = now.0.saturating_sub(pending.requested_at.0);
            age <= Self::PENDING_TTL_TICKS
        });
    }

    fn register_request(
        &mut self,
        now: fret_runtime::TickId,
        source_window: AppWindowId,
        panel: &fret_core::PanelKey,
    ) -> bool {
        self.prune_expired(now);
        match self.pending_by_panel.get(panel) {
            Some(_) => false,
            None => {
                self.pending_by_panel.insert(
                    panel.clone(),
                    DockTearOffPending {
                        source_window,
                        requested_at: now,
                        canceled: false,
                    },
                );
                true
            }
        }
    }

    fn cancel_for_panel(&mut self, panel: &fret_core::PanelKey) {
        if let Some(pending) = self.pending_by_panel.get_mut(panel) {
            pending.canceled = true;
        }
    }

    fn complete_for_create_request(
        &mut self,
        request: &CreateWindowRequest,
        now: fret_runtime::TickId,
    ) -> DockTearOffCompletion {
        self.prune_expired(now);
        let CreateWindowKind::DockFloating {
            source_window,
            panel,
        } = &request.kind
        else {
            return DockTearOffCompletion::Proceed;
        };

        let Some(pending) = self.pending_by_panel.remove(panel) else {
            // If we can't correlate the request, default to proceeding; callers may still apply the
            // graph update if the panel exists.
            return DockTearOffCompletion::Proceed;
        };

        if pending.canceled || pending.source_window != *source_window {
            return DockTearOffCompletion::CancelAndCloseWindow;
        }

        DockTearOffCompletion::Proceed
    }
}

fn invalidate_windows<H: UiHost>(app: &mut H, windows: impl IntoIterator<Item = AppWindowId>) {
    DockInvalidationService::bump_windows(app, windows);
}

/// Request docking layout invalidation for the provided windows.
///
/// This is a small app-layer integration hook: it bumps the internal invalidation models that the
/// dock host observes, forcing a layout pass on the next frame.
pub fn request_dock_invalidation<H: UiHost>(
    app: &mut H,
    windows: impl IntoIterator<Item = AppWindowId>,
) {
    invalidate_windows(app, windows);
}

fn clamp_rect_to_bounds(rect: fret_core::Rect, bounds: fret_core::Rect) -> fret_core::Rect {
    let mut out = rect;
    if bounds.size.width.0 > 0.0 && bounds.size.height.0 > 0.0 {
        let min_x = bounds.origin.x.0;
        let min_y = bounds.origin.y.0;
        let max_x = bounds.origin.x.0 + (bounds.size.width.0 - out.size.width.0).max(0.0);
        let max_y = bounds.origin.y.0 + (bounds.size.height.0 - out.size.height.0).max(0.0);
        out.origin.x = fret_core::Px(out.origin.x.0.clamp(min_x, max_x.max(min_x)));
        out.origin.y = fret_core::Px(out.origin.y.0.clamp(min_y, max_y.max(min_y)));
    }
    out
}

fn default_in_window_float_rect<H: UiHost>(
    app: &H,
    target_window: AppWindowId,
    anchor: Option<fret_core::WindowAnchor>,
) -> fret_core::Rect {
    let bounds = app
        .global::<fret_core::WindowMetricsService>()
        .and_then(|svc| svc.inner_bounds(target_window))
        .unwrap_or_else(|| {
            fret_core::Rect::new(
                fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
                fret_core::Size::new(fret_core::Px(960.0), fret_core::Px(720.0)),
            )
        });

    let size = fret_core::Size::new(fret_core::Px(480.0), fret_core::Px(360.0));

    let origin = if let Some(anchor) = anchor {
        fret_core::Point::new(
            fret_core::Px(anchor.position.x.0 - size.width.0 * 0.25),
            fret_core::Px(anchor.position.y.0 - size.height.0 * 0.25),
        )
    } else {
        fret_core::Point::new(
            fret_core::Px(bounds.size.width.0 * 0.5 - size.width.0 * 0.5),
            fret_core::Px(bounds.size.height.0 * 0.5 - size.height.0 * 0.5),
        )
    };

    clamp_rect_to_bounds(fret_core::Rect::new(origin, size), bounds)
}

/// Handle a docking transaction emitted by the UI layer.
///
/// Call this from your runner/driver when consuming `Effect::Dock(op)`.
pub fn handle_dock_op<H: UiHost>(app: &mut H, op: DockOp) -> bool {
    match op {
        DockOp::RequestFloatPanelToNewWindow {
            source_window,
            panel,
            anchor,
        } => {
            if app.global::<DockManager>().is_none() {
                return false;
            }

            if let Some(caps) = app.global::<PlatformCapabilities>() {
                let tear_off_supported = caps.ui.multi_window
                    && caps.ui.window_tear_off
                    && caps.ui.window_hover_detection
                        != fret_runtime::WindowHoverDetectionQuality::None;
                if !tear_off_supported {
                    let target_window = anchor.map(|a| a.window).unwrap_or(source_window);
                    let rect = default_in_window_float_rect(app, target_window, anchor);
                    return handle_dock_op(
                        app,
                        DockOp::FloatPanelInWindow {
                            source_window,
                            panel,
                            target_window,
                            rect,
                        },
                    );
                }
            } else {
                let target_window = anchor.map(|a| a.window).unwrap_or(source_window);
                let rect = default_in_window_float_rect(app, target_window, anchor);
                return handle_dock_op(
                    app,
                    DockOp::FloatPanelInWindow {
                        source_window,
                        panel,
                        target_window,
                        rect,
                    },
                );
            }

            let now = app.tick_id();
            let should_emit = app.with_global_mut(DockTearOffMachine::default, |machine, _app| {
                machine.register_request(now, source_window, &panel)
            });
            if !should_emit {
                return true;
            }

            app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
                kind: CreateWindowKind::DockFloating {
                    source_window,
                    panel,
                },
                anchor,
                role: fret_runtime::WindowRole::Auxiliary,
                style: fret_runtime::WindowStyleRequest {
                    taskbar: Some(fret_runtime::TaskbarVisibility::Hide),
                    activation: Some(fret_runtime::ActivationPolicy::Activates),
                    z_level: None,
                },
            })));
            true
        }
        op => {
            if app.global::<DockManager>().is_none() {
                return false;
            }

            let maybe_close_window = match &op {
                DockOp::ClosePanel { window, .. } => Some(*window),
                DockOp::MovePanel { source_window, .. } => Some(*source_window),
                DockOp::MoveTabs { source_window, .. } => Some(*source_window),
                DockOp::FloatPanelToWindow { source_window, .. } => Some(*source_window),
                DockOp::FloatPanelInWindow { source_window, .. } => Some(*source_window),
                DockOp::FloatTabsInWindow { source_window, .. } => Some(*source_window),
                DockOp::MergeWindowInto { source_window, .. } => Some(*source_window),
                _ => None,
            }
            .filter(|w| {
                app.global::<DockFloatingOsWindowRegistry>()
                    .is_some_and(|reg| reg.contains(*w))
            });

            let mut should_auto_close = false;
            let handled = app.with_global_mut(DockManager::default, |dock, app| {
                let now = app.tick_id();
                app.with_global_mut(DockTearOffMachine::default, |machine, _app| match &op {
                    DockOp::ClosePanel { panel, .. }
                    | DockOp::MovePanel { panel, .. }
                    | DockOp::FloatPanelToWindow { panel, .. }
                    | DockOp::FloatPanelInWindow { panel, .. } => {
                        machine.prune_expired(now);
                        machine.cancel_for_panel(panel);
                    }
                    DockOp::MoveTabs { .. } | DockOp::FloatTabsInWindow { .. } => {
                        machine.prune_expired(now);
                    }
                    _ => machine.prune_expired(now),
                });

                let changed = dock.graph.apply_op(&op);
                if !changed {
                    return false;
                }

                if let Some(window) = maybe_close_window
                    && dock.graph.collect_panels_in_window(window).is_empty()
                {
                    should_auto_close = true;
                }

                match &op {
                    DockOp::MovePanel {
                        source_window,
                        target_window,
                        ..
                    } => {
                        dock.clear_viewport_layout_for_window(*source_window);
                        dock.clear_viewport_layout_for_window(*target_window);
                        invalidate_windows(app, [*source_window, *target_window]);
                    }
                    DockOp::MoveTabs {
                        source_window,
                        target_window,
                        ..
                    } => {
                        dock.clear_viewport_layout_for_window(*source_window);
                        dock.clear_viewport_layout_for_window(*target_window);
                        invalidate_windows(app, [*source_window, *target_window]);
                    }
                    DockOp::FloatPanelToWindow {
                        source_window,
                        new_window,
                        ..
                    } => {
                        dock.clear_viewport_layout_for_window(*source_window);
                        dock.clear_viewport_layout_for_window(*new_window);
                        invalidate_windows(app, [*source_window, *new_window]);
                    }
                    DockOp::FloatPanelInWindow {
                        source_window,
                        target_window,
                        ..
                    } => {
                        dock.clear_viewport_layout_for_window(*source_window);
                        dock.clear_viewport_layout_for_window(*target_window);
                        invalidate_windows(app, [*source_window, *target_window]);
                    }
                    DockOp::FloatTabsInWindow {
                        source_window,
                        target_window,
                        ..
                    } => {
                        dock.clear_viewport_layout_for_window(*source_window);
                        dock.clear_viewport_layout_for_window(*target_window);
                        invalidate_windows(app, [*source_window, *target_window]);
                    }
                    DockOp::SetFloatingRect { window, .. }
                    | DockOp::RaiseFloating { window, .. }
                    | DockOp::MergeFloatingInto { window, .. } => {
                        dock.clear_viewport_layout_for_window(*window);
                        invalidate_windows(app, [*window]);
                    }
                    DockOp::MergeWindowInto {
                        source_window,
                        target_window,
                        ..
                    } => {
                        dock.clear_viewport_layout_for_window(*source_window);
                        dock.clear_viewport_layout_for_window(*target_window);
                        invalidate_windows(app, [*source_window, *target_window]);
                    }
                    DockOp::ClosePanel { window, .. } => {
                        dock.clear_viewport_layout_for_window(*window);
                        invalidate_windows(app, [*window]);
                    }
                    DockOp::SetActiveTab { .. }
                    | DockOp::SetSplitFractions { .. }
                    | DockOp::SetSplitFractionsMany { .. }
                    | DockOp::SetSplitFractionTwo { .. } => {
                        invalidate_windows(app, dock.graph.windows());
                    }
                    DockOp::RequestFloatPanelToNewWindow { .. } => unreachable!(),
                }
                true
            });

            if handled
                && should_auto_close
                && let Some(window) = maybe_close_window
            {
                if std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_some_and(|v| !v.is_empty()) {
                    tracing::info!(
                        window = ?window,
                        op = ?op,
                        "dock tear-off: auto-close empty DockFloating window"
                    );
                }
                app.with_global_mut(DockFloatingOsWindowRegistry::default, |reg, _app| {
                    reg.remove(window);
                });
                app.push_effect(Effect::Window(WindowRequest::Close(window)));
            }

            handled
        }
    }
}

/// Complete a dock floating window creation by updating the dock graph.
///
/// Call this from your runner/driver `window_created(...)` callback.
pub fn handle_dock_window_created<H: UiHost>(
    app: &mut H,
    request: &CreateWindowRequest,
    new_window: AppWindowId,
) -> bool {
    let now = app.tick_id();
    let completion = app.with_global_mut(DockTearOffMachine::default, |machine, _app| {
        machine.complete_for_create_request(request, now)
    });
    if matches!(completion, DockTearOffCompletion::CancelAndCloseWindow) {
        if std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_some_and(|v| !v.is_empty()) {
            tracing::info!(
                new_window = ?new_window,
                request_kind = ?request.kind,
                "dock tear-off: cancel and close newly created window"
            );
        }
        app.push_effect(Effect::Window(WindowRequest::Close(new_window)));
        return true;
    }

    let CreateWindowKind::DockFloating {
        source_window,
        panel,
    } = &request.kind
    else {
        return false;
    };

    if app.global::<DockManager>().is_none() {
        if std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_some_and(|v| !v.is_empty()) {
            tracing::info!(
                new_window = ?new_window,
                request_kind = ?request.kind,
                "dock tear-off: missing DockManager; closing newly created window"
            );
        }
        app.push_effect(Effect::Window(WindowRequest::Close(new_window)));
        return true;
    }

    let handled = app.with_global_mut(DockManager::default, |dock, app| {
        let changed = dock
            .graph
            .float_panel_to_window(*source_window, panel.clone(), new_window);
        if !changed {
            return false;
        }

        if let Some(drag) = app.drag_mut(fret_core::PointerId(0))
            && drag.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            && drag.source_window == *source_window
        {
            drag.source_window = new_window;
            drag.current_window = new_window;
        }

        dock.clear_viewport_layout_for_window(*source_window);
        dock.clear_viewport_layout_for_window(new_window);
        invalidate_windows(app, [*source_window, new_window]);
        true
    });

    if handled {
        app.with_global_mut(DockFloatingOsWindowRegistry::default, |reg, _app| {
            reg.register(new_window);
        });
    }

    handled
}

/// Merge a closing floating dock window back into a target window.
///
/// This matches the common editor UX expectation that closing a floating dock window keeps its
/// panels alive by merging them into a stable target (usually the main window).
///
/// Call this from your runner/driver `before_close_window(...)` hook.
pub fn handle_dock_before_close_window<H: UiHost>(
    app: &mut H,
    closing_window: AppWindowId,
    target_window: AppWindowId,
) -> bool {
    if closing_window == target_window {
        return true;
    }
    if app.global::<DockManager>().is_none() {
        return true;
    }

    app.with_global_mut(DockFloatingOsWindowRegistry::default, |reg, _app| {
        reg.remove(closing_window);
    });

    app.with_global_mut(DockManager::default, |dock, app| {
        if dock.graph.window_root(closing_window).is_none() {
            return true;
        }
        let Some(target_tabs) = dock.graph.first_tabs_in_window(target_window) else {
            return true;
        };

        let _ = dock.graph.apply_op(&DockOp::MergeWindowInto {
            source_window: closing_window,
            target_window,
            target_tabs,
        });

        dock.clear_viewport_layout_for_window(closing_window);
        dock.clear_viewport_layout_for_window(target_window);
        invalidate_windows(app, [target_window]);
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::TestHost;
    use fret_core::{DockNode, DropZone, PanelKey};
    use slotmap::KeyData;

    #[test]
    fn request_float_creates_window_and_window_created_moves_panel() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let panel = PanelKey::new("test.panel");

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(
                panel.clone(),
                crate::DockPanel {
                    title: "Panel".to_string(),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                },
            );
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        let op = DockOp::RequestFloatPanelToNewWindow {
            source_window: window_a,
            panel: panel.clone(),
            anchor: None,
        };
        assert!(handle_dock_op(&mut app, op));

        let effects = app.take_effects();
        let create = effects
            .iter()
            .find_map(|e| match e {
                Effect::Window(WindowRequest::Create(req)) => Some(req.clone()),
                _ => None,
            })
            .expect("expected WindowRequest::Create");

        assert!(matches!(
            create.kind,
            CreateWindowKind::DockFloating { source_window, .. } if source_window == window_a
        ));
        assert_eq!(create.role, fret_runtime::WindowRole::Auxiliary);
        assert_eq!(
            create.style.taskbar,
            Some(fret_runtime::TaskbarVisibility::Hide)
        );
        assert_eq!(
            create.style.activation,
            Some(fret_runtime::ActivationPolicy::Activates)
        );

        assert!(handle_dock_window_created(&mut app, &create, window_b));

        let changed = app.take_changed_models();
        assert!(
            !changed.is_empty(),
            "expected docking invalidation to bump an observed model (MVP 66)"
        );

        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert!(
            dock.graph.find_panel_in_window(window_b, &panel).is_some(),
            "expected panel to be floated into the new window"
        );
        assert!(
            dock.graph.find_panel_in_window(window_a, &panel).is_none(),
            "expected panel to be removed from the source window"
        );
    }

    #[test]
    fn request_float_degrades_to_in_window_when_multi_window_is_disabled() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let panel = PanelKey::new("test.panel");

        let mut app = TestHost::new();
        let mut caps = PlatformCapabilities::default();
        caps.ui.multi_window = false;
        caps.ui.window_tear_off = true;
        app.set_global(caps);
        app.set_global(DockManager::default());

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(
                panel.clone(),
                crate::DockPanel {
                    title: "Panel".to_string(),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                },
            );
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        let op = DockOp::RequestFloatPanelToNewWindow {
            source_window: window_a,
            panel: panel.clone(),
            anchor: None,
        };
        assert!(handle_dock_op(&mut app, op));

        let effects = app.take_effects();
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::Window(WindowRequest::Create(_)))),
            "expected no OS window creation effect when multi-window is disabled"
        );

        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert_eq!(dock.graph.floating_windows(window_a).len(), 1);
        assert!(
            dock.graph.find_panel_in_window(window_a, &panel).is_some(),
            "expected panel to remain in window, inside a floating container"
        );
    }

    #[test]
    fn request_float_degrades_to_in_window_when_tear_off_is_disabled() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let panel = PanelKey::new("test.panel");

        let mut app = TestHost::new();
        let mut caps = PlatformCapabilities::default();
        caps.ui.multi_window = true;
        caps.ui.window_tear_off = false;
        app.set_global(caps);
        app.set_global(DockManager::default());

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(
                panel.clone(),
                crate::DockPanel {
                    title: "Panel".to_string(),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                },
            );
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        let op = DockOp::RequestFloatPanelToNewWindow {
            source_window: window_a,
            panel: panel.clone(),
            anchor: None,
        };
        assert!(handle_dock_op(&mut app, op));

        let effects = app.take_effects();
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, Effect::Window(WindowRequest::Create(_)))),
            "expected no OS window creation effect when tear-off is disabled"
        );

        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert_eq!(dock.graph.floating_windows(window_a).len(), 1);
        assert!(
            dock.graph.find_panel_in_window(window_a, &panel).is_some(),
            "expected panel to remain in window, inside a floating container"
        );
    }

    #[test]
    fn request_float_is_idempotent_until_window_created() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let panel = PanelKey::new("test.panel");

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(
                panel.clone(),
                crate::DockPanel {
                    title: "Panel".to_string(),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                },
            );
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        let op = DockOp::RequestFloatPanelToNewWindow {
            source_window: window_a,
            panel: panel.clone(),
            anchor: None,
        };
        assert!(handle_dock_op(&mut app, op.clone()));
        assert!(handle_dock_op(&mut app, op));

        let effects = app.take_effects();
        let create_count = effects
            .iter()
            .filter(|e| matches!(e, Effect::Window(WindowRequest::Create(_))))
            .count();
        assert_eq!(create_count, 1, "expected at most one create request");
    }

    #[test]
    fn window_created_updates_drag_source_window_for_active_dock_drag() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let panel = PanelKey::new("test.panel");

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(
                panel.clone(),
                crate::DockPanel {
                    title: "Panel".to_string(),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                },
            );
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        app.begin_cross_window_drag_with_kind(
            fret_core::PointerId(0),
            fret_runtime::DRAG_KIND_DOCK_PANEL,
            window_a,
            fret_core::Point::default(),
            (),
        );

        let op = DockOp::RequestFloatPanelToNewWindow {
            source_window: window_a,
            panel: panel.clone(),
            anchor: None,
        };
        assert!(handle_dock_op(&mut app, op));

        let effects = app.take_effects();
        let create = effects
            .iter()
            .find_map(|e| match e {
                Effect::Window(WindowRequest::Create(req)) => Some(req.clone()),
                _ => None,
            })
            .expect("expected WindowRequest::Create");

        assert!(handle_dock_window_created(&mut app, &create, window_b));

        let drag = app
            .drag(fret_core::PointerId(0))
            .expect("expected active drag session");
        assert_eq!(drag.source_window, window_b);
        assert_eq!(drag.current_window, window_b);
    }

    #[test]
    fn redock_from_dock_floating_window_auto_closes_empty_os_window() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let panel = PanelKey::new("test.panel");

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(
                panel.clone(),
                crate::DockPanel {
                    title: "Panel".to_string(),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                },
            );
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        assert!(handle_dock_op(
            &mut app,
            DockOp::RequestFloatPanelToNewWindow {
                source_window: window_a,
                panel: panel.clone(),
                anchor: None,
            }
        ));

        let create = app
            .take_effects()
            .iter()
            .find_map(|e| match e {
                Effect::Window(WindowRequest::Create(req)) => Some(req.clone()),
                _ => None,
            })
            .expect("expected WindowRequest::Create");

        assert!(handle_dock_window_created(&mut app, &create, window_b));
        app.take_effects();

        let target_tabs = app
            .global::<DockManager>()
            .expect("dock manager exists")
            .graph
            .first_tabs_in_window(window_a)
            .expect("expected a target tabs node in the main window");

        assert!(handle_dock_op(
            &mut app,
            DockOp::MovePanel {
                source_window: window_b,
                panel: panel.clone(),
                target_window: window_a,
                target_tabs,
                zone: DropZone::Center,
                insert_index: None,
            }
        ));

        let effects = app.take_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::Window(WindowRequest::Close(w)) if *w == window_b)),
            "expected an auto-close request for the now-empty dock-floating OS window"
        );

        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert!(
            dock.graph.collect_panels_in_window(window_b).is_empty(),
            "expected the source window to be empty after re-docking its last panel"
        );
        assert!(
            dock.graph.find_panel_in_window(window_a, &panel).is_some(),
            "expected the panel to be present in the target window after re-dock"
        );
    }

    #[test]
    fn before_close_window_merges_dock_floating_panels_into_target_window() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let panel = PanelKey::new("test.panel");

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(
                panel.clone(),
                crate::DockPanel {
                    title: "Panel".to_string(),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                },
            );

            let tabs_a = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![PanelKey::new("main.placeholder")],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs_a);

            let tabs_b = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_b, tabs_b);
        });

        assert!(
            handle_dock_before_close_window(&mut app, window_b, window_a),
            "expected before_close hook to allow closing after merging"
        );

        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert!(
            dock.graph.window_root(window_b).is_none(),
            "expected closing window root to be removed after merge"
        );
        assert!(
            dock.graph.find_panel_in_window(window_a, &panel).is_some(),
            "expected panel to be merged into target window"
        );
    }

    #[test]
    fn request_float_canceled_by_close_panel_closes_created_window() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let panel = PanelKey::new("test.panel");

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(
                panel.clone(),
                crate::DockPanel {
                    title: "Panel".to_string(),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                },
            );
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        let op = DockOp::RequestFloatPanelToNewWindow {
            source_window: window_a,
            panel: panel.clone(),
            anchor: None,
        };
        assert!(handle_dock_op(&mut app, op));

        let effects = app.take_effects();
        let create = effects
            .iter()
            .find_map(|e| match e {
                Effect::Window(WindowRequest::Create(req)) => Some(req.clone()),
                _ => None,
            })
            .expect("expected WindowRequest::Create");

        assert!(handle_dock_op(
            &mut app,
            DockOp::ClosePanel {
                window: window_a,
                panel: panel.clone(),
            }
        ));

        assert!(handle_dock_window_created(&mut app, &create, window_b));

        let effects = app.take_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::Window(WindowRequest::Close(w)) if *w == window_b)),
            "expected the created window to be closed after cancelation"
        );

        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert!(
            dock.graph.find_panel_in_window(window_b, &panel).is_none(),
            "expected panel not to be moved after cancelation"
        );
    }

    #[test]
    fn window_created_does_not_update_drag_source_when_canceled() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let panel = PanelKey::new("test.panel");

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());

        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(
                panel.clone(),
                crate::DockPanel {
                    title: "Panel".to_string(),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                },
            );
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel.clone()],
                active: 0,
            });
            dock.graph.set_window_root(window_a, tabs);
        });

        app.begin_cross_window_drag_with_kind(
            fret_core::PointerId(0),
            fret_runtime::DRAG_KIND_DOCK_PANEL,
            window_a,
            fret_core::Point::default(),
            (),
        );

        let op = DockOp::RequestFloatPanelToNewWindow {
            source_window: window_a,
            panel: panel.clone(),
            anchor: None,
        };
        assert!(handle_dock_op(&mut app, op));

        let effects = app.take_effects();
        let create = effects
            .iter()
            .find_map(|e| match e {
                Effect::Window(WindowRequest::Create(req)) => Some(req.clone()),
                _ => None,
            })
            .expect("expected WindowRequest::Create");

        assert!(handle_dock_op(
            &mut app,
            DockOp::ClosePanel {
                window: window_a,
                panel: panel.clone(),
            }
        ));

        assert!(handle_dock_window_created(&mut app, &create, window_b));

        let drag = app
            .drag(fret_core::PointerId(0))
            .expect("expected active drag session");
        assert_eq!(drag.source_window, window_a);
        assert_eq!(drag.current_window, window_a);
    }
}
