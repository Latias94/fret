pub const SOURCE: &str = include_str!("snippet_plain.rs");

// region: example
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let snippet =
        ui_ai::Snippet::with_code(Arc::<str>::from("git clone https://github.com/user/repo"))
            .test_id("ui-ai-snippet-plain-root")
            .into_element_with_children(cx, |cx| {
                vec![
                    ui_ai::SnippetInput::from_context().into_element(cx),
                    ui_ai::SnippetAddon::new([ui_ai::SnippetCopyButton::from_context()
                        .test_id("ui-ai-snippet-plain-copy")
                        .copied_marker_test_id("ui-ai-snippet-plain-copied-marker")
                        .into_element(cx)])
                    .align(ui_ai::SnippetAddonAlign::InlineEnd)
                    .into_element(cx),
                ]
            });

    ui::v_flex(move |cx| {
        vec![
            cx.text("Snippet without prefix"),
            cx.text("Plain inline snippet composition without a terminal prompt prefix."),
            snippet,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
