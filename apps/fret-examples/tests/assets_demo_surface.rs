fn compact(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn assets_demo_uses_app_ui_assets_facade_for_render_helpers() {
    let source = include_str!("../src/assets_demo.rs");
    let source = compact(source);

    for needle in [
        "usefret::app::prelude::*;",
        "usefret::app::{AppComponentCx,text};",
        "Cx:fret::app::AppRenderContext<'a>,",
        ".view::<AssetsDemoView>()?",
        "ui_assets::rgba8_image_state(cx,96,96,checker_rgba.as_slice(),ui_assets::ImageColorSpace::Srgb,)",
        "letimage_stats=ui_assets::image_stats(cx);",
        "letsvg_stats=ui_assets::svg_stats(cx);",
        "text::control_readout(cx,Arc::<str>::from(line)).inherit_foreground(muted)",
    ] {
        assert!(
            source.contains(needle),
            "assets demo should keep render-time asset helpers on the app ui_assets facade; missing `{needle}`"
        );
    }

    for needle in [
        "usefret_ui_assets::ui::",
        "use_rgba8_image_state_in(cx",
        "image_stats_in(cx)",
        "svg_stats_in(cx)",
        "Cx:fret::app::RenderContextAccess<'a,KernelApp>,",
        "advanced::prelude::*",
        "component::prelude::*",
        "view_with_hooks::<AssetsDemoView>",
        "fnon_event(",
        "fret::advanced::raw::UiTree",
        "AssetsDemoImageEvents",
        "textasdecl_text",
        "decl_text::",
    ] {
        assert!(
            !source.contains(needle),
            "assets demo should not expose raw fret-ui-assets UI helpers from the app render surface; unexpected `{needle}`"
        );
    }
}
