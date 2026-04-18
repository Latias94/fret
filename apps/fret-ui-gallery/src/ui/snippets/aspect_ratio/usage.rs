pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app);
    let muted_bg = theme.color_token("muted");
    let asset_image = super::images::landscape_image_state(cx);
    let image = shadcn::MediaImage::maybe(asset_image.image)
        .loading(asset_image.loading)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-usage-content");

    let frame = shadcn::AspectRatio::with_child(image)
        .ratio(16.0 / 9.0)
        .refine_style(
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .bg(ColorRef::Color(muted_bg)),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-usage");

    ui::h_flex(move |_cx| vec![frame])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_center()
        .into_element(cx)
}
// endregion: example
