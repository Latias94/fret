use fret_core::{AppWindowId, Color, PanelKey};
use fret_docking::advanced::DockRuntimeCommand;
use fret_docking::{
    DockHostOptions, DockPanel, DockSurface, DockSurfacePanelOutcome, DockSurfaceSnapshot,
    DockSurfaceViewportOpenOutcome, DockSurfaceViewportSession,
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

    fn accepts_runtime_command_type(_: Option<DockRuntimeCommand>) {}
    accepts_runtime_command_type(None);

    fn accepts_panel_outcome_type(_: Option<DockSurfacePanelOutcome>) {}
    accepts_panel_outcome_type(None);

    fn accepts_surface_snapshot_type(_: Option<DockSurfaceSnapshot>) {}
    accepts_surface_snapshot_type(None);

    fn accepts_viewport_session_type(_: Option<DockSurfaceViewportSession>) {}
    accepts_viewport_session_type(Some(surface.viewports()));

    fn accepts_viewport_open_outcome_type(_: Option<DockSurfaceViewportOpenOutcome>) {}
    accepts_viewport_open_outcome_type(None);
}
