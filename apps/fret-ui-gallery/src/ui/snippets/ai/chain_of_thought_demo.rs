pub const SOURCE: &str = include_str!("chain_of_thought_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{ImageColorSpace, Px};
use fret_ui_ai as ui_ai;
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::OnceLock;

fn demo_image_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        // Keep the snippet self-contained instead of depending on UI Gallery driver globals.
        ImageSource::rgba8(96, 96, demo_avatar_rgba8(96, 96), ImageColorSpace::Srgb)
    })
}

fn demo_avatar_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;

            let mut r = (26.0 + 92.0 * fx) as u8;
            let mut g = (34.0 + 70.0 * (1.0 - fy)) as u8;
            let mut b = (92.0 + 120.0 * fy) as u8;

            let top_band = y < height / 4;
            let left_eye =
                y > height / 3 && y < height / 2 && x > width / 3 - 5 && x < width / 3 + 2;
            let right_eye = y > height / 3
                && y < height / 2
                && x > (width * 2) / 3 - 2
                && x < (width * 2) / 3 + 5;
            let outline = x < 2 || y < 2 || x + 2 >= width || y + 2 >= height;

            if top_band {
                r = 240;
                g = 192;
                b = 72;
            }
            if left_eye || right_eye {
                r = 18;
                g = 18;
                b = 24;
            }
            if outline {
                r = r.saturating_add(10);
                g = g.saturating_add(10);
                b = b.saturating_add(10);
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
    let image_state = cx.use_image_source_state(demo_image_source());
    let border = cx.with_theme(|theme| theme.color_token("border"));

    let image = image_state
        .image
        .map(|id| {
            ui_ai::Image::new(id)
                .alt("Example generated image")
                .refine_style(
                    ChromeRefinement::default()
                        .border_1()
                        .border_color(ColorRef::Color(border)),
                )
                .refine_layout(LayoutRefinement::default().w_px(Px(150.0)).h_px(Px(150.0)))
                .into_element(cx)
        })
        .unwrap_or_else(|| {
            shadcn::Skeleton::new()
                .refine_style(ChromeRefinement::default().border_1())
                .refine_layout(LayoutRefinement::default().w_px(Px(150.0)).h_px(Px(150.0)))
                .into_element(cx)
        });

    ui_ai::ChainOfThought::new()
        .default_open(true)
        .test_id_root("ui-ai-chain-of-thought-root")
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .header(
            ui_ai::ChainOfThoughtHeader::new().test_id("ui-ai-chain-of-thought-header"),
        )
        .content(ui_ai::ChainOfThoughtContent::new([
                ui_ai::ChainOfThoughtStep::new("Searching for profiles for Hayden Bleasel")
                    .status(ui_ai::ChainOfThoughtStepStatus::Complete)
                    .icon(IconId::new("lucide.search"))
                    .children([ui_ai::ChainOfThoughtSearchResults::new([
                        ui_ai::ChainOfThoughtSearchResult::new("www.x.com")
                            .test_id("ui-ai-chain-of-thought-search-result-x")
                            .into_element(cx),
                        ui_ai::ChainOfThoughtSearchResult::new("www.instagram.com").into_element(cx),
                        ui_ai::ChainOfThoughtSearchResult::new("www.github.com").into_element(cx),
                    ])
                    .test_id("ui-ai-chain-of-thought-search-results-1")
                    .into_element(cx)])
                    .into_element(cx),
                ui_ai::ChainOfThoughtStep::new("Found the profile photo for Hayden Bleasel")
                    .status(ui_ai::ChainOfThoughtStepStatus::Complete)
                    .icon(IconId::new("lucide.image"))
                    .children([
                        ui_ai::ChainOfThoughtImage::new([image])
                            .caption(
                                "Hayden Bleasel's profile photo from x.com, showing a Ghibli-style man.",
                            )
                            .test_id("ui-ai-chain-of-thought-image-1")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                ui_ai::ChainOfThoughtStep::new("Hayden Bleasel is an Australian product designer, software engineer, and founder. He is currently based in the United States working for Vercel, an American cloud application company.")
                    .status(ui_ai::ChainOfThoughtStepStatus::Complete)
                    .into_element(cx),
                ui_ai::ChainOfThoughtStep::new("Searching for recent work...")
                    .status(ui_ai::ChainOfThoughtStepStatus::Active)
                    .icon(IconId::new("lucide.search"))
                    .children([ui_ai::ChainOfThoughtSearchResults::new([
                        ui_ai::ChainOfThoughtSearchResult::new("www.github.com").into_element(cx),
                        ui_ai::ChainOfThoughtSearchResult::new("www.dribbble.com").into_element(cx),
                    ])
                    .test_id("ui-ai-chain-of-thought-search-results-2")
                    .into_element(cx)])
                    .into_element(cx),
            ])
            .test_id("ui-ai-chain-of-thought-content-marker"))
        .into_element(cx)
}
// endregion: example
