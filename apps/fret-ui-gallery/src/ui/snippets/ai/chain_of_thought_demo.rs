pub const SOURCE: &str = include_str!("chain_of_thought_demo.rs");

// region: example
use crate::driver::UiGalleryImageSourceDemoAssets;
use fret_core::Px;
use fret_ui_ai as ui_ai;
use fret_ui_assets::ui::ImageSourceElementContextExt as _;
use fret_ui_kit::declarative::ElementContextThemeExt;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement};
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let Some(assets) = cx.app.global::<UiGalleryImageSourceDemoAssets>().cloned() else {
        return cx.text("Chain of Thought demo assets missing (expected UiGalleryDriver init).");
    };

    let image_state = cx.use_image_source_state(&assets.square_png);
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
        .into_element_with_children(cx, move |cx| {
            vec![
                ui_ai::ChainOfThoughtHeader::new()
                    .test_id("ui-ai-chain-of-thought-header")
                    .into_element(cx),
                ui_ai::ChainOfThoughtContent::new([
                    ui_ai::ChainOfThoughtStep::new("Searching for profiles for Hayden Bleasel")
                        .status(ui_ai::ChainOfThoughtStepStatus::Complete)
                        .icon(IconId::new("lucide.search"))
                        .children([ui_ai::ChainOfThoughtSearchResults::new([
                            ui_ai::ChainOfThoughtSearchResult::new("www.x.com")
                                .test_id("ui-ai-chain-of-thought-search-result-x")
                                .into_element(cx),
                            ui_ai::ChainOfThoughtSearchResult::new("www.instagram.com")
                                .into_element(cx),
                            ui_ai::ChainOfThoughtSearchResult::new("www.github.com")
                                .into_element(cx),
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
                            ui_ai::ChainOfThoughtSearchResult::new("www.github.com")
                                .into_element(cx),
                            ui_ai::ChainOfThoughtSearchResult::new("www.dribbble.com")
                                .into_element(cx),
                        ])
                        .test_id("ui-ai-chain-of-thought-search-results-2")
                        .into_element(cx)])
                        .into_element(cx),
                ])
                .test_id("ui-ai-chain-of-thought-content-marker")
                .into_element(cx),
            ]
        })
}
// endregion: example
