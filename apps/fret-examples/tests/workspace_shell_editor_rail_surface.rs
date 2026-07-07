#[test]
fn workspace_shell_demo_composes_editor_rail_through_workspace_frame_slots() {
    let source = include_str!("../src/workspace_shell_demo/driver.rs");

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
        "use fret::app::{AppRenderContext, text};",
        "fn workspace_shell_readout_text<'a, Cx>(",
        "fn workspace_shell_section_chrome_label<'a, Cx>(",
        "fn workspace_shell_paragraph_text<'a, Cx>(",
        "Cx: AppRenderContext<'a>,",
        "text::control_readout(cx, text)",
        "text::button_label(cx, label.clone())",
        "text::section_chrome_label(cx, text)",
        "text::paragraph(cx, text)",
        "workspace_shell_paragraph_text(\n                    cx,\n                    \"Workspace shell slot + editor-owned inner panel\",",
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
        "fret_ui_kit::ui::text(\"Workspace shell slot + editor-owned inner panel\")",
        ".text_color(fret_ui_kit::ColorRef::Color(muted))",
        "use fret_ui_kit::declarative::text as decl_text;",
        "decl_text::",
        "fret_ui::UiHost",
        "fret_ui::ElementContext<'_, H>",
    ] {
        assert!(
            !source.contains(needle),
            "workspace shell demo should keep editor-rail text on semantic roles; unexpected `{needle}`"
        );
    }
}
