fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn table_stress_demo_keeps_fixed_table_text_on_roles() {
    let source = include_str!("../src/table_stress_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fntable_stress_readout_text<H:fret_ui::UiHost>(",
        "fntable_stress_cell_text<H:fret_ui::UiHost>(",
        "decl_text::text_control_readout(cx,text)",
        "decl_text::text_table_cell(cx,text)",
        "table_stress_readout_text(cx,header)",
        "vec![table_stress_cell_text(cx,label)]",
        "vec![table_stress_cell_text(cx,text)]",
    ] {
        assert!(
            source.contains(needle),
            "table stress demo should keep fixed readout/header/cell text on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "cx.text(header)",
        "vec![cx.text(label)]",
        "vec![cx.text(text)]",
    ] {
        assert!(
            !source.contains(needle),
            "table stress demo should not render fixed table text with bare wrapping text; unexpected `{needle}`"
        );
    }
}

#[test]
fn table_stress_demo_model_writes_stay_behind_owner_helper() {
    let source = include_str!("../src/table_stress_demo.rs");
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("table stress demo should have production source before tests");
    let compact_source = compact(source);
    let compact_production = compact(production_source);

    for needle in [
        "usefret_runtime::{ModelStore,PlatformCapabilities};",
        "structTableStressModelOwner<'a>{",
        "models:&'amutModelStore,",
        "fnupdate_table_state(&mutself,state:&Model<TableState>,f:implFnOnce(&mutTableState),)->bool{",
        "fntoggle_sorting(&mutself,state:&Model<TableState>)->bool{",
        "fntoggle_role_filter(&mutself,state:&Model<TableState>)->bool{",
        "fntoggle_global_filter(&mutself,state:&Model<TableState>)->bool{",
        "fnclear_filters(&mutself,state:&Model<TableState>)->bool{",
        "fnbump_items_revision(&mutself,revision:&Model<u64>)->bool{",
        "TableStressModelOwner::new(app.models_mut()).toggle_sorting(state);",
        "TableStressModelOwner::new(app.models_mut()).toggle_role_filter(state);",
        "TableStressModelOwner::new(app.models_mut()).toggle_global_filter(state);",
        "TableStressModelOwner::new(app.models_mut()).clear_filters(state);",
        "TableStressModelOwner::new(app.models_mut()).bump_items_revision(revision);",
        "TableStressDriver::bump_items_revision(app,&state.items_revision);",
    ] {
        assert!(
            compact_source.contains(needle),
            "table stress demo should route table model writes through its owner helper; missing `{needle}`"
        );
    }

    for forbidden in [
        "models_mut().update(",
        "models_mut().update::<",
        "models_mut().update_any(",
        "models_mut().update_any::<",
        "ModelStore::update(",
        "ModelStore::update::<",
        "ModelStore::update_any(",
        "ModelStore::update_any::<",
        "<ModelStore>::update(",
        "<ModelStore>::update::<",
        "<ModelStore>::update_any(",
        "<ModelStore>::update_any::<",
    ] {
        assert!(
            !compact_production.contains(forbidden),
            "table stress production code should not bypass the owner helper with `{forbidden}`"
        );
    }
}
