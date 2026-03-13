pub const SOURCE: &str = include_str!("square.rs");

// region: example
use fret_ui::Theme;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn square_image<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    demo_image: Option<Model<Option<fret_core::ImageId>>>,
    content_test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let image = if let Some(image) = demo_image {
        shadcn::MediaImage::model(image)
    } else {
        shadcn::MediaImage::maybe(None)
    };

    image
        .loading(true)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id(content_test_id)
}

fn ratio_example<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    ratio: f32,
    max_w: Px,
    test_id: &'static str,
    content_test_id: &'static str,
    demo_image: Option<Model<Option<fret_core::ImageId>>>,
) -> impl IntoUiElement<H> + use<H> {
    let theme = Theme::global(&*cx.app);
    let muted_bg = theme.color_token("muted");
    let border = theme.color_token("border");
    let content = square_image(cx, demo_image, content_test_id).into_element(cx);

    let frame = shadcn::AspectRatio::with_child(content)
        .ratio(ratio)
        .refine_style(
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .border_1()
                .bg(ColorRef::Color(muted_bg))
                .border_color(ColorRef::Color(border)),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(max_w))
        .into_element(cx)
        .test_id(test_id);

    ui::h_flex(move |_cx| vec![frame])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_center()
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    ratio_example(
        cx,
        1.0,
        Px(192.0),
        "ui-gallery-aspect-ratio-square",
        "ui-gallery-aspect-ratio-square-content",
        None,
    )
    .into_element(cx)
}
// endregion: example

pub fn render_preview<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    demo_image: Option<Model<Option<fret_core::ImageId>>>,
) -> impl IntoUiElement<H> + use<H> {
    let asset_image = super::images::portrait_image_id(cx);

    let image = if let Some(image_id) = asset_image {
        shadcn::MediaImage::maybe(Some(image_id))
    } else if let Some(demo_image) = demo_image {
        shadcn::MediaImage::model(demo_image)
    } else {
        return render(cx);
    };

    let image = image
        .loading(true)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-square-content");

    let theme = Theme::global(&*cx.app);
    let muted_bg = theme.color_token("muted");
    let border = theme.color_token("border");

    let frame = shadcn::AspectRatio::with_child(image)
        .refine_style(
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .border_1()
                .bg(ColorRef::Color(muted_bg))
                .border_color(ColorRef::Color(border)),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(192.0)))
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-square");

    ui::h_flex(move |_cx| vec![frame])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_center()
        .into_element(cx)
}
