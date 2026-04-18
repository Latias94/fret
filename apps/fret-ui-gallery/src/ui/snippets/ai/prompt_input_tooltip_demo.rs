pub const SOURCE: &str = include_str!("prompt_input_tooltip_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_icons::IconId;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let simple = ui_ai::PromptInputButton::new("Search the web")
        .icon(IconId::new("lucide.globe"))
        .tooltip(ui_ai::PromptInputButtonTooltip::new("Search the web"))
        .test_id("ui-gallery-ai-prompt-input-tooltip-simple")
        .into_element(cx);

    let shortcut = ui_ai::PromptInputButton::new("Search")
        .icon(IconId::new("lucide.sparkles"))
        .tooltip(
            ui_ai::PromptInputButtonTooltip::new("Search")
                .shortcut("⌘K")
                .panel_test_id("ui-gallery-ai-prompt-input-tooltip-shortcut-panel"),
        )
        .test_id("ui-gallery-ai-prompt-input-tooltip-shortcut")
        .into_element(cx);

    let positioned = ui_ai::PromptInputButton::new("Add files")
        .icon(IconId::new("lucide.paperclip"))
        .tooltip(
            ui_ai::PromptInputButtonTooltip::new("Add files")
                .side(shadcn::TooltipSide::Bottom)
                .panel_test_id("ui-gallery-ai-prompt-input-tooltip-bottom-panel"),
        )
        .test_id("ui-gallery-ai-prompt-input-tooltip-bottom")
        .into_element(cx);

    let tools = ui_ai::PromptInputTools::empty()
        .child(simple)
        .child(shortcut)
        .child(positioned)
        .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text("Prompt Input Button Tooltips (AI Elements)"),
            cx.text(
                "Hover the toolbar actions to preview a simple tooltip, a shortcut hint, and a bottom-positioned tooltip.",
            ),
            tools,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
