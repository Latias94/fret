pub const SOURCE: &str = include_str!("snippet_plain.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_ai as ui_ai;
use fret_ui_kit::LayoutRefinement;
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    ui_ai::Snippet::with_code(Arc::<str>::from("git clone https://github.com/user/repo"))
        .test_id("ui-ai-snippet-plain-root")
        .refine_layout(LayoutRefinement::default().max_w(Px(384.0)))
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
        })
}
// endregion: example
