fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn components_gallery_table_torture_uses_text_roles() {
    let source = include_str!("../src/components_gallery.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "\"id\"=>decl_text::text_table_cell(cx,row.to_string())",
        "\"status\"=>decl_text::text_table_cell(cx,ifrow%3==0",
        "\"cpu\"=>decl_text::text_table_cell(cx,format!(\"{}%\",(row*7)%100))",
        "\"mem_mb\"=>decl_text::text_table_cell(cx,format!(\"{}MB\",128+(row%4096)))",
        "_=>decl_text::text_table_cell(cx,\"?\")",
        "letheader=decl_text::text_paragraph(cx,header);",
    ] {
        assert!(
            source.contains(needle),
            "components gallery table torture should keep retained table text on shared roles; missing `{needle}`"
        );
    }

    for needle in [
        "\"id\"=>cx.text(row.to_string())",
        "\"status\"=>cx.text(ifrow%3==0",
        "\"cpu\"=>cx.text(format!(\"{}%\",(row*7)%100))",
        "\"mem_mb\"=>cx.text(format!(\"{}MB\",128+(row%4096)))",
        "_=>cx.text(\"?\")",
        "letheader=cx.text(header);",
    ] {
        assert!(
            !source.contains(needle),
            "components gallery table torture should not use bare text for retained table cells/header prose; unexpected `{needle}`"
        );
    }
}

#[test]
fn components_gallery_chrome_and_controls_use_text_roles() {
    let source = include_str!("../src/components_gallery.rs");
    let source = compact(source);

    for needle in [
        "decl_text::text_chrome_title(cx,title)",
        "decl_text::text_control_readout(cx,subtitle)",
        "decl_text::text_control_label(cx,Arc::<str>::from(\"Theme:\"),)",
        "decl_text::text_control_readout(cx,Arc::<str>::from(format!(\"Themeconfig:{}\",theme_name)),)",
        "decl_text::text_control_label(cx,label,)",
        "decl_text::text_control_readout(cx,format!(\"checkbox:{checkbox_value}\"),)",
        "decl_text::text_control_readout(cx,format!(\"switch:{switch_value}\"),)",
        "decl_text::text_control_readout(cx,format!(\"radio:{radio_label}\"),)",
        "decl_text::text_control_readout(cx,format!(\"select:{select_label}\"),)",
    ] {
        assert!(
            source.contains(needle),
            "components gallery chrome/control text should stay on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "cx.text(title)",
        "cx.text(subtitle)",
        "cx.text(Arc::<str>::from(\"Theme:\"))",
        "cx.text(Arc::<str>::from(format!(\"Themeconfig:{}\",theme_name)))",
        "cx.text(label)",
        "cx.text(format!(\"checkbox:{checkbox_value}\"))",
        "cx.text(format!(\"switch:{switch_value}\"))",
        "cx.text(format!(\"radio:{radio_label}\"))",
        "cx.text(format!(\"select:{select_label}\"))",
    ] {
        assert!(
            !source.contains(needle),
            "components gallery fixed chrome/control text should not use bare text; unexpected `{needle}`"
        );
    }
}

#[test]
fn components_gallery_overlay_text_uses_text_roles() {
    let source = include_str!("../src/components_gallery.rs");
    let source = compact(source);

    for needle in [
        "decl_text::text_paragraph(cx,\"HoverCardcontent(overlay-root)\")",
        "decl_text::text_paragraph(cx,\"Movepointerfromtriggertocontent.\")",
        "decl_text::text_paragraph(cx,\"Popovercontent\")",
        "decl_text::text_paragraph(cx,\"overlays:tooltip/dropdown/context-menu/popover/dialog/alert-dialog/sheet\",)",
        "decl_text::text_control_readout(cx,format!(\"lastaction:{}\",last_action_value.as_ref()))",
        "decl_text::text_paragraph(cx,\"cmdk:Ctrl/Cmd+Popens,arrows/hoverhighlight,Enterselects\",)",
    ] {
        assert!(
            source.contains(needle),
            "components gallery overlay text should stay on shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "cx.text(\"HoverCardcontent(overlay-root)\")",
        "cx.text(\"Movepointerfromtriggertocontent.\")",
        "cx.text(\"Popovercontent\")",
        "cx.text(\"overlays:tooltip/dropdown/context-menu/popover/dialog/alert-dialog/sheet\")",
        "cx.text(format!(\"lastaction:{}\",last_action_value.as_ref()))",
        "cx.text(\"cmdk:Ctrl/Cmd+Popens,arrows/hoverhighlight,Enterselects\",)",
    ] {
        assert!(
            !source.contains(needle),
            "components gallery overlay text should not use bare text; unexpected `{needle}`"
        );
    }
}

#[test]
fn components_gallery_driver_writes_stay_behind_owner_helpers() {
    let source = include_str!("../src/components_gallery.rs");
    let compact_source = compact(source);

    for needle in [
        "fncomponents_gallery_update_model<T:Any>(",
        "fncomponents_gallery_set_model<T:Any>(",
        "fncomponents_gallery_set_last_action(",
        "fncomponents_gallery_open_command_palette(",
        "fncomponents_gallery_close_transient_surfaces(",
        "components_gallery_close_transient_surfaces(app,state);",
        "components_gallery_open_command_palette(app,state);",
        "components_gallery_set_last_action(app,state,\"context_menu.action\");",
    ] {
        assert!(
            compact_source.contains(needle),
            "components gallery should keep driver/event model writes behind explicit owner helpers; missing `{needle}`"
        );
    }

    assert_eq!(
        source.matches("models_mut().update(").count(),
        1,
        "components gallery should not scatter raw ModelStore updates outside the owner helper"
    );
}
