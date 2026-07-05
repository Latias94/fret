fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn embedded_viewport_demo_keeps_fixed_chrome_text_on_roles() {
    let source = include_str!("../src/embedded_viewport_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnembedded_viewport_button_label_text<H:UiHost>(",
        "fnembedded_viewport_readout_text<H:UiHost>(",
        "decl_text::text_button_label(cx,text)",
        "decl_text::text_control_readout(cx,text)",
        "embedded_viewport_button_label_text(cx.elements(),\"640×360\",)",
        "embedded_viewport_button_label_text(cx.elements(),\"960×540\",)",
        "embedded_viewport_button_label_text(cx.elements(),\"1280×720\",)",
        "embedded_viewport_readout_text(cx,format!(\"Target:{preset_label}\"))",
        "embedded_viewport_readout_text(cx,format!(\"Clicks:{clicks}\"))",
        "embedded_viewport_readout_text(cx,format!(\"Lastinput:{last_input}\"))",
    ] {
        assert!(
            source.contains(needle),
            "embedded viewport demo should keep fixed chrome/readout text on shared roles; missing `{needle}`"
        );
    }

    for needle in [
        "ui::text(\"640×360\")",
        "ui::text(\"960×540\")",
        "ui::text(\"1280×720\")",
        "ui::text(format!(\"Target:{preset_label}\"))",
        "ui::text(format!(\"Clicks:{clicks}\"))",
        "ui::text(format!(\"Lastinput:{last_input}\"))",
    ] {
        assert!(
            !source.contains(needle),
            "embedded viewport demo should not render fixed chrome/readouts with bare wrapping text; unexpected `{needle}`"
        );
    }
}

#[test]
fn embedded_viewport_demo_uses_app_view_imports_with_explicit_interop_hooks() {
    let source = include_str!("../src/embedded_viewport_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret::advanced::driver::UiAppBuilderAdvancedExtas_;",
        "usefret::advanced::interop::embedded_viewport::{selfasembedded,EmbeddedViewportUiAppDriverExtas_,};",
        "usefret::app::prelude::*;",
        "usefret::app::AppComponentCx;",
        "usefret_ui::{ElementContext,ThemeSnapshot,UiHost};",
        "usefret_ui_kit::declarative::ElementContextThemeExtas_;",
        "usefret_ui_kit::declarative::UiElementTestIdExtas_;",
        "usefret_ui_kit::{ColorRef,IntoUiElement,IntoUiElementInExtas_,LayoutRefinement,Radius,Space,UiSupportsLayoutas_,ui,};",
        "usefret_ui_shadcn::facadeasshadcn;",
        "fninit(app:&mutApp,window:WindowId)->Self",
        "Cx:fret::app::ElementContextAccess<'a,App>,",
        "C:IntoUiElement<App>,",
        "fnrecord_embedded_viewport(&mutself,app:&mutApp,window:WindowId,",
        ".view_with_hooks::<EmbeddedViewportDemoView>(|d|d.drive_embedded_viewport())?",
        "fninstall_demo_theme(app:&mutApp)",
    ] {
        assert!(
            compact.contains(needle),
            "embedded viewport demo should keep app view imports and explicit interop hooks; missing `{needle}`"
        );
    }

    for forbidden in [
        "use fret::{FretApp",
        "advanced::prelude::*",
        "component::prelude::*",
        "KernelApp",
        "AppWindowId",
        "ViewElements",
        "IntoUiElement<KernelApp>",
    ] {
        assert!(
            !source.contains(forbidden),
            "embedded viewport demo should not reintroduce broad or kernel-facing imports: `{forbidden}`"
        );
    }
}
