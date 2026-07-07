fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn datatable_demo_keeps_fixed_table_text_on_roles() {
    let source = include_str!("../src/datatable_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::text;",
        "text::control_readout(cx,Arc::from(format!(\"DataTable|selected={selected}sort={sorting}\")),)",
        "\"id\"=>text::table_cell(cx,Arc::from(row.id.to_string()))",
        "\"name\"=>text::table_cell(cx,Arc::clone(&row.name))",
        "\"role\"=>text::table_cell(cx,Arc::clone(&row.role))",
        "\"score\"=>text::table_cell(cx,Arc::from(row.score.to_string()))",
        "_=>text::table_cell(cx,Arc::from(\"\"))",
    ] {
        assert!(
            source.contains(needle),
            "datatable demo should keep fixed readout/cell text on app text facade roles; missing `{needle}`"
        );
    }

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "text_control_readout(",
        "text_table_cell(",
        "cx.text(Arc::from(format!(\"DataTable|selected={selected}sort={sorting}\")))",
        "\"id\"=>cx.text(Arc::from(row.id.to_string()))",
        "\"name\"=>cx.text(Arc::clone(&row.name))",
        "\"role\"=>cx.text(Arc::clone(&row.role))",
        "\"score\"=>cx.text(Arc::from(row.score.to_string()))",
        "_=>cx.text(Arc::from(\"\"))",
    ] {
        assert!(
            !source.contains(needle),
            "datatable demo should not render fixed table text with bare wrapping text; unexpected `{needle}`"
        );
    }
}

#[test]
fn datatable_demo_uses_local_state_table_output() {
    let source = include_str!("../src/datatable_demo.rs");
    let source = compact(source);

    for needle in [
        "table_output:LocalState<shadcn::DataTableViewOutput>,",
        "lettable_output=app.local_state(shadcn::DataTableViewOutput::default());",
        "lettable_output=state.table_output.clone();",
        "let_=table_output.layout_value(cx);",
        "shadcn::DataTablePagination::new(&table_state,table_output.clone())",
        ".output_model(table_output.clone())",
    ] {
        assert!(
            source.contains(needle),
            "datatable demo should keep table output on the app-facing LocalState surface; missing `{needle}`"
        );
    }

    for needle in [
        "Model<shadcn::DataTableViewOutput>",
        "app.models_mut().insert(shadcn::DataTableViewOutput::default())",
        "cx.observe_model(&table_output,Invalidation::Layout);",
        "usefret_app::{App,CommandId,Effect,Model,WindowRequest};",
    ] {
        assert!(
            !source.contains(needle),
            "datatable demo should not expose raw table output model plumbing; unexpected `{needle}`"
        );
    }
}
