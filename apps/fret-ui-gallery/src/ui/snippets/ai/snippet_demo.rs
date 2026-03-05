pub const SOURCE: &str = include_str!("snippet_demo.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let snippet_code: Arc<str> = Arc::from("cargo run -p fret-ui-gallery --release");

    let snippet = ui_ai::Snippet::new([
        ui_ai::SnippetText::new("$").into_element(cx),
        ui_ai::SnippetInput::new(snippet_code.clone()).into_element(cx),
        ui_ai::SnippetCopyButton::new(snippet_code)
            .test_id("ui-ai-snippet-copy")
            .copied_marker_test_id("ui-ai-snippet-copied-marker")
            .into_element(cx),
    ])
    .test_id("ui-ai-snippet-root")
    .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text("Snippet (AI Elements)"),
            cx.text("Inline command snippet with a copy button."),
            snippet,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
