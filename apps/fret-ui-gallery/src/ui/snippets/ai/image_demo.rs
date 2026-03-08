pub const SOURCE: &str = include_str!("image_demo.rs");

// region: example
use crate::driver::UiGalleryImageSourceDemoAssets;
use fret_core::Px;
use fret_ui_ai as ui_ai;
use fret_ui_assets as ui_assets;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;
use ui_assets::ui::ImageSourceElementContextExt as _;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let Some(assets) = cx.app.global::<UiGalleryImageSourceDemoAssets>().cloned() else {
        return cx.text("Image demo assets missing (expected UiGalleryDriver init).");
    };

    let state = cx.use_image_source_state(&assets.square_png);
    let status_line = cx
        .text(format!("status={:?}", state.status))
        .test_id("ui-ai-image-demo-status");

    let border = cx.with_theme(|theme| theme.color_token("border"));
    let image = state.image.map(|id| {
        ui_ai::Image::new(id)
            .alt("Generated image")
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(border)),
            )
            .refine_layout(LayoutRefinement::default().w_px(Px(300.0)).h_px(Px(300.0)))
            .test_id("ui-ai-image-demo-image")
            .into_element(cx)
    });

    let image = image.unwrap_or_else(|| {
        cx.text("Loading image...")
            .test_id("ui-ai-image-demo-loading")
    });

    let props = cx.with_theme(|theme| {
        let chrome = ChromeRefinement::default().p(Space::N4);
        let layout = LayoutRefinement::default().w_full().min_w_0().min_h_0();
        decl_style::container_props(theme, chrome, layout)
    });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Image (AI Elements): generated image presentation surface."),
            status_line,
            cx.container(props, move |cx| {
                vec![
                    ui::h_flex(move |_cx| vec![image])
                        .layout(LayoutRefinement::default().w_full())
                        .justify_center()
                        .items_center()
                        .into_element(cx),
                ]
            }),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .into_element(cx)
}
// endregion: example
