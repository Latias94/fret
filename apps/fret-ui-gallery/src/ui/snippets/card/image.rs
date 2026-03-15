pub const SOURCE: &str = include_str!("image.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Color as CoreColor, ImageColorSpace};
use fret_ui::Theme;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::{Arc, OnceLock};

fn demo_event_cover_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        // Keep the snippet self-contained instead of depending on repo-relative demo assets.
        ImageSource::rgba8(
            320,
            180,
            demo_event_cover_rgba8(320, 180),
            ImageColorSpace::Srgb,
        )
    })
}

fn demo_event_cover_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;

            let mut r = (18.0 + 72.0 * fx) as u8;
            let mut g = (24.0 + 124.0 * (1.0 - fy)) as u8;
            let mut b = (44.0 + 156.0 * fy) as u8;

            let border = x < 3 || y < 3 || x + 3 >= width || y + 3 >= height;
            let banner = y > height / 5 && y < (height * 2) / 5;
            let focus =
                x > width / 6 && x < (width * 3) / 5 && y > height / 4 && y < (height * 4) / 5;

            if border {
                r = 245;
                g = 245;
                b = 245;
            } else if banner {
                r = r.saturating_add(14);
                g = g.saturating_add(14);
                b = b.saturating_add(10);
            } else if focus {
                r = r.saturating_add(24);
                g = g.saturating_add(24);
                b = b.saturating_add(24);
            }

            out[idx] = r;
            out[idx + 1] = g;
            out[idx + 2] = b;
            out[idx + 3] = 255;
        }
    }

    out
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

            let event_cover_state = cx.use_image_source_state(demo_event_cover_source());
            let event_cover = event_cover_state.image;
            let event_cover_source_available = true;
            let event_cover_state = Some(event_cover_state);

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

            let debug_overlay = if debug_image_loading
                || event_cover_state.as_ref().and_then(|s| s.error.as_deref()).is_some()
            {
                let status = event_cover_state
                    .as_ref()
                    .map(|s| format!("{:?}", s.status))
                    .unwrap_or_else(|| "<no-state>".to_string());
                let intrinsic = event_cover_state
                    .as_ref()
                    .and_then(|s| s.intrinsic_size_px)
                    .map(|(w, h)| format!("{w}x{h}"))
                    .unwrap_or_else(|| "-".to_string());
                let has_image = event_cover_state
                    .as_ref()
                    .map(|s| s.image.is_some())
                    .unwrap_or(false);
                let error = event_cover_state
                    .as_ref()
                    .and_then(|s| s.error.as_deref())
                    .unwrap_or("-");

                let text: Arc<str> = Arc::from(format!(
                    "event_cover: status={status} image={has_image} intrinsic={intrinsic} source_available={event_cover_source_available} err={error}"
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
                        "A practical talk on component APIs, accessibility, and shipping faster.",
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
