pub const SOURCE: &str = include_str!("image.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Color as CoreColor, ImageColorSpace, ImageId};
use fret_ui::Theme;
use fret_ui_assets::ImageSource;
use fret_ui_assets::ui::ImageSourceElementContextExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::{Arc, OnceLock};

fn demo_cover_image(cx: &mut UiCx<'_>) -> Option<ImageId> {
    let source = ImageSource::rgba8(
        4,
        4,
        vec![
            244, 181, 99, 255, 229, 115, 58, 255, 71, 149, 212, 255, 19, 78, 117, 255, 237, 149,
            74, 255, 206, 86, 52, 255, 79, 172, 167, 255, 23, 103, 124, 255, 221, 91, 47, 255, 196,
            69, 54, 255, 98, 188, 153, 255, 42, 124, 108, 255, 181, 68, 50, 255, 151, 56, 58, 255,
            95, 161, 118, 255, 54, 103, 86, 255,
        ],
        ImageColorSpace::Srgb,
    );
    cx.use_image_source_state(&source).image
}

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
            let event_cover_source = "rgba8";

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
                        "A practical talk on component APIs, accessibility, and shipping faster. The cover image uses a self-contained RGBA source so the snippet stays copyable outside UI Gallery.",
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
