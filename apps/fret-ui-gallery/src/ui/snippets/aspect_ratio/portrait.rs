pub const SOURCE: &str = include_str!("portrait.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn portrait_image<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    demo_image: Option<Model<Option<fret_core::ImageId>>>,
    content_test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let model_image_id = demo_image
        .as_ref()
        .and_then(|model| cx.watch_model(model).layout().cloned().flatten());
    let asset_image = super::images::portrait_image_state(cx);
    let image_id = model_image_id.or(asset_image.image);
    let loading = model_image_id.is_none() && asset_image.loading;

    shadcn::MediaImage::maybe(image_id)
        .loading(loading)
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

    let frame = shadcn::AspectRatio::with_child(
        portrait_image(cx, demo_image, content_test_id).into_element(cx),
    )
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

#[allow(dead_code)]
pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ratio_example(
        cx,
        9.0 / 16.0,
        Px(160.0),
        "ui-gallery-aspect-ratio-portrait",
        "ui-gallery-aspect-ratio-portrait-content",
        None,
    )
    .into_element(cx)
}
// endregion: example

use fret::component::prelude::Model;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;

pub fn render_preview<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    demo_image: Option<Model<Option<fret_core::ImageId>>>,
) -> impl IntoUiElement<H> + use<H> {
    ratio_example(
        cx,
        9.0 / 16.0,
        Px(160.0),
        "ui-gallery-aspect-ratio-portrait",
        "ui-gallery-aspect-ratio-portrait-content",
        demo_image,
    )
    .into_element(cx)
}
