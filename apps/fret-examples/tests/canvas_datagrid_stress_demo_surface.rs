fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn canvas_datagrid_stress_demo_keeps_header_text_on_readout_role() {
    let source = include_str!("../src/canvas_datagrid_stress_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fncanvas_datagrid_stress_readout_text<H:fret_ui::UiHost>(",
        "decl_text::text_control_readout(cx,text)",
        "canvas_datagrid_stress_readout_text(cx,header)",
    ] {
        assert!(
            source.contains(needle),
            "canvas datagrid stress demo should keep compact header text on the shared readout role; missing `{needle}`"
        );
    }

    for needle in ["cx.text(header)"] {
        assert!(
            !source.contains(needle),
            "canvas datagrid stress demo should not render the fixed header readout with bare wrapping text; unexpected `{needle}`"
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
