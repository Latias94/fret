use super::*;
use crate::dock::{DockManager, DockPanel, DockViewportLayout};
use crate::runtime::DockRuntimeCommand;
use crate::test_host::TestHost;
use fret_core::{
    AppWindowId, DockLayout, DockLayoutNode, DockLayoutWindow, DockNode, DockOp,
    DockWindowPlacement, DropZone, PanelKey, Point, Px, Rect, RenderTargetId, Size, ViewportFit,
    ViewportMapping,
};
use fret_runtime::{CreateWindowKind, Effect, PlatformCapabilities, WindowRequest};
use slotmap::KeyData;

fn test_panel(title: &str) -> DockPanel {
    DockPanel {
        title: title.to_string(),
        color: fret_core::Color::TRANSPARENT,
        viewport: None,
    }
}

fn render_target(id: u64) -> RenderTargetId {
    RenderTargetId::from(KeyData::from_ffi(id))
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(width), Px(height)))
}

fn viewport_layout(rect: Rect) -> DockViewportLayout {
    let mapping = ViewportMapping {
        content_rect: rect,
        target_px_size: (320, 240),
        fit: ViewportFit::Stretch,
    };
    DockViewportLayout {
        content_rect: rect,
        mapping,
        draw_rect: mapping.map().draw_rect,
    }
}

fn queued_create_panel(command: &DockRuntimeCommand) -> Option<(AppWindowId, PanelKey)> {
    let DockRuntimeCommand::CreateWindow(create) = command else {
        return None;
    };
    let CreateWindowKind::DockFloating {
        source_window,
        panel,
    } = &create.kind
    else {
        return None;
    };
    Some((*source_window, panel.clone()))
}

fn queued_create_request(
    commands: &[DockRuntimeCommand],
    source_window: AppWindowId,
    panel: &PanelKey,
) -> fret_runtime::CreateWindowRequest {
    commands
        .iter()
        .find_map(|command| {
            let DockRuntimeCommand::CreateWindow(create) = command else {
                return None;
            };
            let CreateWindowKind::DockFloating {
                source_window: create_source,
                panel: create_panel,
            } = &create.kind
            else {
                return None;
            };
            (*create_source == source_window && create_panel == panel).then(|| create.clone())
        })
        .expect("expected queued dock-floating create request")
}

#[test]
fn dock_surface_register_panel_reports_duplicate_keys() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.duplicate");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("First"))
        .expect("initial panel registers");

    let error = surface
        .register_panel(&mut app, panel.clone(), test_panel("Second"))
        .expect_err("duplicate key should be reported");

    assert_eq!(error, DockSurfacePanelError::DuplicatePanelKey { panel });
}

#[test]
fn dock_surface_panel_commands_return_typed_outcomes() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel_a.clone(), test_panel("A"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, panel_b.clone(), test_panel("B"))
        .expect("panel registers");

    let opened_a = surface
        .open_panel(&mut app, &panel_a)
        .expect("open panel a");
    assert_eq!(opened_a.panel, panel_a);
    assert_eq!(opened_a.change, DockSurfaceChange::Changed);
    assert_eq!(
        opened_a.location,
        Some(DockSurfacePanelLocation {
            window,
            placement: DockSurfacePanelPlacement::Docked,
            tab_index: 0,
            tab_count: 1,
            active: true,
        })
    );

    let opened_b = surface
        .open_panel(&mut app, &panel_b)
        .expect("open panel b");
    assert_eq!(opened_b.change, DockSurfaceChange::Changed);
    assert_eq!(
        surface.selected_panel_in_window(&app, window),
        Some(panel_b.clone())
    );

    let selected_a = surface
        .select_panel(&mut app, &panel_a)
        .expect("select existing panel");
    assert!(
        selected_a
            .location
            .as_ref()
            .is_some_and(|location| location.active && location.tab_index == 0)
    );
    assert_eq!(
        surface.selected_panel_in_window(&app, window),
        Some(panel_a.clone())
    );

    let closed_a = surface
        .close_panel(&mut app, &panel_a)
        .expect("close selected panel");
    assert_eq!(closed_a.change, DockSurfaceChange::Changed);
    assert_eq!(closed_a.location, None);
    assert_eq!(
        surface.close_panel(&mut app, &panel_a),
        Err(DockSurfacePanelError::PanelNotOpen { panel: panel_a })
    );
}

#[test]
fn dock_surface_panel_commands_report_unchanged_for_noops() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");

    surface.open_panel(&mut app, &panel).expect("open panel");

    let selected = surface
        .select_panel(&mut app, &panel)
        .expect("select already-active panel");
    assert_eq!(selected.change, DockSurfaceChange::Unchanged);

    let opened = surface
        .open_panel(&mut app, &panel)
        .expect("open already-active panel");
    assert_eq!(opened.change, DockSurfaceChange::Unchanged);
    assert_eq!(
        surface
            .panels_in_window(&app, window)
            .iter()
            .filter(|snapshot| snapshot.key == panel)
            .count(),
        1,
        "semantic open must not duplicate an already-open panel"
    );
}

#[test]
fn dock_surface_panel_commands_select_existing_panel_across_windows() {
    let main_window = AppWindowId::from(KeyData::from_ffi(1));
    let other_window = AppWindowId::from(KeyData::from_ffi(2));
    let main_panel = PanelKey::new("test.main");
    let other_panel = PanelKey::new("test.other");
    let surface = DockSurface::new(main_window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, main_panel.clone(), test_panel("Main"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, other_panel.clone(), test_panel("Other"))
        .expect("panel registers");
    surface
        .open_panel_with_preferred_window(&mut app, main_window, &main_panel)
        .expect("open main panel");
    surface
        .open_panel_with_preferred_window(&mut app, other_window, &other_panel)
        .expect("open other panel");

    let opened = surface
        .open_panel_with_preferred_window(&mut app, main_window, &other_panel)
        .expect("open existing other panel from main window");

    assert_eq!(opened.change, DockSurfaceChange::Unchanged);
    assert_eq!(
        opened.location.as_ref().map(|location| location.window),
        Some(other_window)
    );
    assert_eq!(
        surface
            .panels_in_window(&app, main_window)
            .iter()
            .filter(|snapshot| snapshot.key == other_panel)
            .count(),
        0,
        "open_panel_with_preferred_window must select the existing owner instead of copying across windows"
    );
    assert_eq!(
        surface
            .panels_in_window(&app, other_window)
            .iter()
            .filter(|snapshot| snapshot.key == other_panel)
            .count(),
        1
    );
}

#[test]
fn dock_surface_preferred_open_invalidates_actual_owner_window() {
    let main_window = AppWindowId::from(KeyData::from_ffi(1));
    let other_window = AppWindowId::from(KeyData::from_ffi(2));
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let surface = DockSurface::new(main_window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel_a.clone(), test_panel("A"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, panel_b.clone(), test_panel("B"))
        .expect("panel registers");
    surface
        .open_panel_with_preferred_window(&mut app, other_window, &panel_a)
        .expect("open panel a in other window");
    surface
        .open_panel_with_preferred_window(&mut app, other_window, &panel_b)
        .expect("open panel b in other window");
    let main_target = render_target(201);
    let other_target = render_target(202);
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.set_viewport_layout(
            main_window,
            main_target,
            viewport_layout(rect(0.0, 0.0, 320.0, 240.0)),
        );
        dock.set_viewport_layout(
            other_window,
            other_target,
            viewport_layout(rect(10.0, 10.0, 320.0, 240.0)),
        );
    });
    app.take_redraws();

    let outcome = surface
        .open_panel_with_preferred_window(&mut app, main_window, &panel_a)
        .expect("select panel a from preferred main window");

    assert_eq!(outcome.change, DockSurfaceChange::Changed);
    assert_eq!(
        outcome.location.as_ref().map(|location| location.window),
        Some(other_window)
    );
    let redraws = app.take_redraws();
    assert!(
        redraws.contains(&other_window),
        "selecting an existing panel must invalidate its actual owner window"
    );
    assert!(
        redraws.contains(&main_window),
        "preferred-window command should also invalidate the preferred window"
    );
    let dock = app.global::<DockManager>().expect("dock manager exists");
    assert_eq!(dock.viewport_layout(main_window, main_target), None);
    assert_eq!(dock.viewport_layout(other_window, other_target), None);
}

#[test]
fn dock_surface_panel_commands_report_typed_errors() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let registered = PanelKey::new("test.registered");
    let unopened = PanelKey::new("test.unopened");
    let missing = PanelKey::new("test.missing");
    let surface = DockSurface::new(window);

    let mut no_manager = TestHost::new();
    assert_eq!(
        surface.try_registered_panels(&no_manager),
        Err(DockSurfacePanelError::DockManagerUnavailable)
    );
    assert_eq!(
        surface.try_panels_in_window(&no_manager, window),
        Err(DockSurfacePanelError::DockManagerUnavailable)
    );
    assert_eq!(
        surface.try_selected_panel_in_window(&no_manager, window),
        Err(DockSurfacePanelError::DockManagerUnavailable)
    );
    assert_eq!(
        surface.try_panel_location(&no_manager, &registered),
        Err(DockSurfacePanelError::DockManagerUnavailable)
    );
    assert_eq!(
        surface.open_panel(&mut no_manager, &registered),
        Err(DockSurfacePanelError::DockManagerUnavailable)
    );
    assert_eq!(
        surface.select_panel(&mut no_manager, &registered),
        Err(DockSurfacePanelError::DockManagerUnavailable)
    );
    assert_eq!(
        surface.close_panel(&mut no_manager, &registered),
        Err(DockSurfacePanelError::DockManagerUnavailable)
    );

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, registered.clone(), test_panel("Registered"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, unopened.clone(), test_panel("Unopened"))
        .expect("panel registers");
    surface
        .open_panel(&mut app, &registered)
        .expect("open registered panel");

    assert_eq!(
        surface.open_panel(&mut app, &missing),
        Err(DockSurfacePanelError::PanelNotRegistered { panel: missing })
    );
    assert_eq!(
        surface.select_panel(&mut app, &unopened),
        Err(DockSurfacePanelError::PanelNotOpen {
            panel: unopened.clone()
        })
    );
    assert_eq!(
        surface.close_panel(&mut app, &unopened),
        Err(DockSurfacePanelError::PanelNotOpen { panel: unopened })
    );
}

#[test]
fn dock_surface_registered_panels_include_locations_and_descriptor_flags() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel_b.clone(), test_panel("B"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, panel_a.clone(), test_panel("A"))
        .expect("panel registers");
    surface
        .open_panel(&mut app, &panel_b)
        .expect("open panel b");

    let panels = surface.registered_panels(&app);
    assert_eq!(
        panels.iter().map(|panel| &panel.key).collect::<Vec<_>>(),
        vec![&panel_a, &panel_b],
        "registered panel snapshots should be stable-sorted by key"
    );
    assert_eq!(panels[0].title, "A");
    assert!(!panels[0].descriptor_only);
    assert_eq!(panels[0].location, None);
    assert!(
        panels[1]
            .location
            .as_ref()
            .is_some_and(|location| location.window == window && location.active)
    );
}

#[test]
fn dock_surface_snapshot_exports_layout_and_panel_facts() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface.open_panel(&mut app, &panel).expect("open panel");

    let snapshot = surface
        .snapshot_with_placement(&app, &[(window, "main".to_string())], |candidate| {
            (candidate == window).then_some(DockWindowPlacement {
                width: 1200,
                height: 800,
                x: Some(10),
                y: Some(20),
                monitor_hint: Some("primary".to_string()),
            })
        })
        .expect("snapshot");

    assert_eq!(snapshot.layout.windows.len(), 1);
    assert_eq!(snapshot.layout.windows[0].logical_window_id, "main");
    assert!(snapshot.layout.windows[0].placement.is_some());
    assert_eq!(snapshot.panels.len(), 1);
    assert_eq!(snapshot.panels[0].key, panel);
    assert!(snapshot.panels[0].location.is_some());
}

#[test]
fn dock_surface_snapshot_reports_descriptor_only_imported_panels() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let known = PanelKey::new("test.known");
    let restored = PanelKey::new("test.restored");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, known.clone(), test_panel("Known"))
        .expect("panel registers");

    let layout = DockLayout::new(
        vec![DockLayoutWindow {
            logical_window_id: "main".to_string(),
            root: 1,
            placement: None,
            floatings: Vec::new(),
        }],
        vec![DockLayoutNode::Tabs {
            id: 1,
            tabs: vec![known.clone(), restored.clone()],
            active: 1,
        }],
    );

    assert!(
        surface
            .try_import_layout_for_windows(&mut app, &layout, &[(window, "main".to_string())])
            .expect("layout imports")
    );

    let snapshot = surface
        .snapshot(&app, &[(window, "main".to_string())])
        .expect("snapshot");
    let restored_snapshot = snapshot
        .panels
        .iter()
        .find(|snapshot| snapshot.key == restored)
        .expect("restored descriptor-only panel");
    assert_eq!(restored_snapshot.title, restored.kind.0);
    assert!(restored_snapshot.descriptor_only);
    assert_eq!(
        restored_snapshot.location,
        Some(DockSurfacePanelLocation {
            window,
            placement: DockSurfacePanelPlacement::Docked,
            tab_index: 1,
            tab_count: 2,
            active: true,
        })
    );
}

#[test]
fn dock_surface_import_layout_invalidates_imported_windows() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.imported");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(DockManager::default());

    let layout = DockLayout::new(
        vec![DockLayoutWindow {
            logical_window_id: "main".to_string(),
            root: 1,
            placement: None,
            floatings: Vec::new(),
        }],
        vec![DockLayoutNode::Tabs {
            id: 1,
            tabs: vec![panel],
            active: 0,
        }],
    );

    assert!(
        surface
            .try_import_layout_for_windows(&mut app, &layout, &[(window, "main".to_string())])
            .expect("layout imports")
    );

    assert!(
        !app.take_changed_models().is_empty(),
        "importing a layout should bump dock host invalidation models"
    );
    assert!(
        app.take_redraws().contains(&window),
        "importing a layout should request redraw for imported windows"
    );
}

#[test]
fn dock_surface_viewport_session_queues_create_with_typed_outcome() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface.open_panel(&mut app, &panel).expect("open panel");
    app.take_effects();

    let outcome = surface
        .viewports()
        .open_panel(&mut app, &panel, None)
        .expect("open viewport");

    assert_eq!(outcome.panel, panel);
    assert_eq!(
        outcome.status,
        DockSurfaceViewportOpenStatus::WindowCreateQueued
    );
    assert_eq!(outcome.change, DockSurfaceChange::Unchanged);
    assert_eq!(outcome.window_requests, 1);
    assert_eq!(
        app.take_effects()
            .iter()
            .filter(|effect| matches!(effect, Effect::Window(WindowRequest::Create(_))))
            .count(),
        1
    );
}

#[test]
fn dock_surface_viewport_session_uses_panel_owner_as_source_window() {
    let main_window = AppWindowId::from(KeyData::from_ffi(1));
    let other_window = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(main_window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface
        .open_panel_with_preferred_window(&mut app, other_window, &panel)
        .expect("open panel in other window");
    app.take_effects();

    let outcome = surface
        .viewports()
        .open_panel(&mut app, &panel, None)
        .expect("open viewport from owner window");

    assert_eq!(outcome.source_window, other_window);
    assert_eq!(outcome.window_requests, 1);
    let effects = app.take_effects();
    let create = effects
        .iter()
        .find_map(|effect| match effect {
            Effect::Window(WindowRequest::Create(create)) => Some(create),
            _ => None,
        })
        .expect("expected create-window effect");
    assert!(matches!(
        create.kind,
        CreateWindowKind::DockFloating {
            source_window,
            panel: ref requested,
        } if source_window == other_window && requested == &panel
    ));
}

#[test]
fn dock_surface_viewport_session_reports_already_pending() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface.open_panel(&mut app, &panel).expect("open panel");
    app.take_effects();

    let first = surface
        .viewports()
        .open_panel(&mut app, &panel, None)
        .expect("open viewport");
    assert_eq!(
        first.status,
        DockSurfaceViewportOpenStatus::WindowCreateQueued
    );
    assert_eq!(first.window_requests, 1);
    app.take_effects();

    let second = surface
        .viewports()
        .open_panel(&mut app, &panel, None)
        .expect("open viewport while pending");

    assert_eq!(second.status, DockSurfaceViewportOpenStatus::AlreadyPending);
    assert_eq!(second.change, DockSurfaceChange::Unchanged);
    assert_eq!(second.window_requests, 0);
    assert!(
        app.take_effects()
            .iter()
            .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Create(_))))
    );
}

#[test]
fn dock_surface_viewport_session_reports_in_window_fallback() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window);

    let mut caps = PlatformCapabilities::default();
    caps.ui.multi_window = false;

    let mut app = TestHost::new();
    app.set_global(caps);
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface.open_panel(&mut app, &panel).expect("open panel");
    app.take_effects();

    let outcome = surface
        .viewports()
        .open_panel(&mut app, &panel, None)
        .expect("open viewport fallback");

    assert_eq!(
        outcome.status,
        DockSurfaceViewportOpenStatus::InWindowFallback
    );
    assert_eq!(outcome.change, DockSurfaceChange::Changed);
    assert_eq!(outcome.window_requests, 0);
    assert_eq!(
        surface
            .panel_location(&app, &panel)
            .map(|location| location.placement),
        Some(DockSurfacePanelPlacement::Floating)
    );
    assert!(
        app.take_effects()
            .iter()
            .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Create(_))))
    );
}

#[test]
fn dock_surface_viewport_session_reports_panel_not_open() {
    let main_window = AppWindowId::from(KeyData::from_ffi(1));
    let other_window = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(main_window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface.open_panel(&mut app, &panel).expect("open panel");

    assert_eq!(
        surface
            .viewports()
            .open_panel_from_window(&mut app, other_window, &panel, None),
        Err(DockSurfaceViewportError::PanelNotOpen {
            source_window: other_window,
            panel,
        })
    );
}

#[test]
fn dock_surface_viewport_session_reports_no_manager_without_effects() {
    let main_window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(main_window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());

    assert_eq!(
        surface.viewports().open_panel(&mut app, &panel, None),
        Err(DockSurfaceViewportError::DockManagerUnavailable)
    );
    assert_eq!(
        surface
            .viewports()
            .before_close_window(&mut app, main_window),
        Err(DockSurfaceViewportError::DockManagerUnavailable)
    );
    assert!(
        app.take_effects().is_empty(),
        "viewport errors must not emit host effects"
    );
}

#[test]
fn dock_surface_viewport_session_before_close_merges_panels() {
    let main_window = AppWindowId::from(KeyData::from_ffi(1));
    let closing_window = AppWindowId::from(KeyData::from_ffi(2));
    let main_panel = PanelKey::new("test.main");
    let floating_panel = PanelKey::new("test.floating");
    let surface = DockSurface::new(main_window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, main_panel.clone(), test_panel("Main"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, floating_panel.clone(), test_panel("Floating"))
        .expect("panel registers");
    surface
        .open_panel_with_preferred_window(&mut app, main_window, &main_panel)
        .expect("open main panel");
    surface
        .open_panel_with_preferred_window(&mut app, closing_window, &floating_panel)
        .expect("open floating panel");

    let outcome = surface
        .viewports()
        .before_close_window(&mut app, closing_window)
        .expect("before close");

    assert_eq!(outcome.window, closing_window);
    assert_eq!(outcome.change, DockSurfaceChange::Changed);
    assert_eq!(outcome.window_requests, 0);
    assert_eq!(
        surface
            .panel_location(&app, &floating_panel)
            .map(|location| location.window),
        Some(main_window)
    );
}

#[test]
fn dock_surface_viewport_before_close_can_merge_into_explicit_target() {
    let main_window = AppWindowId::from(KeyData::from_ffi(1));
    let closing_window = AppWindowId::from(KeyData::from_ffi(2));
    let target_window = AppWindowId::from(KeyData::from_ffi(3));
    let target_panel = PanelKey::new("test.target");
    let closing_panel = PanelKey::new("test.closing");
    let surface = DockSurface::new(main_window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, target_panel.clone(), test_panel("Target"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, closing_panel.clone(), test_panel("Closing"))
        .expect("panel registers");
    surface
        .open_panel_with_preferred_window(&mut app, target_window, &target_panel)
        .expect("open target panel");
    surface
        .open_panel_with_preferred_window(&mut app, closing_window, &closing_panel)
        .expect("open closing panel");

    let outcome = surface
        .viewports()
        .before_close_window_into(&mut app, closing_window, target_window)
        .expect("before close into explicit target");

    assert_eq!(outcome.window, closing_window);
    assert_eq!(outcome.change, DockSurfaceChange::Changed);
    assert_eq!(
        surface
            .panel_location(&app, &closing_panel)
            .map(|location| location.window),
        Some(target_window)
    );
    assert!(
        surface
            .panel_location(&app, &closing_panel)
            .is_none_or(|location| { location.window != main_window }),
        "explicit target close must not merge into surface.main_window"
    );
}

#[test]
fn dock_surface_viewport_session_before_close_reports_noop_for_main_or_empty_window() {
    let main_window = AppWindowId::from(KeyData::from_ffi(1));
    let empty_window = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(main_window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface.open_panel(&mut app, &panel).expect("open panel");

    let main_outcome = surface
        .viewports()
        .before_close_window(&mut app, main_window)
        .expect("before close main window");
    assert_eq!(main_outcome.change, DockSurfaceChange::Unchanged);
    assert_eq!(main_outcome.window_requests, 0);

    let empty_outcome = surface
        .viewports()
        .before_close_window(&mut app, empty_window)
        .expect("before close empty window");
    assert_eq!(empty_outcome.change, DockSurfaceChange::Unchanged);
    assert_eq!(empty_outcome.window_requests, 0);
}

#[test]
fn dock_surface_open_panel_invalidates_layout_and_cancels_pending_tearoff() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window_a);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface.open_panel(&mut app, &panel).expect("open panel");
    let changed_after_open = app.take_changed_models();
    assert!(
        !changed_after_open.is_empty(),
        "EnsurePanelVisible should bump docking invalidation models"
    );

    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel.clone(),
            None
        )
    );
    let commands = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
        panic!("expected create-window docking runtime command");
    };

    assert_eq!(
        surface
            .open_panel_with_preferred_window(&mut app, window_a, &panel)
            .expect("reopen existing panel")
            .change,
        DockSurfaceChange::Unchanged
    );
    assert!(DockSurfaceDriver::new(surface).on_window_created(&mut app, &create, window_b));
    assert_eq!(
        DockSurfaceDriver::new(surface).take_runtime_commands(&mut app),
        vec![DockRuntimeCommand::CloseWindow(window_b)],
        "EnsurePanelVisible should cancel an outstanding tear-off for the same panel"
    );
    assert_eq!(
        surface
            .panel_location(&app, &panel)
            .map(|location| location.window),
        Some(window_a)
    );
}

#[test]
fn dock_surface_semantic_open_does_not_flush_existing_runtime_commands() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let queued_panel = PanelKey::new("test.queued");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, queued_panel.clone(), test_panel("Queued"))
        .expect("panel registers");
    surface.open_panel(&mut app, &panel).expect("open panel");
    surface
        .open_panel(&mut app, &queued_panel)
        .expect("open queued panel");
    surface
        .open_panel(&mut app, &panel)
        .expect("make panel active again");
    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window,
            queued_panel.clone(),
            None
        )
    );
    app.take_effects();

    let outcome = surface
        .open_panel(&mut app, &panel)
        .expect("open already-visible panel");

    assert_eq!(outcome.change, DockSurfaceChange::Unchanged);
    assert!(
        app.take_effects()
            .iter()
            .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Create(_)))),
        "semantic panel commands must not flush pre-existing docking runtime commands"
    );
    assert!(
        DockSurfaceDriver::new(surface)
            .take_runtime_commands(&mut app)
            .iter()
            .any(|command| queued_create_panel(command) == Some((window, queued_panel.clone()))),
        "pre-existing unrelated docking runtime command should remain queued"
    );
}

#[test]
fn dock_surface_semantic_open_removes_canceled_create_for_same_panel_only() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let other_panel = PanelKey::new("test.other");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, other_panel.clone(), test_panel("Other"))
        .expect("panel registers");
    surface.open_panel(&mut app, &panel).expect("open panel");
    surface
        .open_panel(&mut app, &other_panel)
        .expect("open other panel");
    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window,
            panel.clone(),
            None
        )
    );
    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window,
            other_panel.clone(),
            None
        )
    );
    app.take_effects();

    surface.open_panel(&mut app, &panel).expect("reopen panel");

    assert!(
        app.take_effects()
            .iter()
            .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Create(_)))),
        "semantic panel commands must not emit a canceled create-window request"
    );
    let queued = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    assert_eq!(queued.len(), 1);
    assert_eq!(
        queued_create_panel(&queued[0]),
        Some((window, other_panel)),
        "only the unrelated pending create should remain queued"
    );
}

#[test]
fn dock_surface_semantic_open_removes_canceled_create_for_matching_source_window_only() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let surface = DockSurface::new(window_a);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel_a.clone(), test_panel("A"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, panel_b.clone(), test_panel("B"))
        .expect("panel registers");
    surface
        .open_panel_with_preferred_window(&mut app, window_a, &panel_a)
        .expect("open panel a");
    surface
        .open_panel_with_preferred_window(&mut app, window_b, &panel_b)
        .expect("open panel b");
    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel_a.clone(),
            None
        )
    );
    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_b,
            panel_b.clone(),
            None
        )
    );
    app.take_effects();

    surface
        .open_panel_with_preferred_window(&mut app, window_a, &panel_a)
        .expect("reopen panel a");

    assert!(
        app.take_effects()
            .iter()
            .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Create(_)))),
        "semantic panel commands must not emit a canceled cross-window create request"
    );
    let queued = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    assert_eq!(queued.len(), 1);
    assert_eq!(
        queued_create_panel(&queued[0]),
        Some((window_b, panel_b)),
        "canceling a pending create for one source window must keep unrelated source-window requests"
    );
}

#[test]
fn dock_surface_canceled_window_created_keeps_unrelated_pending_completion_alive() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let new_window_a = AppWindowId::from(KeyData::from_ffi(3));
    let new_window_b = AppWindowId::from(KeyData::from_ffi(4));
    let panel_a = PanelKey::new("test.a");
    let panel_b = PanelKey::new("test.b");
    let surface = DockSurface::new(window_a);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel_a.clone(), test_panel("A"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, panel_b.clone(), test_panel("B"))
        .expect("panel registers");
    surface
        .open_panel_with_preferred_window(&mut app, window_a, &panel_a)
        .expect("open panel a");
    surface
        .open_panel_with_preferred_window(&mut app, window_b, &panel_b)
        .expect("open panel b");

    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel_a.clone(),
            None
        )
    );
    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_b,
            panel_b.clone(),
            None
        )
    );
    let commands = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    assert_eq!(
        commands
            .iter()
            .filter(|command| matches!(command, DockRuntimeCommand::CreateWindow(_)))
            .count(),
        2
    );
    let create_a = queued_create_request(&commands, window_a, &panel_a);
    let create_b = queued_create_request(&commands, window_b, &panel_b);

    surface
        .close_panel(&mut app, &panel_a)
        .expect("close panel a");

    assert!(DockSurfaceDriver::new(surface).on_window_created(&mut app, &create_a, new_window_a,));
    assert_eq!(
        DockSurfaceDriver::new(surface).take_runtime_commands(&mut app),
        vec![DockRuntimeCommand::CloseWindow(new_window_a)],
        "the canceled create should close only its newly created OS window"
    );

    assert!(DockSurfaceDriver::new(surface).on_window_created(&mut app, &create_b, new_window_b,));
    assert!(
        DockSurfaceDriver::new(surface)
            .take_runtime_commands(&mut app)
            .is_empty(),
        "the unrelated pending create should still complete normally"
    );
    assert_eq!(
        surface
            .panel_location(&app, &panel_b)
            .map(|location| location.window),
        Some(new_window_b)
    );
    assert_eq!(surface.panel_location(&app, &panel_a), None);
}

#[test]
fn dock_surface_close_flushes_auto_close_after_canceled_create_removal() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window_a);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface.open_panel(&mut app, &panel).expect("open panel");
    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel.clone(),
            None
        )
    );
    let commands = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
        panic!("expected create-window docking runtime command");
    };
    assert!(DockSurfaceDriver::new(surface).on_window_created(&mut app, &create, window_b));
    assert!(
        DockSurfaceDriver::new(surface)
            .take_runtime_commands(&mut app)
            .is_empty()
    );
    assert_eq!(
        surface
            .panel_location(&app, &panel)
            .map(|location| location.window),
        Some(window_b)
    );

    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_b,
            panel.clone(),
            None
        )
    );
    app.take_effects();

    let outcome = surface.close_panel(&mut app, &panel).expect("close panel");

    assert_eq!(outcome.location, None);
    assert!(
        DockSurfaceDriver::new(surface)
            .take_runtime_commands(&mut app)
            .is_empty(),
        "canceled create and operation-local auto-close should both be drained"
    );
    let effects = app.take_effects();
    assert_eq!(effects.len(), 1);
    assert!(
        matches!(&effects[0], Effect::Window(WindowRequest::Close(window)) if *window == window_b),
        "closing the last panel in a dock floating window should flush its auto-close effect"
    );
}

#[test]
fn dock_surface_viewport_open_flushes_only_new_runtime_commands() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let old_panel = PanelKey::new("test.old");
    let new_panel = PanelKey::new("test.new");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, old_panel.clone(), test_panel("Old"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, new_panel.clone(), test_panel("New"))
        .expect("panel registers");
    surface.open_panel(&mut app, &old_panel).expect("open old");
    surface.open_panel(&mut app, &new_panel).expect("open new");
    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window,
            old_panel.clone(),
            None
        )
    );
    app.take_effects();

    let outcome = surface
        .viewports()
        .open_panel(&mut app, &new_panel, None)
        .expect("open viewport for new panel");

    assert_eq!(outcome.window_requests, 1);
    let effects = app.take_effects();
    assert_eq!(effects.len(), 1);
    let Effect::Window(WindowRequest::Create(create)) = &effects[0] else {
        panic!("expected only the newly queued create command to be flushed");
    };
    assert!(matches!(
        create.kind,
        CreateWindowKind::DockFloating {
            source_window,
            panel: ref requested,
        } if source_window == window && requested == &new_panel
    ));
    let queued = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    assert_eq!(queued.len(), 1);
    assert_eq!(queued_create_panel(&queued[0]), Some((window, old_panel)));
}

#[test]
fn dock_surface_viewport_before_close_does_not_flush_existing_runtime_commands() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface.open_panel(&mut app, &panel).expect("open panel");
    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window,
            panel.clone(),
            None
        )
    );
    app.take_effects();

    let outcome = surface
        .viewports()
        .before_close_window(&mut app, window)
        .expect("before close main window");

    assert_eq!(outcome.change, DockSurfaceChange::Unchanged);
    assert_eq!(outcome.window_requests, 0);
    assert!(
        app.take_effects()
            .iter()
            .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Create(_)))),
        "viewport close lifecycle must not flush pre-existing docking runtime commands"
    );
    assert!(
        DockSurfaceDriver::new(surface)
            .take_runtime_commands(&mut app)
            .iter()
            .any(|command| matches!(command, DockRuntimeCommand::CreateWindow(_))),
        "pre-existing docking runtime command should remain queued"
    );
}

#[test]
fn dock_surface_request_float_panel_uses_runtime_command_queue() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window_a);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window_a, tabs);
    });

    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel.clone(),
            None
        )
    );

    assert!(
        app.take_effects().is_empty(),
        "DockSurface command path should not emit Effect::Dock or WindowRequest::Create"
    );
    let commands = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    assert_eq!(commands.len(), 1);
    let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
        panic!("expected create-window docking runtime command");
    };
    assert!(matches!(
        create.kind,
        CreateWindowKind::DockFloating {
            source_window,
            panel: ref requested,
        }
            if source_window == window_a && requested == &panel
    ));

    assert!(DockSurfaceDriver::new(surface).on_window_created(&mut app, &create, window_b));
    let dock = app.global::<DockManager>().expect("dock manager exists");
    assert!(
        dock.workspace
            .graph
            .find_panel_in_window(window_b, &panel)
            .is_some(),
        "expected panel to move after completing the queued create command"
    );
    assert!(
        dock.workspace
            .graph
            .find_panel_in_window(window_a, &panel)
            .is_none(),
        "expected panel to leave the source window after queued create completion"
    );
}

#[test]
fn dock_surface_runtime_command_queue_deduplicates_until_window_created() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window,
            panel.clone(),
            None
        )
    );
    assert!(
        DockSurfaceDriver::new(surface)
            .request_float_panel_to_new_window(&mut app, window, panel, None)
    );

    let commands = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    assert_eq!(
        commands
            .iter()
            .filter(|command| matches!(command, DockRuntimeCommand::CreateWindow(_)))
            .count(),
        1,
        "DockSurface command queue should preserve runtime request idempotency"
    );
    assert!(
        app.take_effects().iter().all(|effect| !matches!(
            effect,
            Effect::Dock(_) | Effect::Window(WindowRequest::Create(_))
        )),
        "DockSurface command queue should not mirror create requests through Effect::Dock"
    );
}

#[test]
fn dock_surface_request_float_panel_does_not_emit_host_effects_before_flush() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window,
            panel.clone(),
            None
        )
    );

    assert!(
        app.take_effects().iter().all(|effect| !matches!(
            effect,
            Effect::Dock(_) | Effect::Window(WindowRequest::Create(_))
        )),
        "DockSurface request helpers should route float requests through docking runtime commands"
    );
    assert_eq!(
        DockSurfaceDriver::new(surface)
            .take_runtime_commands(&mut app)
            .iter()
            .filter(|command| matches!(command, DockRuntimeCommand::CreateWindow(_)))
            .count(),
        1
    );
}

#[test]
fn dock_surface_flushes_runtime_commands_to_host_effects() {
    let window = AppWindowId::from(KeyData::from_ffi(1));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window, tabs);
    });

    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window,
            panel.clone(),
            None
        )
    );

    assert_eq!(
        DockSurfaceDriver::new(surface).flush_runtime_commands_to_effects(&mut app),
        1
    );
    assert!(
        DockSurfaceDriver::new(surface)
            .take_runtime_commands(&mut app)
            .is_empty(),
        "flushed commands should be drained from the docking runtime queue"
    );
    let effects = app.take_effects();
    assert_eq!(
        effects
            .iter()
            .filter(|effect| matches!(effect, Effect::Window(WindowRequest::Create(_))))
            .count(),
        1
    );
    let Effect::Window(WindowRequest::Create(create)) = &effects[0] else {
        panic!("expected flushed docking runtime command to become WindowRequest::Create");
    };
    assert!(matches!(
        create.kind,
        CreateWindowKind::DockFloating {
            source_window,
            panel: ref requested,
        }
            if source_window == window && requested == &panel
    ));
    assert_eq!(
        DockSurfaceDriver::new(surface).flush_runtime_commands_to_effects(&mut app),
        0,
        "flushing an empty runtime command queue should be a no-op"
    );
}

#[test]
fn dock_surface_flushes_close_runtime_commands_to_host_effects() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let window_c = AppWindowId::from(KeyData::from_ffi(3));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window_a);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window_a, tabs);
    });

    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel.clone(),
            None
        )
    );
    let commands = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
        panic!("expected create-window docking runtime command");
    };
    assert!(DockSurfaceDriver::new(surface).on_dock_op(
        &mut app,
        DockOp::MovePanelToEmptyDockSpace {
            source_window: window_a,
            panel: panel.clone(),
            target_window: window_c,
        },
    ));
    assert!(DockSurfaceDriver::new(surface).on_window_created(&mut app, &create, window_b));

    assert_eq!(
        DockSurfaceDriver::new(surface).flush_runtime_commands_to_effects(&mut app),
        1
    );
    assert!(
        DockSurfaceDriver::new(surface)
            .take_runtime_commands(&mut app)
            .is_empty(),
        "flushed close command should be drained from the docking runtime queue"
    );
    let effects = app.take_effects();
    assert_eq!(effects.len(), 1);
    assert!(
        matches!(&effects[0], Effect::Window(WindowRequest::Close(window)) if *window == window_b),
        "expected the only flushed docking runtime command to become WindowRequest::Close"
    );
}

#[test]
fn dock_surface_window_created_for_stale_source_queues_close_command() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let window_c = AppWindowId::from(KeyData::from_ffi(3));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window_a);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window_a, tabs);
    });

    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel.clone(),
            None
        )
    );
    let commands = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
        panic!("expected create-window docking runtime command");
    };
    assert!(DockSurfaceDriver::new(surface).on_dock_op(
        &mut app,
        DockOp::MovePanelToEmptyDockSpace {
            source_window: window_a,
            panel: panel.clone(),
            target_window: window_c,
        },
    ));

    assert!(DockSurfaceDriver::new(surface).on_window_created(&mut app, &create, window_b));
    assert_eq!(
        DockSurfaceDriver::new(surface).take_runtime_commands(&mut app),
        vec![DockRuntimeCommand::CloseWindow(window_b)]
    );
    assert!(
        app.take_effects()
            .iter()
            .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Close(_)))),
        "DockSurface stale create cleanup should stay on the docking command queue"
    );
    let dock = app.global::<DockManager>().expect("dock manager exists");
    assert!(
        dock.workspace
            .graph
            .find_panel_in_window(window_c, &panel)
            .is_some(),
        "stale queued window completion must preserve the current graph owner"
    );
}

#[test]
fn dock_surface_window_created_graph_commit_failure_queues_close_command() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");
    let surface = DockSurface::new(window_a);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![panel.clone()],
            active: 0,
        });
        dock.workspace.graph.set_window_root(window_a, tabs);
    });

    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel.clone(),
            None
        )
    );
    let commands = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
        panic!("expected create-window docking runtime command");
    };
    app.with_global_mut(DockManager::default, |dock, _app| {
        assert!(
            dock.workspace.graph.close_panel(window_a, panel.clone()),
            "test setup should remove the source panel without notifying the tear-off machine"
        );
    });

    assert!(DockSurfaceDriver::new(surface).on_window_created(&mut app, &create, window_b));
    assert_eq!(
        DockSurfaceDriver::new(surface).take_runtime_commands(&mut app),
        vec![DockRuntimeCommand::CloseWindow(window_b)]
    );
    assert!(
        app.take_effects()
            .iter()
            .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Close(_)))),
        "DockSurface graph commit failure cleanup should stay on the docking command queue"
    );
    let dock = app.global::<DockManager>().expect("dock manager exists");
    assert!(
        dock.workspace
            .graph
            .find_panel_in_window(window_b, &panel)
            .is_none(),
        "failed graph commit must not invent the panel in the new window"
    );
}

#[test]
fn dock_surface_redock_auto_close_uses_runtime_command_queue() {
    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let panel = PanelKey::new("test.panel");
    let placeholder = PanelKey::new("test.placeholder");
    let surface = DockSurface::new(window_a);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    app.set_global(DockManager::default());
    surface
        .register_panel(&mut app, panel.clone(), test_panel("Panel"))
        .expect("panel registers");
    surface
        .register_panel(&mut app, placeholder.clone(), test_panel("Placeholder"))
        .expect("panel registers");
    app.with_global_mut(DockManager::default, |dock, _app| {
        let tabs = dock.workspace.graph.insert_node(DockNode::Tabs {
            tabs: vec![placeholder.clone(), panel.clone()],
            active: 1,
        });
        dock.workspace.graph.set_window_root(window_a, tabs);
    });

    assert!(
        DockSurfaceDriver::new(surface).request_float_panel_to_new_window(
            &mut app,
            window_a,
            panel.clone(),
            None
        )
    );
    let commands = DockSurfaceDriver::new(surface).take_runtime_commands(&mut app);
    let DockRuntimeCommand::CreateWindow(create) = commands[0].clone() else {
        panic!("expected create-window docking runtime command");
    };
    assert!(DockSurfaceDriver::new(surface).on_window_created(&mut app, &create, window_b));
    assert!(
        DockSurfaceDriver::new(surface)
            .take_runtime_commands(&mut app)
            .is_empty(),
        "successful window creation should not queue a close command"
    );

    let target_tabs = app
        .global::<DockManager>()
        .expect("dock manager exists")
        .workspace
        .graph
        .first_tabs_in_window(window_a)
        .expect("source window should still have placeholder tabs");

    assert!(DockSurfaceDriver::new(surface).on_dock_op(
        &mut app,
        DockOp::MovePanel {
            source_window: window_b,
            panel: panel.clone(),
            target_window: window_a,
            target_tabs,
            zone: DropZone::Center,
            insert_index: None,
        },
    ));

    assert_eq!(
        DockSurfaceDriver::new(surface).take_runtime_commands(&mut app),
        vec![DockRuntimeCommand::CloseWindow(window_b)]
    );
    assert!(
        app.take_effects()
            .iter()
            .all(|effect| !matches!(effect, Effect::Window(WindowRequest::Close(_)))),
        "DockSurface auto-close should stay on the docking command queue"
    );
    let dock = app.global::<DockManager>().expect("dock manager exists");
    assert!(
        dock.workspace
            .graph
            .find_panel_in_window(window_a, &panel)
            .is_some(),
        "redocked panel should return to the target window"
    );
    assert!(
        dock.workspace
            .graph
            .collect_panels_in_window(window_b)
            .is_empty(),
        "redocking the last panel should empty the dock-floating window"
    );
}
