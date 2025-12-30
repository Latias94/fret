//! App/runner integration helpers for docking.
//!
//! The docking UI emits high-level `DockOp` transactions via `Effect::Dock(...)` (ADR 0013).
//! These ops must be applied by the app/runner layer:
//! - apply graph mutations (`DockGraph::apply_op`)
//! - translate `RequestFloatPanelToNewWindow` into a `WindowRequest::Create`
//! - complete the float by updating the graph once the OS window exists

use fret_core::{AppWindowId, DockOp};
use fret_runtime::{CreateWindowKind, CreateWindowRequest, Effect, UiHost, WindowRequest};

use crate::DockManager;
use crate::invalidation::DockInvalidationService;

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
            if let Some(caps) = app.global::<fret_core::PlatformCapabilities>()
                && !caps.ui.multi_window
            {
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

            app.push_effect(Effect::Window(WindowRequest::Create(CreateWindowRequest {
                kind: CreateWindowKind::DockFloating {
                    source_window,
                    panel,
                },
                anchor,
            })));
            true
        }
        op => {
            if app.global::<DockManager>().is_none() {
                return false;
            }

            app.with_global_mut(DockManager::default, |dock, app| {
                let changed = dock.graph.apply_op(&op);
                if !changed {
                    return false;
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
            })
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
    let CreateWindowKind::DockFloating {
        source_window,
        panel,
    } = &request.kind
    else {
        return false;
    };

    if app.global::<DockManager>().is_none() {
        return false;
    }

    app.with_global_mut(DockManager::default, |dock, app| {
        let changed = dock
            .graph
            .float_panel_to_window(*source_window, panel.clone(), new_window);
        if !changed {
            return false;
        }

        dock.clear_viewport_layout_for_window(*source_window);
        dock.clear_viewport_layout_for_window(new_window);
        invalidate_windows(app, [*source_window, new_window]);
        true
    })
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
    use fret_core::{DockNode, PanelKey};
    use slotmap::KeyData;

    #[test]
    fn request_float_creates_window_and_window_created_moves_panel() {
        let window_a = AppWindowId::from(KeyData::from_ffi(1));
        let window_b = AppWindowId::from(KeyData::from_ffi(2));
        let panel = PanelKey::new("test.panel");

        let mut app = TestHost::new();
        app.set_global(fret_core::PlatformCapabilities::default());
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
        let mut caps = fret_core::PlatformCapabilities::default();
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
}
