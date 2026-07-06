fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn gizmo3d_demo_uses_app_facing_plot3d_binding() {
    let source = compact(include_str!("../src/gizmo3d_demo.rs"));

    for needle in [
        "usefret_plot3d::{Plot3dPanelBinding,Plot3dStyle,Plot3dViewport,plot3d_panel};",
        "plot:Plot3dPanelBinding,",
        "Plot3dPanelBinding::new(app,Plot3dViewport{",
        "state.plot.viewport_untracked(app).target_px_size",
        "state.plot.sync_viewport_target(app,id,size)",
        "state.plot.panel_props().style(style)",
    ] {
        assert!(
            source.contains(needle),
            "gizmo3d_demo should keep Plot3D panel state behind Plot3dPanelBinding; missing `{needle}`"
        );
    }

    for legacy in [
        "plot:fret_runtime::Model<Plot3dModel>",
        "usefret_plot3d::{Plot3dModel,Plot3dPanelProps,Plot3dStyle,Plot3dViewport,plot3d_panel};",
        "app.models_mut().insert(Plot3dModel{",
        "state.plot.read(app,",
        "state.plot.update(app,",
        "Plot3dPanelProps::new(state.plot.clone())",
    ] {
        assert!(
            !source.contains(legacy),
            "gizmo3d_demo should not expose raw Plot3D model handles in app code; unexpected `{legacy}`"
        );
    }
}

#[test]
fn gizmo3d_demo_hides_demo_model_handle_behind_binding() {
    let source = compact(include_str!("../src/gizmo3d_demo.rs"));

    for needle in [
        "structGizmo3dDemoModelBinding{",
        "model:fret_runtime::Model<Gizmo3dDemoModel>,",
        "per_window:HashMap<AppWindowId,Gizmo3dDemoModelBinding>,",
        "demo:Gizmo3dDemoModelBinding,",
        "letdemo=Gizmo3dDemoModelBinding::new(app);",
        "demo.apply_viewport_theme(app);",
        "svc.per_window.insert(window,demo.clone());",
        "state.demo.sync_viewport_target(app,id,size)",
    ] {
        assert!(
            source.contains(needle),
            "gizmo3d_demo should keep the shared demo model handle behind Gizmo3dDemoModelBinding; missing `{needle}`"
        );
    }

    for legacy in [
        "per_window:HashMap<AppWindowId,fret_runtime::Model<Gizmo3dDemoModel>>",
        "demo:fret_runtime::Model<Gizmo3dDemoModel>,",
        "letdemo=app.models_mut().insert(Gizmo3dDemoModel::default());",
        "let_=demo.update(app,|m,_cx|{apply_viewport_gizmo_theme(&theme,m);});",
        "let_=state.demo.update(app,|m,_cx|{m.viewport_target=id;m.viewport_px=size;});",
    ] {
        assert!(
            !source.contains(legacy),
            "gizmo3d_demo should not expose the raw Gizmo3dDemoModel handle outside the binding; unexpected `{legacy}`"
        );
    }
}
