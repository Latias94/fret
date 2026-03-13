pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app);
    let muted_bg = theme.color_token("muted");
    let muted_fg = theme.color_token("muted-foreground");

    let label = ui::text("16:9")
        .text_sm()
        .font_semibold()
        .text_color(ColorRef::Color(muted_fg))
        .into_element(cx);

    let content = ui::v_flex(move |_cx| vec![label])
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_center()
        .justify_center()
        .into_element(cx)
        .test_id("ui-gallery-aspect-ratio-usage-content");

    let frame = shadcn::AspectRatio::with_child(content)
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
