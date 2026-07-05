fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn drop_shadow_demo_uses_app_view_imports_with_explicit_effect_seams() {
    let source = include_str!("../src/drop_shadow_demo.rs");
    let compact = compact(source);

    for needle in [
        "usefret::app::prelude::*;",
        "usefret_core::scene::{Color,DropShadowV1,EffectChain,EffectMode,EffectQuality,EffectStep};",
        "usefret_ui::element::{AnyElement,ContainerProps,LayoutStyle,Length,Overflow,SizeStyle,SpacerProps,};",
        "usefret_ui::{ElementContext,UiHost};",
        "fninstall_demo_theme(app:&mutApp)",
        "fninit(_app:&mutApp,_window:WindowId)->Self",
        ".view::<DropShadowDemoView>()?",
        "EffectStep::DropShadowV1(DropShadowV1{",
    ] {
        assert!(
            compact.contains(needle),
            "drop_shadow_demo should keep app entry imports and explicit effect seams; missing `{needle}`",
        );
    }

    for forbidden in [
        "advanced::prelude::*",
        "component::prelude::*",
        "KernelApp",
        "AppWindowId",
    ] {
        assert!(
            !source.contains(forbidden),
            "drop_shadow_demo should not reintroduce broad or kernel-facing imports: `{forbidden}`",
        );
    }
}
