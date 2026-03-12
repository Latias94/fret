pub const SOURCE: &str = include_str!("inline_citation_demo.rs");

// region: example
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let selected_model = cx.local_model_keyed("selected", || None::<Arc<str>>);
    let selected_now = cx
        .get_model_cloned(&selected_model, Invalidation::Paint)
        .flatten();

    let marker = cx
        .text(format!(
            "selected_source={}",
            selected_now.as_deref().unwrap_or("<none>")
        ))
        .test_id("ui-ai-inline-citation-demo-selected");

    let sources: Arc<[ui_ai::SourceItem]> = Arc::from(vec![
        ui_ai::SourceItem::new("src-0", "Advances in Natural Language Processing")
            .url("https://example.com/nlp-advances")
            .description(
                "A comprehensive study on recent developments in natural language processing technologies and their applications.",
            )
            .quote(
                "The technology continues to evolve rapidly, with new breakthroughs being announced regularly.",
            ),
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
    ]);

    let citation_text = cx.text(
        "The technology continues to evolve rapidly, with new breakthroughs being announced regularly",
    );
    let citation_ids: Arc<[Arc<str>]> = Arc::from(vec![
        Arc::<str>::from("src-0"),
        Arc::<str>::from("src-1"),
        Arc::<str>::from("src-2"),
    ]);
    let citation = ui_ai::InlineCitation::with_children([citation_text])
        .sources(sources)
        .source_ids(citation_ids)
        .select_source_model(selected_model)
        .test_id("ui-ai-inline-citation-demo-citation")
        .into_element(cx);

    let intro = cx.text(
        "According to recent studies, artificial intelligence has shown remarkable progress in natural language processing.",
    );
    let hint = cx.text(
        "Hover the hostname badge to preview sources and page through them. Fret also exposes `select_source_model(...)` so apps can sync the citation with a nearby Sources block.",
    );

    ui::v_flex(move |_cx| vec![intro, citation, marker, hint])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N4)
        .into_element(cx)
}
// endregion: example
