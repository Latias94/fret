//! App/runner integration helpers for docking.
//!
//! The docking UI emits high-level `DockOp` transactions via `Effect::Dock(...)` (ADR 0013).
//! These ops must be applied by the app/runner layer:
//! - apply graph mutations (`DockGraph::apply_op`)
//! - translate `RequestFloatPanelToNewWindow` into a `WindowRequest::Create`
//! - translate `RequestFloatTabsToNewWindow` into a `WindowRequest::Create`
//! - complete the float by updating the graph once the OS window exists

use fret_core::{AppWindowId, DockOp};
use fret_runtime::{CreateWindowRequest, UiHost};

use crate::DockManager;
use crate::invalidation::DockInvalidationService;

mod auto_close;
mod before_close;
mod in_window;
mod request;
mod tear_off;
mod window_created;

pub use in_window::recenter_in_window_floatings;
use tear_off::DockTearOffMachine;
pub(crate) use tear_off::is_dock_floating_os_window;

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

/// Handle a docking transaction emitted by the UI layer.
///
/// Call this from your runner/driver when consuming `Effect::Dock(op)`.
pub fn handle_dock_op<H: UiHost>(app: &mut H, op: DockOp) -> bool {
    match op {
        op @ DockOp::RequestFloatPanelToNewWindow { .. } => {
            request::handle_request_float_to_new_window(app, op)
        }
        op @ DockOp::RequestFloatTabsToNewWindow { .. } => {
            request::handle_request_float_to_new_window(app, op)
        }
        op => {
            if app.global::<DockManager>().is_none() {
                return false;
            }

            let tearoff_log =
                std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_some_and(|v| !v.is_empty());
            let mut windows_to_auto_close: Vec<AppWindowId> = Vec::new();
            let handled = app.with_global_mut(DockManager::default, |dock, app| {
                let now = app.tick_id();
                app.with_global_mut(DockTearOffMachine::default, |machine, _app| {
                    machine.prune_and_cancel_for_op(now, dock, &op);
                });

                let changed = dock.graph.apply_op(&op);
                if !changed {
                    return false;
                }

                if tearoff_log {
                    match &op {
                        DockOp::MovePanel {
                            source_window,
                            target_window,
                            ..
                        }
                        | DockOp::MoveTabs {
                            source_window,
                            target_window,
                            ..
                        }
                        | DockOp::MergeWindowInto {
                            source_window,
                            target_window,
                            ..
                        } => {
                            let src_panels = dock.graph.collect_panels_in_window(*source_window);
                            let tgt_panels = dock.graph.collect_panels_in_window(*target_window);
                            tracing::info!(
                                op = ?op,
                                source_window = ?source_window,
                                target_window = ?target_window,
                                source_panel_count = src_panels.len(),
                                target_panel_count = tgt_panels.len(),
                                "dock tear-off: applied cross-window dock op"
                            );
                        }
                        _ => {}
                    }
                }

                windows_to_auto_close =
                    auto_close::collect_empty_dock_floating_windows(app, dock, tearoff_log);

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
                    DockOp::MovePanelToEmptyDockSpace {
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
                    DockOp::MoveTabsToEmptyDockSpace {
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
                    DockOp::RequestFloatPanelToNewWindow { .. }
                    | DockOp::RequestFloatTabsToNewWindow { .. } => unreachable!(),
                }
                true
            });

            auto_close::close_empty_dock_floating_windows(app, &op, windows_to_auto_close);

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
    window_created::handle_dock_window_created(app, request, new_window)
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
    before_close::handle_dock_before_close_window(app, closing_window, target_window)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_host::TestHost;
    use fret_core::{DockNode, DropZone, PanelKey};
    use fret_runtime::{CreateWindowKind, Effect, PlatformCapabilities, WindowRequest};
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
    fn request_float_degrades_to_in_window_when_window_hover_detection_is_none() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let panel = PanelKey::new("test.panel");

        let mut app = TestHost::new();
        let mut caps = PlatformCapabilities::default();
        caps.ui.multi_window = true;
        caps.ui.window_tear_off = true;
        caps.ui.window_hover_detection = fret_runtime::WindowHoverDetectionQuality::None;
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
            "expected no OS window creation effect when window hover detection is unavailable"
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
        let pointer_id = fret_core::PointerId(7);

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
            pointer_id,
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

        let drag = app.drag(pointer_id).expect("expected active drag session");
        assert_eq!(drag.source_window, window_b);
        assert_eq!(drag.current_window, window_b);
    }

    #[test]
    fn window_created_updates_drag_source_window_for_active_dock_tabs_drag() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let panel_a = PanelKey::new("test.a");
        let panel_b = PanelKey::new("test.b");
        let pointer_id = fret_core::PointerId(9);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());
        app.set_global(DockManager::default());

        let source_tabs = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.insert_panel(
                panel_a.clone(),
                crate::DockPanel {
                    title: "A".to_string(),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                },
            );
            dock.insert_panel(
                panel_b.clone(),
                crate::DockPanel {
                    title: "B".to_string(),
                    color: fret_core::Color::TRANSPARENT,
                    viewport: None,
                },
            );
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![panel_a.clone(), panel_b.clone()],
                active: 1,
            });
            dock.graph.set_window_root(window_a, tabs);
            tabs
        });

        app.begin_cross_window_drag_with_kind(
            pointer_id,
            fret_runtime::DRAG_KIND_DOCK_TABS,
            window_a,
            fret_core::Point::default(),
            (),
        );

        let op = DockOp::RequestFloatTabsToNewWindow {
            source_window: window_a,
            source_tabs,
            panel: panel_b.clone(),
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

        let drag = app.drag(pointer_id).expect("expected active drag session");
        assert_eq!(drag.source_window, window_b);
        assert_eq!(drag.current_window, window_b);

        let dock = app.global::<DockManager>().expect("dock manager exists");
        assert!(
            dock.graph.collect_panels_in_window(window_a).is_empty(),
            "expected source window to be empty after floating tabs to new window"
        );
        assert_eq!(
            dock.graph.collect_panels_in_window(window_b),
            vec![panel_a, panel_b],
            "expected all tabs to be moved to the new window"
        );
    }

    #[test]
    fn window_created_prefers_pending_pointer_id_over_drag_source_window_match() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let window_c = AppWindowId::from(KeyData::from_ffi(3));
        let panel = PanelKey::new("test.panel");
        let pointer_id = fret_core::PointerId(9);

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
            pointer_id,
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

        // Simulate a runner/UI interaction mutating the drag session's source_window before the
        // create callback arrives. The tear-off completion should still update the session by
        // pointer_id.
        if let Some(drag) = app.drag_mut(pointer_id) {
            drag.source_window = window_c;
            drag.current_window = window_c;
        }

        assert!(handle_dock_window_created(&mut app, &create, window_b));

        let drag = app.drag(pointer_id).expect("expected active drag session");
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

            // Keep a stable target tabs node in the main window even if the only "real" panel is
            // temporarily floated away. Canonicalization in `fret-core` prunes empty tabs nodes,
            // so tests should avoid assuming an empty root tabs survives as a drop target.
            let tabs = dock.graph.insert_node(DockNode::Tabs {
                tabs: vec![PanelKey::new("main.placeholder"), panel.clone()],
                active: 1,
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
