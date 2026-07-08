use fret_core::{AppWindowId, Color, PanelKey};
use fret_docking::{
    DockHostOptions, DockPanel, DockSurface, DockSurfaceChange, DockSurfaceHostSession,
    DockSurfacePanelError, DockSurfacePanelLocation, DockSurfacePanelOutcome,
    DockSurfacePanelPlacement, DockSurfacePanelSnapshot, DockSurfaceSnapshot,
    DockSurfaceViewportCloseOutcome, DockSurfaceViewportError, DockSurfaceViewportOpenOutcome,
    DockSurfaceViewportOpenStatus, DockSurfaceViewportSession,
};
use slotmap::KeyData;

#[test]
fn public_root_consumer_constructs_dock_surface_without_internal_runtime_helpers() {
    let main_window = AppWindowId::from(KeyData::from_ffi(1));
    let surface = DockSurface::new(main_window);

    assert_eq!(surface.main_window(), main_window);

    let options = DockHostOptions::default();
    assert!(!options.allow_multi_window_tear_off);

    let panel = DockPanel {
        title: "Inspector".to_string(),
        color: Color::TRANSPARENT,
        viewport: None,
    };
    assert_eq!(panel.title, "Inspector");

    let panel_key = PanelKey::new("external.inspector");
    assert_eq!(panel_key.kind.0.as_str(), "external.inspector");

    assert!(!DockSurfaceChange::Unchanged.changed());
    assert!(DockSurfaceChange::Changed.changed());

    let location = DockSurfacePanelLocation {
        window: main_window,
        placement: DockSurfacePanelPlacement::Docked,
        tab_index: 0,
        tab_count: 1,
        active: true,
    };
    let snapshot = DockSurfacePanelSnapshot {
        key: panel_key.clone(),
        title: "Inspector".to_string(),
        descriptor_only: false,
        location: Some(location.clone()),
    };
    assert_eq!(snapshot.location, Some(location));

    let panel_error = DockSurfacePanelError::PanelNotOpen {
        panel: panel_key.clone(),
    };
    assert!(matches!(
        panel_error,
        DockSurfacePanelError::PanelNotOpen { .. }
    ));

    let close_outcome = DockSurfaceViewportCloseOutcome {
        window: main_window,
        change: DockSurfaceChange::Unchanged,
        window_requests: 0,
    };
    assert_eq!(close_outcome.window, main_window);
    assert_eq!(
        DockSurfaceViewportOpenStatus::AlreadyPending,
        DockSurfaceViewportOpenStatus::AlreadyPending
    );
    assert_eq!(
        DockSurfaceViewportError::DockManagerUnavailable,
        DockSurfaceViewportError::DockManagerUnavailable
    );

    fn accepts_panel_outcome_type(_: Option<DockSurfacePanelOutcome>) {}
    accepts_panel_outcome_type(None);

    fn accepts_surface_snapshot_type(_: Option<DockSurfaceSnapshot>) {}
    accepts_surface_snapshot_type(None);

    fn accepts_viewport_session_type(_: Option<DockSurfaceViewportSession>) {}
    accepts_viewport_session_type(Some(surface.viewports()));

    fn accepts_host_session_type(_: Option<DockSurfaceHostSession>) {}
    accepts_host_session_type(Some(surface.host_lifecycle()));

    fn accepts_viewport_open_outcome_type(_: Option<DockSurfaceViewportOpenOutcome>) {}
    accepts_viewport_open_outcome_type(None);
}
