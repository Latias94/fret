use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_sources_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_markdown::OnLinkActivate;
    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Default)]
    struct DemoModels {
        selected: Option<Model<Option<Arc<str>>>>,
        last_opened_url: Option<Model<Option<Arc<str>>>>,
    }

    let needs_init = cx.with_state(DemoModels::default, |st| {
        st.selected.is_none() || st.last_opened_url.is_none()
    });
    if needs_init {
        let selected = cx.app.models_mut().insert(None::<Arc<str>>);
        let last_opened_url = cx.app.models_mut().insert(None::<Arc<str>>);
        cx.with_state(DemoModels::default, |st| {
            st.selected = Some(selected.clone());
            st.last_opened_url = Some(last_opened_url.clone());
        });
    }

    let (selected_model, last_opened_url_model) = cx.with_state(DemoModels::default, |st| {
        (
            st.selected.clone().expect("selected"),
            st.last_opened_url.clone().expect("last_opened_url"),
        )
    });

    let selected_now = cx
        .get_model_cloned(&selected_model, Invalidation::Paint)
        .flatten();
    let last_url = cx
        .get_model_cloned(&last_opened_url_model, Invalidation::Paint)
        .flatten();

    let marker_selected = cx
        .text(format!(
            "highlighted_source={}",
            selected_now.as_deref().unwrap_or("<none>")
        ))
        .test_id("ui-ai-sources-demo-highlighted");
    let marker_url = cx
        .text(format!(
            "last_opened_url={}",
            last_url.as_deref().unwrap_or("<none>")
        ))
        .test_id("ui-ai-sources-demo-last-url");

    let on_open_url: OnLinkActivate = Arc::new({
        let last_opened_url_model = last_opened_url_model.clone();
        move |host, acx, _reason, link| {
            let _ = host.models_mut().update(&last_opened_url_model, |v| {
                *v = Some(Arc::<str>::from(link.href));
            });
            host.request_redraw(acx.window);
        }
    });

    let sources: Arc<[ui_ai::SourceItem]> = Arc::from(vec![
        ui_ai::SourceItem::new("src-0", "Example source A")
            .url("https://example.com/a")
            .excerpt("A short excerpt used for truncation and wrapping tests."),
        ui_ai::SourceItem::new("src-1", "Example source B")
            .url("https://example.com/b")
            .excerpt("Another excerpt: this should wrap and remain readable."),
        ui_ai::SourceItem::new("src-2", "Example source C")
            .url("https://example.com/c")
            .excerpt("A third row to exercise list layout + highlighting behavior."),
    ]);

    let sources_for_citations = sources.clone();
    let sources_for_block = sources.clone();
    let selected_for_citations = selected_model.clone();
    let selected_for_block = selected_model.clone();

    let cite_label = cx.text("Select:");
    let c0 = ui_ai::InlineCitation::new("[1]")
        .sources(sources_for_citations.clone())
        .source_id("src-0")
        .select_source_model(selected_for_citations.clone())
        .test_id("ui-ai-sources-demo-c0")
        .into_element(cx);
    let c1 = ui_ai::InlineCitation::new("[2]")
        .sources(sources_for_citations.clone())
        .source_id("src-1")
        .select_source_model(selected_for_citations.clone())
        .test_id("ui-ai-sources-demo-c1")
        .into_element(cx);
    let c2 = ui_ai::InlineCitation::new("[3]")
        .sources(sources_for_citations)
        .source_id("src-2")
        .select_source_model(selected_for_citations)
        .test_id("ui-ai-sources-demo-c2")
        .into_element(cx);

    let citations = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2),
        move |_cx| vec![cite_label, c0, c1, c2],
    );

    let block = ui_ai::SourcesBlock::new(sources_for_block)
        .title("Used sources")
        .default_open(false)
        .on_open_url(on_open_url)
        .highlighted_source_model(selected_for_block)
        .test_id_root("ui-ai-sources-demo-root")
        .test_id_row_prefix("ui-ai-sources-demo-row-")
        .into_element(cx);

    let title =
        cx.text("SourcesBlock (AI Elements): collapsible list + highlight + open-url seam.");
    let hint = cx.text(
        "Upstream AI Elements defaults sources to collapsed; activate the trigger to expand.",
    );

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |_cx| vec![title, marker_selected, marker_url, citations, hint, block],
    )]
}
