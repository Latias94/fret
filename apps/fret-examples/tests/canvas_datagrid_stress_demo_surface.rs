fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn canvas_datagrid_stress_demo_keeps_header_text_on_readout_role() {
    let source = include_str!("../src/canvas_datagrid_stress_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::text;",
        "usefret::app::AppRenderContext;",
        "fncanvas_datagrid_stress_readout_text<'a,Cx>(",
        "Cx:AppRenderContext<'a>,",
        "text::control_readout(cx,text)",
        "canvas_datagrid_stress_readout_text(cx,header)",
    ] {
        assert!(
            source.contains(needle),
            "canvas datagrid stress demo should keep compact header text on the app readout facade; missing `{needle}`"
        );
    }

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "text_control_readout(",
        "cx.text(header)",
    ] {
        assert!(
            !source.contains(needle),
            "canvas datagrid stress demo should not render the fixed header readout through raw text seams; unexpected `{needle}`"
        );
    }
}

#[test]
fn canvas_datagrid_stress_demo_uses_local_state_grid_output() {
    let source = include_str!("../src/canvas_datagrid_stress_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::{AppLocalStateExtas_,AppLocalStateTxnExtas_,LocalState};",
        "grid_output:LocalState<shadcn::DataGridCanvasOutput>,",
        "letgrid_output=app.local_state(shadcn::DataGridCanvasOutput::default());",
        "letgrid=app.local_state_txn(|tx|tx.value_or_default(&state.grid_output));",
        "letgrid=state.grid_output.layout_value(cx);",
        ".output_model(state.grid_output.clone())",
    ] {
        assert!(
            source.contains(needle),
            "canvas datagrid stress demo should keep grid output on the app-facing LocalState surface; missing `{needle}`"
        );
    }

    for needle in [
        "grid_output:Model<shadcn::DataGridCanvasOutput>",
        "app.models_mut().insert(shadcn::DataGridCanvasOutput::default())",
        ".read(&state.grid_output,|v|*v)",
    ] {
        assert!(
            !source.contains(needle),
            "canvas datagrid stress demo should not expose raw grid output model plumbing; unexpected `{needle}`"
        );
    }
}

#[test]
fn canvas_datagrid_stress_demo_bundles_stress_controls() {
    let source = include_str!("../src/canvas_datagrid_stress_demo.rs");
    let source = compact(source);

    for needle in [
        "structCanvasDataGridStressControls{",
        "variable_sizes:Model<bool>,",
        "clamp_rows:Model<bool>,",
        "revision:Model<u64>,",
        "fnnew(app:&mutApp)->Self{",
        "letcontrols=CanvasDataGridStressControls::new(app);",
        "controls:CanvasDataGridStressControls,",
        "letcontrols=state.controls.layout_snapshot(cx);",
        "letmutaxis=shadcn::DataGridCanvasAxis::new(Arc::clone(&rows),controls.revision,Px(24.0),)",
        "letmutaxis=shadcn::DataGridCanvasAxis::new(Arc::clone(&cols),controls.revision,Px(120.0),)",
    ] {
        assert!(
            source.contains(needle),
            "canvas datagrid stress demo should route retained stress control models through a controls bundle; missing `{needle}`"
        );
    }

    for forbidden in [
        "CanvasDataGridStressWindowState{ui:UiTree<App>,rows:Arc<Vec<u64>>,cols:Arc<Vec<u64>>,cell_texts:Arc<Vec<Arc<str>>>,variable_sizes:Model<bool>,",
        "clamp_rows:Model<bool>,revision:Model<u64>,grid_output:",
        "letvariable_sizes=app.models_mut().insert(",
        "letclamp_rows=app.models_mut().insert(",
        "letrevision=app.models_mut().insert(1u64);",
        "(&state.variable_sizes,&state.clamp_rows,&state.revision)",
    ] {
        assert!(
            !source.contains(forbidden),
            "canvas datagrid stress demo should not scatter stress control models outside CanvasDataGridStressControls; unexpected `{forbidden}`"
        );
    }
}
