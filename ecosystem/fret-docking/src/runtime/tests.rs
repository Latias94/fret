use super::*;
use crate::DockManager;
use crate::test_host::TestHost;
use fret_core::{DockNode, DropZone, PanelKey};
use fret_runtime::{
    CreateWindowKind, CreateWindowRequest, Effect, PlatformCapabilities, WindowRequest,
    WindowStyleRequest,
};
use slotmap::KeyData;

fn test_panel(title: &str) -> crate::DockPanel {
    crate::DockPanel {
        title: title.to_string(),
        color: fret_core::Color::TRANSPARENT,
        viewport: None,
    }
}

fn dock_floating_create_request(
    source_window: AppWindowId,
    panel: PanelKey,
) -> CreateWindowRequest {
    CreateWindowRequest {
        kind: CreateWindowKind::DockFloating {
            source_window,
            panel,
        },
        anchor: None,
        role: fret_runtime::WindowRole::Auxiliary,
        style: WindowStyleRequest::default(),
    }
}

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
fn request_float_without_dock_manager_fails_closed_without_effects() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let handled = handle_dock_op(
        &mut app,
        DockOp::RequestFloatPanelToNewWindow {
            source_window: window_a,
            panel,
            anchor: None,
        },
    );

    assert!(
        !handled,
        "missing DockManager should fail closed instead of creating a floating OS window"
    );
    assert!(
        app.take_effects().is_empty(),
        "missing DockManager should not emit any window effects"
    );
}

#[test]
fn window_created_without_dock_manager_closes_created_window() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");
    let request = dock_floating_create_request(window_a, panel);

    let mut app = TestHost::new();

    assert!(handle_dock_window_created(&mut app, &request, window_b));

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Window(WindowRequest::Close(w)) if *w == window_b)),
        "missing DockManager should close the newly-created floating window"
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
fn request_float_tabs_degrades_to_in_window_when_multi_window_is_disabled() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");

    let mut app = TestHost::new();
    let mut caps = PlatformCapabilities::default();
    caps.ui.multi_window = false;
    caps.ui.window_tear_off = true;
    app.set_global(caps);
    app.set_global(DockManager::default());

    let source_tabs = app.with_global_mut(DockManager::default, |dock, _app| {
        dock.insert_panel(panel_a.clone(), test_panel("A"));
        dock.insert_panel(panel_b.clone(), test_panel("B"));
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone(), panel_b.clone()],
            active: 1,
        });
        dock.graph.set_window_root(window_a, tabs);
        tabs
    });

    let op = DockOp::RequestFloatTabsToNewWindow {
        source_window: window_a,
        source_tabs,
        panel: panel_b.clone(),
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
    assert_eq!(
        dock.graph.collect_panels_in_window(window_a),
        vec![panel_a, panel_b],
        "expected all dragged tabs to remain in window inside a floating container"
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
fn request_float_tabs_is_idempotent_until_window_created() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());

    let source_tabs = app.with_global_mut(DockManager::default, |dock, _app| {
        dock.insert_panel(panel_a.clone(), test_panel("A"));
        dock.insert_panel(panel_b.clone(), test_panel("B"));
        let tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel_a.clone(), panel_b.clone()],
            active: 1,
        });
        dock.graph.set_window_root(window_a, tabs);
        tabs
    });

    let op = DockOp::RequestFloatTabsToNewWindow {
        source_window: window_a,
        source_tabs,
        panel: panel_b,
        anchor: None,
    };
    assert!(handle_dock_op(&mut app, op.clone()));
    assert!(handle_dock_op(&mut app, op));

    let effects = app.take_effects();
    let create_count = effects
        .iter()
        .filter(|e| matches!(e, Effect::Window(WindowRequest::Create(_))))
        .count();
    assert_eq!(
        create_count, 1,
        "expected at most one tab-stack create request"
    );
}

#[test]
fn expired_pending_request_allows_later_float_request() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.insert_panel(panel.clone(), test_panel("Panel"));
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
    let first_effects = app.take_effects();
    assert_eq!(
        first_effects
            .iter()
            .filter(|e| matches!(e, Effect::Window(WindowRequest::Create(_))))
            .count(),
        1,
        "expected the first request to create a floating OS window"
    );

    app.advance_ticks(601);
    assert!(handle_dock_op(&mut app, op));
    let second_effects = app.take_effects();
    assert_eq!(
        second_effects
            .iter()
            .filter(|e| matches!(e, Effect::Window(WindowRequest::Create(_))))
            .count(),
        1,
        "expired pending request should not suppress a later request forever"
    );
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
fn window_created_for_stale_source_request_closes_created_window_without_moving_panel() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let stale_source = AppWindowId::from(KeyData::from_ffi(3));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.insert_panel(panel.clone(), test_panel("Panel"));
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

    let mut request = app
        .take_effects()
        .iter()
        .find_map(|e| match e {
            Effect::Window(WindowRequest::Create(req)) => Some(req.clone()),
            _ => None,
        })
        .expect("expected WindowRequest::Create");
    request.kind = CreateWindowKind::DockFloating {
        source_window: stale_source,
        panel: panel.clone(),
    };

    assert!(handle_dock_window_created(&mut app, &request, window_b));

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Window(WindowRequest::Close(w)) if *w == window_b)),
        "stale source mismatch should close the created window"
    );
    let dock = app.global::<DockManager>().expect("dock manager exists");
    assert!(
        dock.graph.find_panel_in_window(window_a, &panel).is_some(),
        "stale source mismatch should preserve the panel in the original source window"
    );
    assert!(
        dock.graph.find_panel_in_window(window_b, &panel).is_none(),
        "stale source mismatch should not move the panel into the new window"
    );
}

#[test]
fn window_created_graph_commit_failure_closes_created_window() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.insert_panel(panel.clone(), test_panel("Panel"));
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

    let request = app
        .take_effects()
        .iter()
        .find_map(|e| match e {
            Effect::Window(WindowRequest::Create(req)) => Some(req.clone()),
            _ => None,
        })
        .expect("expected WindowRequest::Create");

    app.with_global_mut(DockManager::default, |dock, _app| {
        assert!(
            dock.graph.close_panel(window_a, panel.clone()),
            "test setup should remove the source panel without notifying the tear-off machine"
        );
    });

    assert!(handle_dock_window_created(&mut app, &request, window_b));

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Window(WindowRequest::Close(w)) if *w == window_b)),
        "graph commit failure should close the created dock-floating window"
    );
    let dock = app.global::<DockManager>().expect("dock manager exists");
    assert!(
        dock.graph.find_panel_in_window(window_b, &panel).is_none(),
        "failed graph commit must not invent the panel in the new window"
    );
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
fn before_close_window_without_target_tabs_moves_closing_root_to_target_window() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());

    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.insert_panel(panel.clone(), test_panel("Panel"));

        let tabs_b = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window_b, tabs_b);
    });

    assert!(
        handle_dock_before_close_window(&mut app, window_b, window_a),
        "expected before_close hook to allow closing after preserving the dock root"
    );

    let dock = app.global::<DockManager>().expect("dock manager exists");
    assert!(
        dock.graph.window_root(window_b).is_none(),
        "expected closing window root to be removed"
    );
    assert!(
        dock.graph.window_root(window_a).is_some(),
        "expected target window to receive the closing dock root"
    );
    assert!(
        dock.graph.find_panel_in_window(window_a, &panel).is_some(),
        "expected panel to survive in the target window"
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
fn request_float_canceled_by_move_panel_closes_created_window_and_preserves_moved_panel() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let window_c = AppWindowId::from(KeyData::from_ffi(3));
    let panel = PanelKey::new("test.panel");

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());

    let target_tabs = app.with_global_mut(DockManager::default, |dock, _app| {
        dock.insert_panel(panel.clone(), test_panel("Panel"));
        let source_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.graph.set_window_root(window_a, source_tabs);
        let target_tabs = dock.graph.insert_node(DockNode::Tabs {
            tabs: vec![PanelKey::new("target.placeholder")],
            active: 0,
        });
        dock.graph.set_window_root(window_c, target_tabs);
        target_tabs
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

    assert!(handle_dock_op(
        &mut app,
        DockOp::MovePanel {
            source_window: window_a,
            panel: panel.clone(),
            target_window: window_c,
            target_tabs,
            zone: DropZone::Center,
            insert_index: None,
        }
    ));

    assert!(handle_dock_window_created(&mut app, &create, window_b));

    let effects = app.take_effects();
    assert!(
        effects
            .iter()
            .any(|e| matches!(e, Effect::Window(WindowRequest::Close(w)) if *w == window_b)),
        "expected the created window to close after the pending request was canceled"
    );

    let dock = app.global::<DockManager>().expect("dock manager exists");
    assert!(
        dock.graph.find_panel_in_window(window_c, &panel).is_some(),
        "panel should remain in the window it was moved to while the request was pending"
    );
    assert!(
        dock.graph.find_panel_in_window(window_b, &panel).is_none(),
        "canceled pending request must not steal the moved panel into the new window"
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
