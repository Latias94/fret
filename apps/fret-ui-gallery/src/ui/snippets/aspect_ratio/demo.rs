pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Theme;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn render_frame<H: UiHost, E>(image: E) -> impl IntoUiElement<H> + use<H, E>
where
    E: IntoUiElement<H>,
{
    ui::h_flex(move |cx| {
        let theme = Theme::global(&*cx.app);
        let muted_bg = theme.color_token("muted");
        let frame = shadcn::AspectRatio::with_child(image.into_element(cx))
            .ratio(16.0 / 9.0)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .bg(ColorRef::Color(muted_bg)),
            )
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
            .into_element(cx)
            .test_id("ui-gallery-aspect-ratio-demo");

        [frame]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .justify_center()
}

#[allow(dead_code)]
pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let asset_image = super::images::landscape_image_state(cx);
    let image = shadcn::MediaImage::maybe(asset_image.image)
        .loading(asset_image.loading)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-demo-content");

    render_frame(image).into_element(cx)
}
// endregion: example

use fret::component::prelude::Model;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::primitives::visually_hidden::visually_hidden_label;

fn preview_status_marker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    image_id: Option<fret_core::ImageId>,
    loading: bool,
) -> AnyElement {
    let (test_id, label) = if image_id.is_some() {
        (
            "ui-gallery-aspect-ratio-demo-image-status-loaded",
            "Aspect Ratio demo image loaded",
        )
    } else if loading {
        (
            "ui-gallery-aspect-ratio-demo-image-status-loading",
            "Aspect Ratio demo image loading",
        )
    } else {
        (
            "ui-gallery-aspect-ratio-demo-image-status-missing",
            "Aspect Ratio demo image unavailable",
        )
    };

    visually_hidden_label(cx, label).test_id(test_id)
}

pub fn render_preview<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    demo_image: Option<Model<Option<fret_core::ImageId>>>,
) -> impl IntoUiElement<H> + use<H> {
    let model_image_id = demo_image
        .as_ref()
        .and_then(|model| cx.watch_model(model).layout().cloned().flatten());
    let asset_image = super::images::landscape_image_state(cx);
    let image_id = model_image_id.or(asset_image.image);
    let loading = model_image_id.is_none() && asset_image.loading;
    let status_marker = preview_status_marker(cx, image_id, loading);

    let image = shadcn::MediaImage::maybe(image_id)
        .loading(loading)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-demo-content");

    ui::v_flex(move |cx| vec![status_marker, render_frame(image).into_element(cx)])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx)
}
