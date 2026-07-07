fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn embedded_viewport_demo_keeps_fixed_chrome_text_on_roles() {
    let source = include_str!("../src/embedded_viewport_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::{AppComponentCx,AppRenderContext,text};",
        "fnembedded_viewport_button_label_text<'a,Cx>(",
        "fnembedded_viewport_readout_text<'a,Cx>(",
        "Cx:AppRenderContext<'a>,",
        "text::button_label(cx,text)",
        "text::control_readout(cx,text)",
        "embedded_viewport_button_label_text(cx,\"640×360\")",
        "embedded_viewport_button_label_text(cx,\"960×540\")",
        "embedded_viewport_button_label_text(cx,\"1280×720\")",
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
        "usefret_ui_kit::declarative::textasdecl_text;",
        "decl_text::",
        "text_button_label(",
        "text_control_readout(",
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
        "usefret::app::{AppComponentCx,AppRenderContext,text};",
        "usefret_ui::ThemeSnapshot;",
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

#[test]
fn embedded_viewport_demo_model_writes_stay_behind_owner_helper() {
    let source = include_str!("../src/embedded_viewport_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret_runtime::{FrameId,ModelStore,TickId};",
        "structEmbeddedViewportDemoModelOwner<'a>{",
        "models:&'amutModelStore,",
        "fnset_last_input(&mutself,models:&embedded::EmbeddedViewportModels,input:implInto<Arc<str>>,)->bool{",
        "self.models.update(&models.last_input,|value|{*value=input;true}).unwrap_or(false)",
        "EmbeddedViewportDemoModelOwner::new(app.models_mut()).set_last_input(&models,\"Clickinsidetheviewportpaneltoseeinputforwarding.\",);",
    ] {
        assert!(
            compact.contains(needle),
            "embedded viewport demo should route model writes through its owner helper; missing `{needle}`"
        );
    }

    assert_eq!(
        compact.matches("models_mut().update(").count(),
        0,
        "embedded viewport demo should not scatter direct model store writes outside the owner helper"
    );
    assert!(
        !compact.contains("ModelStore::update("),
        "embedded viewport demo should not bypass the owner helper with UFCS ModelStore writes"
    );
}
