fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn todo_demo_keeps_visible_text_on_roles() {
    let source = include_str!("../src/todo_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "fntodo_readout_text<'a,Cx,T>(cx:&mutCx,text:T)->implUiChild+use<Cx,T>",
        "fntodo_chrome_title_text<'a,Cx,T>(cx:&mutCx,text:T)->implUiChild+use<Cx,T>",
        "fntodo_compact_paragraph_text<'a,Cx,T>(cx:&mutCx,text:T)->implUiChild+use<Cx,T>",
        "fntodo_filter_label_text<'a,Cx,T>(cx:&mutCx,text:T)->implUiChild+use<Cx,T>",
        "fntodo_row_label_text<'a,Cx,T>(",
        "fntodo_attributed_row_label_text<'a,Cx>(",
        "Cx:fret::app::AppRenderContext<'a>,",
        "T:Into<Arc<str>>,",
        "text::control_readout(cx,text)",
        "text::chrome_title(cx,text)",
        "text::compact_paragraph(cx,text)",
        "text::button_label(cx,text)",
        "text::list_row_label_with_foreground(cx,text,foreground)",
        "text::list_row_label_attributed_with_foreground(cx,rich,foreground)",
        "todo_readout_text(cx,\"Addatasktogetstarted\")",
        "letcompleted_text=todo_readout_text(cx,\"Alltaskscompleted\");",
        "todo_readout_text(cx,format!(\"{active_count}{task_label}left\"))",
        "lettitle=todo_chrome_title_text(cx,\"Mytasks\");",
        "letprogress_label=todo_readout_text(cx,\"Progress\");",
        "letprogress_value=todo_readout_text(cx,format!(\"{:.0}%\",progress_pct));",
        "letempty_text=todo_compact_paragraph_text(cx,empty_label);",
        "letlabel=todo_filter_label_text(cx,filter.label()).into_element_in(cx);",
        "todo_attributed_row_label_text(cx,rich,muted_foreground)",
        "todo_row_label_text(cx,row_text.clone(),foreground)",
    ] {
        assert!(
            compact_source.contains(needle),
            "todo demo visible text should use shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(",
        "ui::rich_text(",
        "typography::",
        "textasdecl_text",
        "todo_readout_text(cx.elements()",
        "todo_filter_label_text(cx.elements()",
        "usefret_ui::element::AnyElement;",
        "usefret_ui::Invalidation;",
        ")->AnyElement",
    ] {
        assert!(
            !compact_source.contains(needle),
            "todo demo should not render app text with local text policy; unexpected `{needle}`"
        );
    }
}

#[test]
fn todo_demo_keeps_raw_view_runtime_harness_out_of_app_source() {
    let compact_source = compact(include_str!("../src/todo_demo.rs"));
    let compact_harness = compact(include_str!("../src/todo_demo_runtime_tests.rs"));

    for legacy in [
        "usefret::advanced::view::{",
        "view_init_window::<TodoDemoView>",
        "view_view(cx,state)",
        "ViewWindowState<TodoDemoView>",
        "UiTree::<App>::new()",
        "usefret_runtime::{FrameId,TickId};",
    ] {
        assert!(
            !compact_source.contains(legacy),
            "todo demo app source should not own raw view runtime harness code; unexpected `{legacy}`"
        );
    }

    for needle in [
        "usefret::advanced::view::{AppUiRenderRootState,ViewWindowState,render_root_with_app_ui,view_init_window,view_view,};",
        "fnrender_todo_demo_runtime_snapshot_for_frame(",
        "view_init_window::<TodoDemoView>(&mutapp,window)",
        "view_view(cx,state)",
        "ui.set_view_cache_enabled(true);",
    ] {
        assert!(
            compact_harness.contains(needle),
            "todo demo runtime harness should retain cache transition coverage; missing `{needle}`"
        );
    }
}
