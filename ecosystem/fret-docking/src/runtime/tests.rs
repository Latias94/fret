use super::*;
use crate::DockManager;
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
