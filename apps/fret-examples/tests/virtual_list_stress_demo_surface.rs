fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn virtual_list_stress_demo_keeps_fixed_row_text_on_roles() {
    let source = include_str!("../src/virtual_list_stress_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::{AppRenderContext,text};",
        "fnvirtual_list_stress_readout_text<'a,Cx>(",
        "fnvirtual_list_stress_row_label_text<'a,Cx>(",
        "Cx:AppRenderContext<'a>,",
        "text::control_readout(cx,text)",
        "text::list_row_label(cx,text)",
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
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "text_control_readout(",
        "text_list_row_label(",
    ] {
        assert!(
            !source.contains(needle),
            "virtual-list stress demo should not render fixed row text with bare wrapping text; unexpected `{needle}`"
        );
    }
}

#[test]
fn virtual_list_stress_demo_model_state_stays_behind_controls_binding() {
    let source = include_str!("../src/virtual_list_stress_demo.rs");
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("virtual-list stress demo should have production source before tests");
    let compact_source = compact(source);
    let compact_production = compact(production_source);

    for needle in [
        "usefret_runtime::{ModelStore,PlatformCapabilities};",
        "structVirtualListStressControls{",
        "tall_rows_enabled:Model<bool>,",
        "reversed:Model<bool>,",
        "items_revision:Model<u64>,",
        "fnnew(models:&mutModelStore)->Self{",
        "fntoggle_rows_enabled(&self,models:&mutModelStore)->bool{",
        "fntoggle_reversed_and_bump_revision(&self,models:&mutModelStore)->bool{",
        "fnlayout_snapshot(&self,cx:&mutElementContext<'_,App>)->VirtualListStressSnapshot{",
        "letcontrols=VirtualListStressControls::new(app.models_mut());",
        "controls:VirtualListStressControls,",
        "state.controls.toggle_rows_enabled(app.models_mut());",
        "state.controls.toggle_reversed_and_bump_revision(app.models_mut());",
        "letcontrols=state.controls.layout_snapshot(cx);",
    ] {
        assert!(
            compact_source.contains(needle),
            "virtual-list stress demo should keep shared-model state behind a named controls binding; missing `{needle}`"
        );
    }

    for forbidden in [
        "VirtualListStressModelOwner",
        "tall_rows_enabled:fret_app::Model<bool>",
        "reversed:fret_app::Model<bool>",
        "items_revision:fret_app::Model<u64>",
        "&state.tall_rows_enabled",
        "&state.reversed",
        "&state.items_revision",
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
