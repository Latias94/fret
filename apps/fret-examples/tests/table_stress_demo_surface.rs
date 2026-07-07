fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn table_stress_demo_keeps_fixed_table_text_on_roles() {
    let source = include_str!("../src/table_stress_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::text;",
        "text::control_readout(cx,header)",
        "vec![text::table_cell(cx,label)]",
        "vec![text::table_cell(cx,text)]",
    ] {
        assert!(
            source.contains(needle),
            "table stress demo should keep fixed readout/header/cell text on app text facade roles; missing `{needle}`"
        );
    }

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "text_control_readout(",
        "text_table_cell(",
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
fn table_stress_demo_model_state_stays_behind_controls_binding() {
    let source = include_str!("../src/table_stress_demo.rs");
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("table stress demo should have production source before tests");
    let compact_source = compact(source);
    let compact_production = compact(production_source);

    for needle in [
        "usefret_runtime::{ModelStore,PlatformCapabilities};",
        "structTableStressControls{",
        "table_state:Model<TableState>,",
        "items_revision:Model<u64>,",
        "controls:TableStressControls,",
        "fnnew(models:&mutModelStore,row_count:usize)->Self{",
        "fnrender_snapshot(&self,cx:&mutElementContext<'_,App>)->TableStressSnapshot{",
        "lettable_state=state.controls.table_model();",
        "letcontrols=state.controls.render_snapshot(cx);",
        "structTableStressModelOwner<'a>{",
        "models:&'amutModelStore,",
        "fnupdate_table_state(&mutself,state:&Model<TableState>,f:implFnOnce(&mutTableState),)->bool{",
        "fntoggle_sorting(&mutself,state:&Model<TableState>)->bool{",
        "fntoggle_role_filter(&mutself,state:&Model<TableState>)->bool{",
        "fntoggle_global_filter(&mutself,state:&Model<TableState>)->bool{",
        "fnclear_filters(&mutself,state:&Model<TableState>)->bool{",
        "fnbump_items_revision(&mutself,revision:&Model<u64>)->bool{",
        "TableStressModelOwner::new(app.models_mut()).toggle_sorting(&self.table_state)",
        "TableStressModelOwner::new(app.models_mut()).toggle_role_filter(&self.table_state)",
        "TableStressModelOwner::new(app.models_mut()).toggle_global_filter(&self.table_state)",
        "TableStressModelOwner::new(app.models_mut()).clear_filters(&self.table_state)",
        "TableStressModelOwner::new(app.models_mut()).bump_items_revision(&self.items_revision)",
        "state.controls.toggle_sorting(app);",
        "state.controls.toggle_role_filter(app);",
        "state.controls.toggle_global_filter(app);",
        "state.controls.clear_filters(app);",
        "state.controls.bump_items_revision(app);",
    ] {
        assert!(
            compact_source.contains(needle),
            "table stress demo should route table model state through its controls binding; missing `{needle}`"
        );
    }

    for forbidden in [
        "table_state:Model<TableState>,rows:",
        "items_revision:Model<u64>,scroll:",
        "&state.table_state",
        "&state.items_revision",
        "TableStressDriver::toggle_sorting",
        "TableStressDriver::toggle_role_filter",
        "TableStressDriver::toggle_global_filter",
        "TableStressDriver::clear_filters",
        "TableStressDriver::bump_items_revision",
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
