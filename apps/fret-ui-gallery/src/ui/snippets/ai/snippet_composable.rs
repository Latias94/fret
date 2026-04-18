pub const SOURCE: &str = include_str!("snippet_composable.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_icons_lucide::generated_ids::lucide::CLIPBOARD;
use fret_ui_ai as ui_ai;
use fret_ui_kit::LayoutRefinement;
use fret_ui_kit::declarative::icon as decl_icon;
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let command: Arc<str> =
        Arc::from("cargo run -p fret-ui-gallery --features gallery-dev --bin fret-ui-gallery");

    ui_ai::Snippet::new([
        ui_ai::SnippetAddon::new([ui_ai::SnippetText::new("$").into_element(cx)]).into_element(cx),
        ui_ai::SnippetInput::new(command.clone()).into_element(cx),
        ui_ai::SnippetAddon::new([ui_ai::SnippetCopyButton::new(command)
            .children([decl_icon::icon(cx, CLIPBOARD)])
            .test_id("ui-ai-snippet-composable-copy")
            .copied_marker_test_id("ui-ai-snippet-composable-copied-marker")
            .into_element(cx)])
        .align(ui_ai::SnippetAddonAlign::InlineEnd)
        .into_element(cx),
    ])
    .test_id("ui-ai-snippet-composable-root")
    .refine_layout(LayoutRefinement::default().max_w(Px(420.0)))
    .into_element(cx)
}
// endregion: example
