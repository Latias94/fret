fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn datatable_demo_keeps_fixed_table_text_on_roles() {
    let source = include_str!("../src/datatable_demo.rs");
    let source = compact(source);

    for needle in [
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
fn datatable_demo_keeps_recipe_state_output_and_debug_identity_explicit() {
    let source = include_str!("../src/datatable_demo.rs");
    let source = compact(source);

    for needle in [
        "table_state:LocalState<shadcn::TableState>",
        "table_output:LocalState<shadcn::DataTableViewOutput>",
        "table_recipe:shadcn::DataTableRecipe<DemoRow>",
        "letcolumns=datatable_columns();",
        "shadcn::DataTableRecipe::new(&table_state,&table_output,columns,|row,_i,_parent|{shadcn::RowKey(row.id)})",
        ".debug_ids(datatable_debug_ids())",
        ".toolbar_test_id_prefix(\"datatable-demo-toolbar\")",
        ".page_sizes(Arc::from([25usize,50,100,250]))",
        ".table(shadcn::DataTable::new().column_actions_menu(true))",
        "header_row_test_id:Some(Arc::<str>::from(\"datatable-demo-header-row\"))",
        "body_test_id:Some(Arc::<str>::from(\"datatable-demo-body\"))",
        "header_cell_test_id_prefix:Some(Arc::<str>::from(\"datatable-demo-header-\"))",
        "row_test_id_prefix:Some(Arc::<str>::from(\"datatable-demo-row-\"))",
        "row_cell_test_ids:true",
        "lettable_parts=table_recipe.into_elements(cx,rows,1,|cx,col,row|",
    ] {
        assert!(
            source.contains(needle),
            "datatable demo should keep recipe state/output/columns/row keys/debug ids app-visible; missing `{needle}`"
        );
    }
}
