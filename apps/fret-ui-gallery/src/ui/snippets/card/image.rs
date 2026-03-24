pub const SOURCE: &str = include_str!("image.rs");

// region: example
use super::demo_cover_image;
use fret::{UiChild, UiCx};
use fret_core::{Color as CoreColor, ImageId};
use fret_ui::Theme;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::{Arc, OnceLock};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app).snapshot();
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(384.0)))
        .min_w_0();

    let cover_bg = theme.color_token("muted");
    let cover_stack = {
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default().bg(ColorRef::Color(cover_bg)),
            LayoutRefinement::default().relative().size_full(),
        );

        cx.container(props, |cx| {
            static DEBUG_IMAGE_LOADING: OnceLock<bool> = OnceLock::new();
            let debug_image_loading = *DEBUG_IMAGE_LOADING.get_or_init(|| {
                std::env::var_os("FRET_UI_GALLERY_DEBUG_IMAGE_LOADING")
                    .is_some_and(|v| !v.is_empty())
            });

            let event_cover: Option<ImageId> = demo_cover_image(cx);
            let event_cover_source = "inline-rgba8";

            let image = shadcn::MediaImage::maybe(event_cover)
                .loading(true)
                .refine_layout(LayoutRefinement::default().size_full())
                .into_element(cx)
                .test_id("ui-gallery-card-image-event-cover-image");

            let overlay_props = decl_style::container_props(
                &theme,
                ChromeRefinement::default().bg(ColorRef::Color(CoreColor {
                    a: 0.35,
                    ..CoreColor::from_srgb_hex_rgb(0x00_00_00)
                })),
                LayoutRefinement::default()
                    .absolute()
                    .inset(Space::N0)
                    .size_full(),
            );

            let overlay = cx
                .container(overlay_props, |_cx| Vec::new())
                .test_id("ui-gallery-card-image-event-cover-overlay");

            let debug_overlay = if debug_image_loading {
                let text: Arc<str> = Arc::from(format!(
                    "event_cover: image={} source={event_cover_source}",
                    event_cover.is_some()
                ));
                Some(
                    shadcn::Badge::new(text)
                        .variant(shadcn::BadgeVariant::Secondary)
                        .refine_layout(
                            LayoutRefinement::default()
                                .absolute()
                                .left(Space::N2)
                                .bottom(Space::N2),
                        )
                        .into_element(cx)
                        .test_id("ui-gallery-card-image-event-cover-debug"),
                )
            } else {
                None
            };

            let mut out = vec![image, overlay];
            if let Some(debug_overlay) = debug_overlay {
                out.push(debug_overlay);
            }
            out
        })
        .test_id("ui-gallery-card-image-event-cover-stack")
    };

    let cover = shadcn::AspectRatio::new(16.0 / 9.0, cover_stack)
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
        .test_id("ui-gallery-card-image-event-cover");

    let featured = shadcn::Badge::new("Featured")
        .variant(shadcn::BadgeVariant::Secondary)
        .into_element(cx)
        .test_id("ui-gallery-card-image-featured");

    shadcn::card(|cx| {
        ui::children![
            cx;
            cover,
            shadcn::card_header(|cx| {
                ui::children![
                    cx;
                    shadcn::card_action(|cx| ui::children![cx; featured]),
                    shadcn::card_title("Design systems meetup"),
                    shadcn::card_description(
                        "A practical talk on component APIs, accessibility, and shipping faster. The cover image uses a self-contained demo buffer so the snippet stays copyable outside UI Gallery.",
                    ),
                ]
            }),
            shadcn::card_footer(|cx| {
                ui::children![
                    cx;
                    shadcn::Button::new("View Event")
                        .refine_layout(LayoutRefinement::default().w_full())
                        .ui()
                        .test_id("ui-gallery-card-image-view-event"),
                ]
            }),
        ]
    })
    .refine_style(ChromeRefinement::default().pt(Space::N0))
    .refine_layout(max_w_sm.relative())
    .into_element(cx)
    .test_id("ui-gallery-card-image")
}
// endregion: example
