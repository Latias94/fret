pub const SOURCE: &str = include_str!("snippet_demo.rs");

// region: example
use fret_core::Px;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let snippet_code: Arc<str> = Arc::from("npx ai-elements add snippet");

    let snippet = ui_ai::Snippet::with_code(snippet_code)
        .test_id("ui-ai-snippet-root")
        .refine_layout(LayoutRefinement::default().max_w(Px(384.0)))
        .into_element_with_children(cx, |cx| {
            vec![
                ui_ai::SnippetAddon::new([ui_ai::SnippetText::new("$").into_element(cx)])
                    .into_element(cx),
                ui_ai::SnippetInput::from_context().into_element(cx),
                ui_ai::SnippetAddon::new([ui_ai::SnippetCopyButton::from_context()
                    .test_id("ui-ai-snippet-copy")
                    .copied_marker_test_id("ui-ai-snippet-copied-marker")
                    .into_element(cx)])
                .align(ui_ai::SnippetAddonAlign::InlineEnd)
                .into_element(cx),
            ]
        });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Snippet (AI Elements)"),
            cx.text("Terminal-style inline command snippet with composable add-ons and copy affordance."),
            snippet,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
