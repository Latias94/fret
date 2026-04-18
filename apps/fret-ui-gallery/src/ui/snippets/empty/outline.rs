pub const SOURCE: &str = include_str!("outline.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::ui;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let muted_foreground = cx.with_theme(|theme| theme.color_token("muted-foreground"));
    let icon = icon::icon(cx, fret_icons::IconId::new_static("lucide.cloud"));

    shadcn::empty(|cx| {
        ui::children![
            cx;
            shadcn::empty_header(|cx| {
                ui::children![
                    cx;
                    shadcn::empty_media(|cx| ui::children![cx; icon])
                        .variant(shadcn::EmptyMediaVariant::Icon),
                    shadcn::empty_title("Cloud Storage Empty")
                        .test_id("ui-gallery-empty-outline-title"),
                    shadcn::empty_description(
                        "Upload files to cloud storage to access them from any device.",
                    ),
                ]
            })
            .test_id("ui-gallery-empty-outline-header"),
            shadcn::empty_content(|cx| {
                ui::children![
                    cx;
                    shadcn::Button::new("Upload Files")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm),
                ]
            }),
        ]
    })
    .refine_style(ChromeRefinement::default().border_color(ColorRef::Color(muted_foreground)))
    .refine_layout(LayoutRefinement::default().w_full().min_h(Px(280.0)))
    .into_element(cx)
    .test_id("ui-gallery-empty-outline")
}
// endregion: example
