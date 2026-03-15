pub const SOURCE: &str = include_str!("demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::Theme;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn render_frame<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    image: AnyElement,
) -> impl IntoUiElement<H> + use<H> {
    let theme = Theme::global(&*cx.app);
    let muted_bg = theme.color_token("muted");

    let frame = shadcn::AspectRatio::with_child(image)
        .ratio(16.0 / 9.0)
        .refine_style(
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .bg(ColorRef::Color(muted_bg)),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-demo");

    ui::h_flex(move |_cx| vec![frame])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_center()
}

// Kept as the copyable app-facing snippet surface; the gallery preview uses `render_preview(...)`
// so it can swap in asset-backed media when available.
#[allow(dead_code)]
pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let image = shadcn::MediaImage::maybe(None)
        .loading(true)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-demo-content");

    render_frame(cx, image).into_element(cx)
}
// endregion: example

pub fn render_preview<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    demo_image: Option<Model<Option<fret_core::ImageId>>>,
) -> impl IntoUiElement<H> + use<H> {
    let image = if let Some(image_id) = super::images::landscape_image_id(cx) {
        shadcn::MediaImage::maybe(Some(image_id))
    } else if let Some(demo_image) = demo_image {
        shadcn::MediaImage::model(demo_image)
    } else {
        let image = shadcn::MediaImage::maybe(None)
            .loading(true)
            .fit(fret_core::ViewportFit::Cover)
            .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
            .refine_layout(LayoutRefinement::default().w_full().h_full())
            .into_element(cx)
            .test_id("ui-gallery-aspect-ratio-demo-content");
        return render_frame(cx, image).into_element(cx);
    };

    let image = image
        .loading(true)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-demo-content");

    render_frame(cx, image).into_element(cx)
}
