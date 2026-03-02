pub const SOURCE: &str = include_str!("image.rs");

// region: example
use fret_app::App;
use fret_core::{Color as CoreColor, ImageId};
use fret_ui::Theme;
use fret_ui_assets::ui::ImageSourceElementContextExt as _;
use fret_ui_kit::declarative::{ModelWatchExt as _, style as decl_style};
use fret_ui_kit::ui;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::{Arc, OnceLock};

fn icon(cx: &mut ElementContext<'_, App>, id: &'static str) -> AnyElement {
    shadcn::icon::icon(cx, fret_icons::IconId::new_static(id))
}

pub fn render(
    cx: &mut ElementContext<'_, App>,
    event_cover_image: Model<Option<ImageId>>,
) -> AnyElement {
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

            let event_cover_fallback = cx.watch_model(&event_cover_image).copied().flatten();

            #[cfg(not(target_arch = "wasm32"))]
            let (event_cover, event_cover_state, event_cover_path_exists) = {
                static EVENT_COVER_TEST_JPG: OnceLock<Option<fret_ui_assets::ImageSource>> =
                    OnceLock::new();
                let source = EVENT_COVER_TEST_JPG.get_or_init(|| {
                    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../assets/textures/test.jpg");
                    if path.exists() {
                        Some(fret_ui_assets::ImageSource::from_path(Arc::new(path)))
                    } else {
                        None
                    }
                });
                let (state, image) = source.as_ref().map_or((None, None), |source| {
                    let state = cx.use_image_source_state(source);
                    let image = state.image;
                    (Some(state), image)
                });
                let path_exists = source.is_some();

                (image.or(event_cover_fallback), state, path_exists)
            };

            #[cfg(target_arch = "wasm32")]
            let (event_cover, event_cover_state, event_cover_path_exists) = {
                static EVENT_COVER_TEST_JPG: OnceLock<fret_ui_assets::ImageSource> =
                    OnceLock::new();
                let source = EVENT_COVER_TEST_JPG
                    .get_or_init(|| fret_ui_assets::ImageSource::from_url(Arc::<str>::from("textures/test.jpg")));
                let state = cx.use_image_source_state(source);
                let image = state.image;
                (image.or(event_cover_fallback), Some(state), true)
            };

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
                    "event_cover: status={status} image={has_image} intrinsic={intrinsic} path_exists={event_cover_path_exists} err={error}"
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

    let badge = |cx: &mut ElementContext<'_, App>, icon_id: &'static str, text: &'static str| {
        shadcn::Badge::new("")
            .variant(shadcn::BadgeVariant::Outline)
            .children([
                icon(cx, icon_id),
                ui::text(cx, text)
                    .nowrap()
                    .into_element(cx)
                    .test_id(format!("ui-gallery-card-image-badge-{text}")),
            ])
            .into_element(cx)
    };

    let footer = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .items_center()
            .justify_between(),
        |cx| {
            let badges = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                |cx| {
                    vec![
                        badge(cx, "lucide.bed", "4"),
                        badge(cx, "lucide.bath", "2"),
                        badge(cx, "lucide.land-plot", "350m²"),
                    ]
                },
            );
            let price = ui::text(cx, "$135,000")
                .font_medium()
                .into_element(cx)
                .test_id("ui-gallery-card-image-price");

            vec![badges, price]
        },
    )
    .test_id("ui-gallery-card-image-footer");

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Is this an image?").into_element(cx),
            shadcn::CardDescription::new("This is a card with an image.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![cover])
            .refine_style(ChromeRefinement::default().px(Space::N0))
            .into_element(cx),
        shadcn::CardFooter::new(vec![footer]).into_element(cx),
    ])
    .refine_layout(max_w_sm.relative())
    .into_element(cx)
    .test_id("ui-gallery-card-image")
}
// endregion: example
