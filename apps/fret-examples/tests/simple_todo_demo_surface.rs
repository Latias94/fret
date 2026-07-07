fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn simple_todo_demo_keeps_visible_text_on_roles() {
    let source = include_str!("../src/simple_todo_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "fnsimple_todo_readout_text",
        "fnsimple_todo_compact_paragraph_text",
        "fnsimple_todo_row_label_text",
        "Cx:AppRenderContext<'a>",
        "text::control_readout(cx,text)",
        "text::compact_paragraph(cx,text)",
        "text::list_row_label_with_foreground(cx,text,foreground)",
        "letsummary=simple_todo_readout_text(cx,status_text);",
        "letempty_text=simple_todo_compact_paragraph_text(",
        "\"Notasksyet.Addoneabove.\",",
        "letremaining=simple_todo_readout_text(cx,format!(\"{active_count}left\"));",
        "lettext=simple_todo_row_label_text(cx,row_text.clone(),ColorRef::Color(row_text_foreground));",
    ] {
        assert!(
            compact_source.contains(needle),
            "simple todo visible text should use shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(status_text)",
        "ui::text(\"Notasksyet.Addoneabove.\")",
        "ui::text(format!(\"{active_count}left\"))",
        "ui::text(row.text.clone())",
        "ui::text(",
        "cx.elements()",
        "usefret_ui_kit::declarative::textasdecl_text;",
    ] {
        assert!(
            !compact_source.contains(needle),
            "simple todo should not render app text through local/raw text policy; unexpected `{needle}`"
        );
    }
}

#[test]
fn simple_todo_driver_uses_shared_default_view_demo_launch_helpers() {
    let compact_source = compact(include_str!("../src/simple_todo_demo/driver.rs"));

    for needle in [
        "crate::build_default_view_demo_app()",
        "crate::build_default_view_demo_runner_config(\"fret-demosimple-todo\",(560.0,520.0))",
        "crate::build_default_view_demo_fn_driver::<SimpleTodoView>(\"simple-todo-demo\")",
        "install_demo_icons(&mutapp);",
        "install_demo_theme(&mutapp);",
    ] {
        assert!(
            compact_source.contains(needle),
            "simple todo driver should use shared default-view launch helpers; missing `{needle}`"
        );
    }

    for legacy in [
        "usefret_bootstrap::ui_app_driver;",
        "ui_app_driver::UiAppDriver::new(",
        "usefret_runtime::PlatformCapabilities;",
        "fret::advanced::view::view_init_window",
        "fret::advanced::view::view_view",
        "fret::advanced::view::ViewWindowState",
    ] {
        assert!(
            !compact_source.contains(legacy),
            "simple todo driver should not own raw view-driver wiring; unexpected `{legacy}`"
        );
    }
}
