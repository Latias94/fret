pub const SOURCE: &str = include_str!("outline.rs");

// region: example
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let muted_foreground = cx.with_theme(|theme| theme.color_token("muted-foreground"));
    let icon = fret_ui_shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.cloud"));

    shadcn::Empty::new([
        fret_ui_shadcn::empty::EmptyHeader::new([
            fret_ui_shadcn::empty::EmptyMedia::new([icon])
                .variant(fret_ui_shadcn::empty::EmptyMediaVariant::Icon)
                .into_element(cx),
            fret_ui_shadcn::empty::EmptyTitle::new("Cloud Storage Empty")
                .into_element(cx)
                .test_id("ui-gallery-empty-outline-title"),
            fret_ui_shadcn::empty::EmptyDescription::new(
                "Upload files to cloud storage to access them from any device.",
            )
            .into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-empty-outline-header"),
        fret_ui_shadcn::empty::EmptyContent::new([shadcn::Button::new("Upload Files")
            .variant(shadcn::ButtonVariant::Outline)
            .size(shadcn::ButtonSize::Sm)
            .into_element(cx)])
        .into_element(cx),
    ])
    .refine_style(ChromeRefinement::default().border_color(ColorRef::Color(muted_foreground)))
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
    .into_element(cx)
    .test_id("ui-gallery-empty-outline")
}
// endregion: example
