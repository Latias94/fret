fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn async_playground_demo_keeps_visible_text_on_roles() {
    let source = include_str!("../src/async_playground_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "usefret::app::{AppRenderContext,LocalState,RenderContextAccessas_,pressable,text};",
        "fnasync_chrome_title_text<'a,Cx,T>(cx:&mutCx,text:T)->AnyElement",
        "text::chrome_title(cx,text)",
        "text::section_chrome_label(cx,text)",
        "text::list_row_label(cx,text)",
        "text::control_readout(cx,text)",
        "text::code_label(cx,text)",
        "text::compact_paragraph(cx,text)",
        "lettitle=async_chrome_title_text(cx,\"AsyncPlayground\");",
        "letslow_label=async_readout_text(cx,\"Slownetwork(x2)\");",
        "letheader=async_section_text(cx,\"Catalog\");",
        "pressable::command_button(cx,select_cmd,id.label(),move|cx,st_press|{",
        "lettitle=async_list_row_text(cx,id.label());",
        "lettitle=async_section_text(cx,selected.label());",
        "out.push(async_code_label_text(cx,key.namespace()));",
        "out.push(async_readout_text(cx,format!(\"status:{status:?}\")));",
        "letpolicy_editor=policy_editor(cx,st,selected);",
        "letkeep_prev_label=async_readout_text(cx,\"keepPreviousDataWhileLoading\");",
        "letfail_label=async_readout_text(cx,\"failmode\");",
        "async_section_text(cx,\"Policy\")",
        "letinputs=query_inputs_row(cx,locals,id);",
        "letview=query_result_view(cx,id,key,&state,snap.as_ref(),&policy);",
        "children.push(async_compact_paragraph_text(cx,matchid{",
        "letleft=async_code_label_text(cx,id.namespace());",
        "QueryStatus::Idle=>async_compact_paragraph_text(cx,\"Idle(notfetchedyet).\"),",
        "QueryStatus::Success=>async_compact_paragraph_text(",
        "lettitle=async_section_text(cx,\"Result\");",
    ] {
        assert!(
            compact_source.contains(needle),
            "async playground visible text should use shared text roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(",
        "ui::rich_text(",
        ".font_semibold()",
        ".font_medium()",
        ".truncate()",
        "text_color(ColorRef::Color(theme.color_token(\"muted-foreground\")))",
        "policy_editor(cx,st,theme.clone(),selected)",
        "query_inputs_row(cx,locals,theme.clone(),id)",
        "query_result_view(cx,theme,id,",
        "decl_text::",
        "ElementContext",
        "UiHost",
        "PressableProps",
        "PressableA11y",
        "pressable_dispatch_command_if_enabled",
    ] {
        assert!(
            !compact_source.contains(needle),
            "async playground should not render app text with local text policy; unexpected `{needle}`"
        );
    }
}

#[test]
fn async_playground_demo_uses_app_view_imports() {
    let source = include_str!("../src/async_playground_demo.rs");
    let compact_source = compact(source);

    for needle in [
        "usefret::app::prelude::*;",
        "usefret::app::{AppRenderContext,LocalState,RenderContextAccessas_,pressable,text};",
        "usefret::actions::CommandId;",
        "usefret::scroll::ScrollHandle;",
        "usefret::style::ThemeSnapshot;",
        ".view::<AsyncPlaygroundView>()?",
        "fninstall_tokio_spawner(app:&mutApp)",
        "fnapply_theme(app:&mutApp,dark:bool)",
        "fninstall_light_theme(app:&mutApp)",
        "fnnew(app:&mutApp,initial:Option<&'staticstr>)->Self",
        "fnnew(app:&mutApp)->Self",
        "fninit(app:&mutApp,_window:WindowId)->Self",
        "fnrender(&mutself,cx:&mutAppUi<'_,'_>)->Ui",
    ] {
        assert!(
            compact_source.contains(needle),
            "async playground should stay on app-facing view imports; missing `{needle}`"
        );
    }

    for forbidden in [
        "advanced::prelude::*",
        "component::prelude::*",
        "use fret_core::",
        "fret_core::",
        "ElementContext",
        "UiHost",
        "PressableProps",
        "PressableA11y",
        "pressable_dispatch_command_if_enabled",
        "KernelApp",
        "AppWindowId",
    ] {
        assert!(
            !source.contains(forbidden),
            "async playground should not reintroduce broad or kernel-facing imports: `{forbidden}`"
        );
    }
}
