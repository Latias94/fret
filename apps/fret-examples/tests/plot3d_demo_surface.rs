fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn plot3d_demo_uses_app_facing_plot3d_binding() {
    let source = compact(include_str!("../src/plot3d_demo.rs"));

    for needle in [
        "usefret_plot3d::{Plot3dPanelBinding,Plot3dStyle,Plot3dViewport,plot3d_panel};",
        "plot:Plot3dPanelBinding,",
        "target:ViewportRenderTarget,",
        "Plot3dPanelBinding::new(app,Plot3dViewport{",
        "target_px_size:(960,540)",
        "fit:fret_core::ViewportFit::Contain",
        "ViewportRenderTarget::new(wgpu::TextureFormat::Bgra8UnormSrgb,RenderTargetColorSpace::Srgb,)",
        "fnensure_target(",
        "state.plot.viewport_untracked(app).target_px_size",
        "state.target.ensure_size(",
        "state.plot.sync_viewport_target(app,id,new_size)",
        "fnrecord_engine_frame(",
        "EngineFrameUpdate{",
        "command_buffers:vec![encoder.finish()],",
        ".render_root(\"plot3d-demo\",|cx|{",
        "letstyle=Plot3dStyle::default();",
        "state.plot.panel_props().style(style)",
    ] {
        assert!(
            source.contains(needle),
            "plot3d_demo should keep app code on the Plot3D binding surface; missing `{needle}`"
        );
    }

    for legacy in [
        "plot:fret_runtime::Model<Plot3dModel>",
        "usefret_plot3d::{Plot3dModel,Plot3dPanelProps,Plot3dStyle,Plot3dViewport,plot3d_panel};",
        "app.models_mut().insert(Plot3dModel{",
        "state.plot.read(app,",
        "state.plot.update(app,",
        "Plot3dPanelProps::new(state.plot.clone())",
        "Plot3dPanelProps::new(",
        "plot3d_panel_with_model(",
        "Plot3dPanelBinding::from_model(",
    ] {
        assert!(
            !source.contains(legacy),
            "plot3d_demo should not expose raw Plot3D model handles in app code; unexpected `{legacy}`"
        );
    }
}
