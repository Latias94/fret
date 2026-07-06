fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn docking_arbitration_demo_keeps_body_and_state_text_on_roles() {
    let source = include_str!("../src/docking_arbitration_demo.rs");
    let source = compact(source);

    for needle in [
        "fndocking_arbitration_readout_text<H:fret_ui::UiHost>(",
        "fndocking_arbitration_paragraph_text<H:fret_ui::UiHost>(",
        "fret_ui_kit::declarative::text::text_control_readout(cx,text)",
        "fret_ui_kit::declarative::text::text_paragraph(cx,text)",
        "docking_arbitration_paragraph_text(cx,\"Non-modaloverlay(Popover).\")",
        ".map(|v|docking_arbitration_readout_text(cx,v))",
        "docking_arbitration_readout_text(cx,ifpopover_is_open{\"Popover:open\"}else{\"Popover:closed\"},)",
        "docking_arbitration_readout_text(cx,ifdialog_is_open{\"Dialog:open\"}else{\"Dialog:closed\"},)",
        "docking_arbitration_readout_text(cx,ifdrop_mask_left_disallowed{\"Dropmask:leftedgedockingdisallowed\"}else{\"Dropmask:leftedgedockingallowed\"},)",
    ] {
        assert!(
            source.contains(needle),
            "docking arbitration demo should keep fixed state readouts on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "vec![cx.text(ifpopover_is_open",
        "vec![cx.text(ifdialog_is_open",
        "vec![cx.text(ifdrop_mask_left_disallowed",
        "cx.text(\"Non-modaloverlay(Popover).\")",
        ".map(|v|cx.text(v))",
    ] {
        assert!(
            !source.contains(needle),
            "docking arbitration demo should not render state readouts with bare wrapping text; unexpected `{needle}`"
        );
    }
}

#[test]
fn docking_arbitration_demo_model_writes_stay_behind_owner_helper() {
    let source = include_str!("../src/docking_arbitration_demo.rs");
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("docking arbitration demo should have production source before tests");
    let compact_source = compact(source);
    let compact_production = compact(production_source);

    for needle in [
        "usefret_runtime::{ModelStore,PlatformCapabilities};",
        "structDockingArbitrationModelOwner<'a>{",
        "models:&'amutModelStore,",
        "fntoggle_drop_mask_disallow_left_edge(&mutself,model:&Model<bool>)->bool{",
        "fnset_last_viewport_input(&mutself,model:&Model<Arc<str>>,input:implInto<Arc<str>>,)->bool{",
        "fnset_synth_pointer_debug(&mutself,model:&Model<Arc<str>>,debug:implInto<Arc<str>>,)->bool{",
        "DockingArbitrationModelOwner::new(host.models_mut()).toggle_drop_mask_disallow_left_edge(&drop_mask_disallow_left_edge);",
        "DockingArbitrationModelOwner::new(app.models_mut()).set_synth_pointer_debug(&models.synth_pointer_debug,msg);",
        "DockingArbitrationModelOwner::new(app.models_mut()).set_last_viewport_input(&model,msg.clone());",
    ] {
        assert!(
            compact_source.contains(needle),
            "docking arbitration demo should route diagnostic model writes through its owner helper; missing `{needle}`"
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
            "docking arbitration production code should not bypass the owner helper with `{forbidden}`"
        );
    }
}
