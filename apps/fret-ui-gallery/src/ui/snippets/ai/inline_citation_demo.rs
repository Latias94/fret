pub const SOURCE: &str = include_str!("inline_citation_demo.rs");

// region: example
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    selected: Option<Model<Option<Arc<str>>>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let needs_init = cx.with_state(DemoModels::default, |st| st.selected.is_none());
    if needs_init {
        let model = cx.app.models_mut().insert(None::<Arc<str>>);
        cx.with_state(DemoModels::default, |st| st.selected = Some(model.clone()));
    }

    let selected_model = cx
        .with_state(DemoModels::default, |st| st.selected.clone())
        .expect("selected");
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
        ui_ai::SourceItem::new("src-0", "Example source A")
            .url("https://example.com/a")
            .excerpt("A short excerpt used for truncation and wrapping tests."),
        ui_ai::SourceItem::new("src-1", "Example source B")
            .url("https://example.com/b")
            .excerpt("Another excerpt: this should wrap and remain readable."),
    ]);

    let c0 = ui_ai::InlineCitation::new("[1]")
        .sources(sources.clone())
        .source_id("src-0")
        .select_source_model(selected_model.clone())
        .test_id("ui-ai-inline-citation-demo-c0")
        .into_element(cx);

    let c1_ids: Arc<[Arc<str>]> =
        Arc::from(vec![Arc::<str>::from("src-0"), Arc::<str>::from("src-1")]);
    let c1 = ui_ai::InlineCitation::new("[2]")
        .sources(sources)
        .source_ids(c1_ids)
        .select_source_model(selected_model)
        .test_id("ui-ai-inline-citation-demo-c1")
        .into_element(cx);

    let title = cx.text("InlineCitation (AI Elements): inline label + hover card + pager.");
    let hint = cx.text("Hover the badge to preview sources; activation emits a selected source id. (Upstream composes InlineCitationText + HoverCard + Carousel header.)");
    let row_label = cx.text("Citations:");

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2),
        move |_cx| vec![row_label, c0, c1],
    );

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |_cx| vec![title, marker, row, hint],
    )
}
// endregion: example
