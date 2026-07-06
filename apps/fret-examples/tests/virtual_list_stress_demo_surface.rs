fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn virtual_list_stress_demo_keeps_fixed_row_text_on_roles() {
    let source = include_str!("../src/virtual_list_stress_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnvirtual_list_stress_readout_text<H:fret_ui::UiHost>(",
        "fnvirtual_list_stress_row_label_text<H:fret_ui::UiHost>(",
        "decl_text::text_control_readout(cx,text)",
        "decl_text::text_list_row_label(cx,text)",
        "virtual_list_stress_readout_text(cx,header)",
        "letlabel=Arc::<str>::from(format!(\"Row{id}(tall={tall_rows_enabled})\"));",
        "vec![virtual_list_stress_row_label_text(cx,label)]",
    ] {
        assert!(
            source.contains(needle),
            "virtual-list stress demo should keep fixed header/row text on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "cx.text(header)",
        "vec![cx.text(Arc::<str>::from(format!(\"Row{id}(tall={tall_rows_enabled})\")))]",
    ] {
        assert!(
            !source.contains(needle),
            "virtual-list stress demo should not render fixed row text with bare wrapping text; unexpected `{needle}`"
        );
    }
}

#[test]
fn virtual_list_stress_demo_model_writes_stay_behind_owner_helpers() {
    let source = include_str!("../src/virtual_list_stress_demo.rs");
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("virtual-list stress demo should have production source before tests");
    let compact_source = compact(source);
    let compact_production = compact(production_source);

    for needle in [
        "usefret_runtime::{ModelStore,PlatformCapabilities};",
        "structVirtualListStressModelOwner<'a>{",
        "models:&'amutModelStore,",
        "fntoggle_rows_enabled(&mutself,model:&Model<bool>)->bool{",
        "fntoggle_reversed(&mutself,model:&Model<bool>)->bool{",
        "fnbump_items_revision(&mutself,model:&Model<u64>)->bool{",
        "VirtualListStressModelOwner::new(app.models_mut()).toggle_rows_enabled(&state.tall_rows_enabled);",
        "VirtualListStressModelOwner::new(app.models_mut()).toggle_reversed(&state.reversed);",
        "VirtualListStressModelOwner::new(app.models_mut()).bump_items_revision(&state.items_revision);",
    ] {
        assert!(
            compact_source.contains(needle),
            "virtual-list stress demo should keep shared-model writes behind a named owner helper; missing `{needle}`"
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
        "fnvirtual_list_stress_update_model",
        "fnvirtual_list_stress_toggle_model",
        "fnvirtual_list_stress_bump_revision",
    ] {
        assert!(
            !compact_production.contains(forbidden),
            "virtual-list stress production code should not bypass the owner helper with `{forbidden}`"
        );
    }
}
