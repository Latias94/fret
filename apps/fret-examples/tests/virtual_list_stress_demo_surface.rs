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
    let compact_source = compact(source);

    for needle in [
        "fnvirtual_list_stress_update_model<T:Any>(",
        "fnvirtual_list_stress_toggle_model(app:&mutApp,model:&Model<bool>)",
        "fnvirtual_list_stress_bump_revision(app:&mutApp,model:&Model<u64>)",
        "virtual_list_stress_toggle_model(app,&state.tall_rows_enabled);",
        "virtual_list_stress_toggle_model(app,&state.reversed);",
        "virtual_list_stress_bump_revision(app,&state.items_revision);",
    ] {
        assert!(
            compact_source.contains(needle),
            "virtual-list stress demo should keep shared-model writes behind explicit owner helpers; missing `{needle}`"
        );
    }

    assert_eq!(
        source.matches("models_mut().update(").count(),
        1,
        "virtual-list stress demo should not scatter raw ModelStore updates outside the owner helper"
    );
}
