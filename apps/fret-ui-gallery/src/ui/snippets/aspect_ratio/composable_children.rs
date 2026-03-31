pub const SOURCE: &str = include_str!("composable_children.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app);
    let muted_bg = theme.color_token("muted");
    let border = theme.color_token("border");
    let asset_image = super::images::landscape_image_state(cx);

    let image = shadcn::MediaImage::maybe(asset_image.image)
        .loading(asset_image.loading)
        .fit(fret_core::ViewportFit::Cover)
        .refine_style(ChromeRefinement::default().rounded(Radius::Lg))
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-composable-children-image");

    let badge = shadcn::Badge::new("Featured")
        .variant(shadcn::BadgeVariant::Secondary)
        .refine_layout(
            LayoutRefinement::default()
                .absolute()
                .left_px(Px(12.0))
                .bottom_px(Px(12.0)),
        )
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-composable-children-badge");

    let frame = shadcn::AspectRatio::with_children([image, badge])
        .ratio(16.0 / 9.0)
        .refine_style(
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .border_1()
                .bg(ColorRef::Color(muted_bg))
                .border_color(ColorRef::Color(border)),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(384.0)))
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-composable-children");

    ui::h_flex(move |_cx| vec![frame])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .justify_center()
        .into_element(cx)
}
// endregion: example
