pub const SOURCE: &str = include_str!("chain_of_thought_demo.rs");

// region: example
use fret_core::Px;
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{self as shadcn, prelude::*};

#[derive(Default)]
struct DemoModels {
    open: Option<Model<bool>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = cx.with_state(DemoModels::default, |st| st.open.clone());
    let open = match open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DemoModels::default, |st| st.open = Some(model.clone()));
            model
        }
    };

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);

    let shell = ui_ai::ChainOfThought::new()
        .open_model(open)
        .test_id_root("ui-ai-chain-of-thought-root")
        .into_element_with_children(cx, move |cx| {
            let mut children = vec![
                ui_ai::ChainOfThoughtHeader::new()
                    .test_id("ui-ai-chain-of-thought-header")
                    .into_element(cx),
            ];

            if is_open {
                children.push(cx.text("").test_id("ui-ai-chain-of-thought-open-true"));
            }

            children.push(
                ui_ai::ChainOfThoughtContent::new([
                    ui_ai::ChainOfThoughtStep::new("Searching for profiles for Hayden Bleasel")
                        .status(ui_ai::ChainOfThoughtStepStatus::Complete)
                        .icon(IconId::new("lucide.search"))
                        .children([ui_ai::ChainOfThoughtSearchResults::new([
                            ui_ai::ChainOfThoughtSearchResult::new("x.com")
                                .test_id("ui-ai-chain-of-thought-search-result-x")
                                .into_element(cx),
                            ui_ai::ChainOfThoughtSearchResult::new("instagram.com").into_element(cx),
                            ui_ai::ChainOfThoughtSearchResult::new("github.com").into_element(cx),
                        ])
                        .test_id("ui-ai-chain-of-thought-search-results-1")
                        .into_element(cx)])
                        .into_element(cx),
                    ui_ai::ChainOfThoughtStep::new("Found the profile photo for Hayden Bleasel")
                        .status(ui_ai::ChainOfThoughtStepStatus::Active)
                        .icon(IconId::new("lucide.image"))
                        .children([ui_ai::ChainOfThoughtImage::new([shadcn::Skeleton::new()
                            .refine_style(ChromeRefinement::default().border_1())
                            .refine_layout(LayoutRefinement::default().w_px(Px(150.0)).h_px(Px(150.0)))
                            .into_element(cx)])
                        .caption("Hayden Bleasel's profile photo from x.com, showing a Ghibli-style man.")
                        .test_id("ui-ai-chain-of-thought-image-1")
                        .into_element(cx)])
                        .into_element(cx),
                    ui_ai::ChainOfThoughtStep::new("Hayden Bleasel is an Australian product designer, software engineer, and founder. He is currently based in the United States working for Vercel, an American cloud application company.")
                        .status(ui_ai::ChainOfThoughtStepStatus::Complete)
                        .into_element(cx),
                    ui_ai::ChainOfThoughtStep::new("Searching for recent work...")
                        .status(ui_ai::ChainOfThoughtStepStatus::Pending)
                        .icon(IconId::new("lucide.search"))
                        .children([ui_ai::ChainOfThoughtSearchResults::new([
                            ui_ai::ChainOfThoughtSearchResult::new("github.com").into_element(cx),
                            ui_ai::ChainOfThoughtSearchResult::new("dribbble.com").into_element(cx),
                        ])
                        .test_id("ui-ai-chain-of-thought-search-results-2")
                        .into_element(cx)])
                        .into_element(cx),
                ])
                .test_id("ui-ai-chain-of-thought-content-marker")
                .into_element(cx),
            );

            children
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Chain of Thought (AI Elements)"),
            cx.text("Click the header to toggle the disclosure."),
            shell,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
