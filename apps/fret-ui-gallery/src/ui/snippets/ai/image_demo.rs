pub const SOURCE: &str = include_str!("image_demo.rs");

// region: example
use super::shared_preview_image_id;
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::prelude::*;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let image_id = shared_preview_image_id(cx);
    let status_line = cx
        .text(format!("image_ready={}", image_id.is_some()))
        .test_id("ui-ai-image-demo-status");

    let border = cx.with_theme(|theme| theme.color_token("border"));
    let image = image_id.map(|id| {
        ui_ai::Image::new(id)
            .alt("Example self-contained demo image")
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
            cx.text("Image (AI Elements): self-contained demo image presentation surface."),
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
