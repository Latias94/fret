#[test]
fn workspace_shell_demo_composes_editor_rail_through_workspace_frame_slots() {
    let source = include_str!("../src/workspace_shell_demo.rs");

    for needle in [
        "fn workspace_shell_editor_rail<'a, Cx>(",
        "Cx: fret::app::ElementContextAccess<'a, App>,",
        "let right = cx.keyed(\"workspace_shell.right\"",
        "workspace_shell_editor_rail(",
        "InspectorPanel::new(None)",
        ".into_element_in(",
        "PropertyGroup::new(\"Selection\")",
        "PropertyGroup::new(\"Shell\")",
        "PropertyGrid::new().into_element_in(",
        "use fret_ui_kit::declarative::text as decl_text;",
        "fn workspace_shell_readout_text<",
        "fn workspace_shell_section_chrome_label<",
        "decl_text::text_control_readout(cx, text)",
        "decl_text::text_button_label(cx, label.clone())",
        "decl_text::text_section_chrome_label(cx, text)",
        "row_cx.label_text(cx, \"Active pane\")",
        "row_cx.label_text(cx, \"Dirty close prompt\")",
        "active_pane_label.clone(),",
        "if prompt_open { \"Open\" } else { \"Closed\" },",
        "workspace_shell_section_chrome_label(",
        "workspace_shell_readout_text(",
        "reason={reason} active={active_tab} close_count={close_count}",
        ".right(right)",
        "\"workspace-shell-editor-rail\"",
    ] {
        assert!(
            source.contains(needle),
            "workspace shell demo should keep the editor-rail composition explicit; missing `{needle}`"
        );
    }

    for needle in [
        "move |cx| vec![cx.text(label.clone())]",
        "|cx| cx.text(\"Active pane\")",
        "|cx| cx.text(\"Dirty close prompt\")",
        "|cx| cx.text(active_pane_label.clone())",
        "cx.text(Arc::<str>::from(\n                                                        \"Dirty close confirmation\",\n                                                    ))",
        "cx.text(Arc::<str>::from(format!(\n                                                        \"reason={reason} active={active_tab} close_count={close_count}\"",
        "cx.text(Arc::<str>::from(format!(\n                                                        \"dirty=[{dirty_list}]\"",
    ] {
        assert!(
            !source.contains(needle),
            "workspace shell demo should keep editor-rail text on semantic roles; unexpected `{needle}`"
        );
    }
}
