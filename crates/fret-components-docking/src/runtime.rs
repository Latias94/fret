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
                    DockOp::SetActiveTab { .. } | DockOp::SetSplitFractionTwo { .. } => {
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
}
