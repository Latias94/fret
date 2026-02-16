use super::super::super::super::*;

use ui_assets::ui::ImageSourceElementContextExt as _;

pub(in crate::ui) fn preview_ai_image_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::declarative::style as decl_style;
    use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

    let Some(assets) = cx.app.global::<UiGalleryImageSourceDemoAssets>().cloned() else {
        return vec![cx.text("Image demo assets missing (expected UiGalleryDriver init).")];
    };

    let state = cx.use_image_source_state(&assets.square_png);
    let status_line = cx
        .text(format!("status={:?}", state.status))
        .test_id("ui-ai-image-demo-status");

    let image = state.image.map(|id| {
        ui_ai::Image::new(id)
            .alt("Generated image")
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_color(ColorRef::Color(theme.color_token("border"))),
            )
            .refine_layout(
                LayoutRefinement::default()
                    .h_px(Px(300.0))
                    .aspect_ratio(1.0),
            )
            .test_id("ui-ai-image-demo-image")
            .into_element(cx)
    });

    let image = image.unwrap_or_else(|| {
        cx.text("Loading image...")
            .test_id("ui-ai-image-demo-loading")
    });

    let props = decl_style::container_props(
        theme,
        ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(theme.color_token("border")))
            .p(Space::N4),
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .min_h_0()
            .h_px(Px(420.0)),
    );

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3),
        move |cx| {
            vec![
                cx.text("Image (AI Elements): generated image presentation surface."),
                status_line,
                cx.container(props, move |cx| {
                    vec![stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .justify_center()
                            .items_center(),
                        move |_cx| vec![image],
                    )]
                }),
            ]
        },
    )]
}
