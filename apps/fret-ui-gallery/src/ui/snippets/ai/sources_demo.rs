pub const SOURCE: &str = include_str!("sources_demo.rs");

// region: example
use fret_markdown::OnLinkActivate;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let on_open_url: OnLinkActivate = Arc::new(move |_host, _acx, _reason, link| {
        tracing::info!("sources_demo open_url: {}", link.href.as_ref());
    });

    // Upstream docs use URLs as the per-row title by default. See:
    // `repo-ref/ai-elements/apps/docs/content/components/(chatbot)/sources.mdx`.
    let sources: Arc<[ui_ai::SourceItem]> = Arc::from(vec![
        ui_ai::SourceItem::new("src-0", "https://example.com/a").url("https://example.com/a"),
        ui_ai::SourceItem::new("src-1", "https://example.com/b").url("https://example.com/b"),
        ui_ai::SourceItem::new("src-2", "https://example.com/c").url("https://example.com/c"),
    ]);

    let block = ui_ai::SourcesBlock::new(sources)
        .default_open(false)
        .on_open_url(on_open_url)
        .test_id_root("ui-ai-sources-demo-root")
        .test_id_row_prefix("ui-ai-sources-demo-row-")
        .into_element(cx);

    let title = cx.text("SourcesBlock (AI Elements): Collapsible list of assistant sources.");
    let hint = cx.text("Upstream AI Elements defaults sources to collapsed; activate to expand.");

    ui::v_flex(move |_cx| vec![title, hint, block])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N4)
        .into_element(cx)
}
// endregion: example
