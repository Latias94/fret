fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

fn assert_custom_effect_overlay_text_roles(source: &str, label: &str) {
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fncustom_effect_label_text<H:UiHost>(",
        "decl_text::text_section_chrome_label(cx,text).inherit_foreground(srgb(255,255,255,0.92))",
        "lettitle=custom_effect_label_text(cx,label.clone());",
    ] {
        assert!(
            source.contains(needle),
            "{label} should keep fixed overlay labels on shared chrome text roles; missing `{needle}`",
        );
    }

    for needle in [
        "cx.text_props(TextProps{",
        "TextProps{layout:Default::default(),text:label.clone()",
        "wrap:fret_core::TextWrap::None",
        "overflow:fret_core::TextOverflow::Clip",
    ] {
        assert!(
            !source.contains(needle),
            "{label} should not render overlay labels with local TextProps policy; unexpected `{needle}`",
        );
    }
}

fn assert_custom_effect_v2_web_overlay_text_roles(source: &str) {
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnoverlay_label_text<H:UiHost>(",
        "fnoverlay_readout_text<H:UiHost>(",
        "decl_text::text_section_chrome_label(cx,text).inherit_foreground(Self::srgb(255,255,255,0.92))",
        "decl_text::text_control_readout(cx,text).inherit_foreground(foreground)",
        "Self::overlay_readout_text(cx,\"CustomV2unsupportedonthisadapter/backend\",theme.color_token(\"muted_foreground\"),)",
        "letbadge_text=Self::overlay_label_text(cx,\"CustomEffectV2(WebGPU)\");",
        "Self::overlay_readout_text(cx,\"PressVtotogglethedemosurface.PressRtoresetcontrols.\",Self::with_alpha(theme.color_token(\"foreground\"),0.55),)",
    ] {
        assert!(
            source.contains(needle),
            "custom_effect_v2_web_demo should keep fixed overlay/readout text on shared roles; missing `{needle}`",
        );
    }

    for needle in [
        "cx.text_props(TextProps{",
        "TextProps{",
        "wrap:fret_core::TextWrap::None",
        "overflow:fret_core::TextOverflow::Clip",
    ] {
        assert!(
            !source.contains(needle),
            "custom_effect_v2_web_demo should not render overlay/readout text with local TextProps policy; unexpected `{needle}`",
        );
    }
}

fn assert_custom_effect_v2_template_overlay_text_roles(
    source: &str,
    label: &str,
    badge: &str,
    hint: &str,
) {
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnoverlay_label_text<H:UiHost>(",
        "fnoverlay_readout_text<H:UiHost>(",
        "decl_text::text_section_chrome_label(cx,text).inherit_foreground(Self::srgb(255,255,255,0.92))",
        "decl_text::text_control_readout(cx,text).inherit_foreground(foreground)",
        "Self::overlay_readout_text(cx,\"CustomV2unsupportedonthisadapter/backend\",theme.color_token(\"muted_foreground\"),)",
        &format!("letbadge_text=Self::overlay_label_text(cx,\"{badge}\");"),
        &format!(
            "Self::overlay_readout_text(cx,\"{hint}\",Self::with_alpha(theme.color_token(\"foreground\"),0.55),)"
        ),
    ] {
        assert!(
            source.contains(needle),
            "{label} should keep fixed overlay/readout text on shared roles; missing `{needle}`",
        );
    }

    for needle in [
        "cx.text_props(TextProps{",
        "TextProps{",
        "wrap:fret_core::TextWrap::None",
        "overflow:fret_core::TextOverflow::Clip",
    ] {
        assert!(
            !source.contains(needle),
            "{label} should not render overlay/readout text with local TextProps policy; unexpected `{needle}`",
        );
    }
}

fn assert_custom_effect_v2_glass_chrome_text_roles(source: &str) {
    let source = compact(source);

    for needle in [
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fncontrol_label_text<H:UiHost>(",
        "fncontrol_readout_text<H:UiHost>(",
        "decl_text::text_control_label(cx,text).inherit_foreground(foreground)",
        "decl_text::text_control_readout(cx,text).inherit_foreground(foreground)",
        "Self::control_label_text(cx,label,theme.color_token(\"muted_foreground\"))",
        "Self::control_readout_text(cx,value,theme.color_token(\"foreground\"))",
        "Self::control_readout_text(cx,\"CustomV2unsupportedonthisadapter/backend\",theme.color_token(\"muted_foreground\"),)",
    ] {
        assert!(
            source.contains(needle),
            "custom_effect_v2_glass_chrome_web_demo should keep fixed control text on shared roles; missing `{needle}`",
        );
    }

    for needle in [
        "cx.text_props(TextProps{",
        "TextProps{",
        "wrap:fret_core::TextWrap::None",
        "overflow:fret_core::TextOverflow::Clip",
    ] {
        assert!(
            !source.contains(needle),
            "custom_effect_v2_glass_chrome_web_demo should not render control text with local TextProps policy; unexpected `{needle}`",
        );
    }
}

fn assert_custom_effect_v3_overlay_text_roles(source: &str) {
    let source = compact(source);

    for needle in [
        "usefret::app::{AppComponentCx,AppRenderContext,LocalState};",
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnoverlay_label_text<'a,Cx>(",
        "Cx:AppRenderContext<'a>,",
        "decl_text::text_section_chrome_label(cx.elements(),text).inherit_foreground(Color{r:1.0,g:1.0,b:1.0,a:0.92,})",
        "letlabel_text=overlay_label_text(cx,title);",
        "move|_cx|vec![label_text]",
    ] {
        assert!(
            source.contains(needle),
            "custom_effect_v3_demo should keep fixed overlay labels on shared chrome text roles; missing `{needle}`",
        );
    }

    for needle in [
        "cx.text_props(TextProps{",
        "TextProps{",
        "wrap:fret_core::TextWrap::None",
        "overflow:fret_core::TextOverflow::Clip",
    ] {
        assert!(
            !source.contains(needle),
            "custom_effect_v3_demo should not render overlay labels with local TextProps policy; unexpected `{needle}`",
        );
    }
}

fn assert_postprocess_theme_overlay_text_roles(source: &str) {
    let source = compact(source);

    for needle in [
        "usefret::app::{AppComponentCx,AppRenderContext,LocalState};",
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnpostprocess_title_text<'a,Cx>(",
        "fnpostprocess_readout_text<'a,Cx>(",
        "decl_text::text_section_chrome_label(cx.elements(),text).inherit_foreground(srgb(255,255,255,0.92))",
        "decl_text::text_control_readout(cx.elements(),text).inherit_foreground(srgb(255,255,255,0.68))",
        "lettitle=postprocess_title_text(cx,\"Theme-likePostprocess(CustomV1)\");",
        "letsubtitle=postprocess_readout_text(cx,\"Scanlines+vignette+chromatic+grain(bounded,deterministic).\",);",
    ] {
        assert!(
            source.contains(needle),
            "postprocess_theme_demo should keep fixed title/readout text on shared roles; missing `{needle}`",
        );
    }

    for needle in [
        "cx.text_props(TextProps{",
        "TextProps{",
        "wrap:fret_core::TextWrap::None",
        "overflow:fret_core::TextOverflow::Clip",
    ] {
        assert!(
            !source.contains(needle),
            "postprocess_theme_demo should not render overlay text with local TextProps policy; unexpected `{needle}`",
        );
    }
}

fn assert_liquid_glass_overlay_text_roles(source: &str) {
    let source = compact(source);

    for needle in [
        "usefret::app::{AppComponentCx,AppRenderContext,LocalState};",
        "usefret_ui_kit::declarative::textasdecl_text;",
        "fnliquid_glass_overlay_text<H:UiHost>(",
        "fnliquid_glass_card_title_text<'a,Cx>(",
        "decl_text::text_section_chrome_label(cx,text).inherit_foreground(srgb(255,255,255,0.92))",
        "decl_text::text_section_chrome_label(cx.elements(),text).inherit_foreground(srgb(255,255,255,0.92))",
        "lettitle=liquid_glass_overlay_text(cx,label.clone());",
        "lettitle=liquid_glass_card_title_text(cx,title);",
    ] {
        assert!(
            source.contains(needle),
            "liquid_glass_demo should keep fixed overlay/card text on shared chrome roles; missing `{needle}`",
        );
    }

    for needle in [
        "cx.text_props(TextProps{",
        "TextProps{",
        "wrap:fret_core::TextWrap::None",
        "overflow:fret_core::TextOverflow::Clip",
    ] {
        assert!(
            !source.contains(needle),
            "liquid_glass_demo should not render overlay/card text with local TextProps policy; unexpected `{needle}`",
        );
    }
}

#[test]
fn custom_effect_v1_v2_overlay_labels_use_shared_chrome_role() {
    assert_custom_effect_overlay_text_roles(
        include_str!("../src/custom_effect_v1_demo.rs"),
        "custom_effect_v1_demo",
    );
    assert_custom_effect_overlay_text_roles(
        include_str!("../src/custom_effect_v2_demo.rs"),
        "custom_effect_v2_demo",
    );
}

#[test]
fn custom_effect_v1_demo_uses_app_view_imports_with_explicit_effect_hook() {
    let source = include_str!("../src/custom_effect_v1_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret::advanced::driver::UiAppBuilderAdvancedExtas_;",
        "usefret::advanced::raw::{LocalStateElementContextExtas_,LocalStateRawModelExtas_};",
        "usefret::app::prelude::*;",
        "usefret::app::{AppComponentCx,LocalState};",
        "usefret_ui_kit::declarative::action_hooks::ActionHooksExtas_;",
        "usefret_ui_kit::{IntoUiElement,Space,UiSupportsLayoutas_,ui};",
        ".view::<CustomEffectV1View>()?",
        ".install_custom_effects(install_custom_effect)",
        "fninstall_demo_theme(app:&mutApp)",
        "fninstall_custom_effect(app:&mutApp,effects:&mutdynfret_core::CustomEffectService)",
        "fninit(_app:&mutApp,_window:WindowId)->Self",
        "fnview(cx:&mutElementContext<'_,App>,st:&mutCustomEffectV1State)->Ui",
        "implIntoUiElement<App>",
    ] {
        assert!(
            compact.contains(needle),
            "custom_effect_v1_demo should keep app view imports and explicit effect hooks; missing `{needle}`",
        );
    }

    for forbidden in [
        "advanced::prelude::*",
        "component::prelude::*",
        "KernelApp",
        "AppWindowId",
        "ViewElements",
    ] {
        assert!(
            !source.contains(forbidden),
            "custom_effect_v1_demo should not reintroduce broad or kernel-facing imports: `{forbidden}`",
        );
    }
}

#[test]
fn custom_effect_v2_demo_uses_app_view_imports_with_explicit_effect_hooks() {
    let source = include_str!("../src/custom_effect_v2_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret::UiAppBuilder;",
        "usefret::advanced::driver::UiAppBuilderAdvancedExtas_;",
        "usefret::advanced::raw::{LocalStateElementContextExtas_,LocalStateRawModelExtas_};",
        "usefret::app::prelude::*;",
        "usefret::app::{AppComponentCx,LocalState};",
        "usefret_ui_kit::declarative::action_hooks::ActionHooksExtas_;",
        "usefret_ui_kit::{IntoUiElement,Space,UiSupportsLayoutas_,ui};",
        ".view::<CustomEffectV2View>()?",
        "fninstall_into<S:'static>(builder:UiAppBuilder<S>)->UiAppBuilder<S>",
        ".install_custom_effects(register_custom_effect)",
        ".on_gpu_ready(upload_input_image)",
        "fninstall_app_globals(app:&mutApp)",
        "fnregister_custom_effect(app:&mutApp,effects:&mutdynfret_core::CustomEffectService)",
        "fnupload_input_image(app:&mutApp,context:&WgpuContext,renderer:&mutRenderer)",
        "fninit(_app:&mutApp,_window:WindowId)->Self",
        "fnview(cx:&mutElementContext<'_,App>,st:&mutCustomEffectV2State)->Ui",
        "implIntoUiElement<App>",
    ] {
        assert!(
            compact.contains(needle),
            "custom_effect_v2_demo should keep app view imports and explicit effect hooks; missing `{needle}`",
        );
    }

    for forbidden in [
        "advanced::prelude::*",
        "component::prelude::*",
        "KernelApp",
        "AppWindowId",
        "ViewElements",
    ] {
        assert!(
            !source.contains(forbidden),
            "custom_effect_v2_demo should not reintroduce broad or kernel-facing imports: `{forbidden}`",
        );
    }
}

#[test]
fn custom_effect_v3_demo_uses_app_view_imports_with_explicit_effect_hooks() {
    let source = include_str!("../src/custom_effect_v3_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret::UiAppBuilder;",
        "usefret::advanced::driver::UiAppBuilderAdvancedExtas_;",
        "usefret::advanced::raw::{LocalStateElementContextExtas_,LocalStateRawModelExtas_};",
        "usefret::app::prelude::*;",
        "usefret::app::{AppComponentCx,AppRenderContext,LocalState};",
        "usefret_ui_kit::declarative::action_hooks::ActionHooksExtas_;",
        "usefret_ui_kit::{IntoUiElement,Space,UiSupportsLayoutas_,ui};",
        "usefret_ui_shadcn::facadeasshadcn;",
        ".view::<CustomEffectV3View>()?",
        "fninstall_into<S:'static>(builder:UiAppBuilder<S>)->UiAppBuilder<S>",
        ".install_custom_effects(register_custom_effect_v3)",
        ".on_gpu_ready(upload_user0_images)",
        "fninstall_app_globals(app:&mutApp)",
        "fnregister_custom_effect_v3(app:&mutApp,effects:&mutdynfret_core::CustomEffectService)",
        "fnupload_user0_images(app:&mutApp,context:&WgpuContext,renderer:&mutRenderer)",
        "fninit(_app:&mutApp,_window:WindowId)->Self",
        "fnview(cx:&mutElementContext<'_,App>,st:&mutState)->Ui",
        "implIntoUiElement<App>",
    ] {
        assert!(
            compact.contains(needle),
            "custom_effect_v3_demo should keep app view imports and explicit effect hooks; missing `{needle}`",
        );
    }

    for forbidden in [
        "use fret::{FretApp",
        "advanced::prelude::*",
        "component::prelude::*",
        "KernelApp",
        "AppWindowId",
        "ViewElements",
    ] {
        assert!(
            !source.contains(forbidden),
            "custom_effect_v3_demo should not reintroduce broad or kernel-facing imports: `{forbidden}`",
        );
    }
}

#[test]
fn postprocess_theme_demo_uses_app_view_imports_with_explicit_effect_hook() {
    let source = include_str!("../src/postprocess_theme_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret::advanced::driver::UiAppBuilderAdvancedExtas_;",
        "usefret::advanced::raw::{LocalStateElementContextExtas_,LocalStateRawModelExtas_};",
        "usefret::app::prelude::*;",
        "usefret::app::{AppComponentCx,AppRenderContext,LocalState};",
        "usefret_ui::Theme;",
        "usefret_ui_kit::declarative::action_hooks::ActionHooksExtas_;",
        "usefret_ui_kit::{IntoUiElement,IntoUiElementInExtas_,LayoutRefinement,Space,UiSupportsLayoutas_,ui,};",
        "usefret_ui_shadcn::facadeasshadcn;",
        ".view::<ThemePostprocessView>()?",
        ".install_custom_effects(install_custom_effect)",
        "fninstall_demo_theme(app:&mutApp)",
        "fninstall_custom_effect(app:&mutApp,effects:&mutdynfret_core::CustomEffectService)",
        "fninit(_app:&mutApp,_window:WindowId)->Self",
        "implIntoUiElement<App>",
    ] {
        assert!(
            compact.contains(needle),
            "postprocess_theme_demo should keep app view imports and explicit effect hook; missing `{needle}`",
        );
    }

    for forbidden in [
        "use fret::{FretApp",
        "advanced::prelude::*",
        "component::prelude::*",
        "KernelApp",
        "AppWindowId",
        "ViewElements",
    ] {
        assert!(
            !source.contains(forbidden),
            "postprocess_theme_demo should not reintroduce broad or kernel-facing imports: `{forbidden}`",
        );
    }
}

#[test]
fn liquid_glass_demo_uses_app_view_imports_with_explicit_effect_hooks() {
    let source = include_str!("../src/liquid_glass_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret::advanced::driver::UiAppBuilderAdvancedExtas_;",
        "usefret::advanced::raw::{LocalStateElementContextExtas_,LocalStateRawModelExtas_};",
        "usefret::app::prelude::*;",
        "usefret::app::{AppComponentCx,AppRenderContext,LocalState};",
        "usefret_ui::{ElementContext,Invalidation,Theme,UiHost};",
        "usefret_ui_kit::declarative::action_hooks::ActionHooksExtas_;",
        "usefret_ui_kit::{IntoUiElement,Space,UiSupportsLayoutas_,ui};",
        "usefret_ui_shadcn::facadeasshadcn;",
        ".view::<LiquidGlassView>()?",
        ".install_custom_effects(install_custom_effects)",
        "fninstall_demo_theme(app:&mutApp)",
        "fninstall_custom_effects(app:&mutApp,effects:&mutdynfret_core::CustomEffectService)",
        "fninit(app:&mutApp,_window:WindowId)->Self",
        "fnview(cx:&mutElementContext<'_,App>,st:&mutLiquidGlassState)->Ui",
        "implIntoUiElement<H>",
    ] {
        assert!(
            compact.contains(needle),
            "liquid_glass_demo should keep app view imports and explicit effect hooks; missing `{needle}`",
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
            "liquid_glass_demo should not reintroduce broad or kernel-facing imports: `{forbidden}`",
        );
    }
}

#[test]
fn custom_effect_v3_and_effect_reference_chrome_use_shared_roles() {
    assert_custom_effect_v3_overlay_text_roles(include_str!("../src/custom_effect_v3_demo.rs"));
    assert_postprocess_theme_overlay_text_roles(include_str!("../src/postprocess_theme_demo.rs"));
    assert_liquid_glass_overlay_text_roles(include_str!("../src/liquid_glass_demo.rs"));
}

#[test]
fn custom_effect_v2_web_overlay_readouts_use_shared_roles() {
    assert_custom_effect_v2_web_overlay_text_roles(include_str!(
        "../src/custom_effect_v2_web_demo.rs"
    ));
}

#[test]
fn custom_effect_v2_web_templates_use_shared_text_roles() {
    assert_custom_effect_v2_template_overlay_text_roles(
        include_str!("../src/custom_effect_v2_identity_web_demo.rs"),
        "custom_effect_v2_identity_web_demo",
        "CustomEffectV2(Starter)",
        "PressVtotogglethelens.PressRtoresetcontrols.",
    );
    assert_custom_effect_v2_template_overlay_text_roles(
        include_str!("../src/custom_effect_v2_lut_web_demo.rs"),
        "custom_effect_v2_lut_web_demo",
        "CustomEffectV2(LUT)",
        "PressVtotogglethedemosurface.PressRtoresetcontrols.",
    );
    assert_custom_effect_v2_glass_chrome_text_roles(include_str!(
        "../src/custom_effect_v2_glass_chrome_web_demo.rs"
    ));
}
