pub const SOURCE: &str = include_str!("inline_citation_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

fn centered_row<H: UiHost, F>(children: F) -> impl IntoUiElement<H> + use<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
{
    ui::h_flex(children)
        .gap(Space::N1)
        .wrap()
        .w_full()
        .max_w(Px(720.0))
        .min_w_0()
        .justify_center()
        .items_center()
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let sources: Arc<[ui_ai::SourceItem]> = Arc::from(vec![
        ui_ai::SourceItem::new("src-0", "Advances in Natural Language Processing")
            .url("https://example.com/nlp-advances")
            .description(
                "A comprehensive study on the recent developments in natural language processing technologies and their applications.",
            )
            .quote("The technology continues to evolve rapidly, with new breakthroughs being announced regularly."),
        ui_ai::SourceItem::new("src-1", "Breakthroughs in Machine Learning")
            .url("https://mlnews.org/breakthroughs")
            .description(
                "An overview of the most significant machine learning breakthroughs in the past year.",
            ),
        ui_ai::SourceItem::new("src-2", "AI in Healthcare: Current Trends")
            .url("https://healthai.com/trends")
            .description(
                "A report on how artificial intelligence is transforming healthcare and diagnostics.",
            ),
        ui_ai::SourceItem::new("src-3", "Ethics of Artificial Intelligence")
            .url("https://aiethics.org/overview")
            .description(
                "A discussion on the ethical considerations and challenges in the development of AI.",
            ),
        ui_ai::SourceItem::new("src-4", "Scaling Deep Learning Models")
            .url("https://deeplearninghub.com/scaling-models")
            .description(
                "Insights into the technical challenges and solutions for scaling deep learning architectures.",
            ),
        ui_ai::SourceItem::new("src-5", "Natural Language Understanding Benchmarks")
            .url("https://nlubenchmarks.com/latest")
            .description(
                "A summary of the latest benchmarks and evaluation metrics for natural language understanding systems.",
            ),
    ]);

    let citation_ids: Arc<[Arc<str>]> = Arc::from(vec![
        Arc::<str>::from("src-0"),
        Arc::<str>::from("src-1"),
        Arc::<str>::from("src-2"),
        Arc::<str>::from("src-3"),
        Arc::<str>::from("src-4"),
        Arc::<str>::from("src-5"),
    ]);

    let citation_text = ui_ai::InlineCitationText::new([cx.text(
        "The technology continues to evolve rapidly, with new breakthroughs being announced regularly",
    )]);
    let citation = ui_ai::InlineCitationRoot::new()
        .sources(sources)
        .source_ids(citation_ids)
        .test_id("ui-ai-inline-citation-demo-citation")
        .refine_layout(LayoutRefinement::default().min_w_0())
        .into_element_parts(citation_text, ui_ai::InlineCitationCard::new(), cx);

    centered_row(|cx| {
        vec![
            cx.text(
                "According to recent studies, artificial intelligence has shown remarkable progress in natural language processing.",
            ),
            citation,
            cx.text("."),
        ]
    })
    .into_element(cx)
    .test_id("ui-ai-inline-citation-demo-root")
}
// endregion: example
