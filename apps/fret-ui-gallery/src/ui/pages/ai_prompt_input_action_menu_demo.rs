use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::AppComponentCx;

pub(super) fn preview_ai_prompt_input_action_menu_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::prompt_input_action_menu_demo::render(cx);

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "AI Elements are policy-level compositions built on top of lower-level primitives.",
        ),
        vec![
            DocSection::build(cx, "Prompt Input Action Menu", demo)
                .description(
                    "Focused footer-tools lane: `PromptInputActionMenuContent::new([]).add_attachments(...)` is the copyable docs-shaped builder surface, while callback resolution still stays deferred inside the prompt scope.",
                )
                .test_id_prefix("ui-gallery-ai-prompt-input-action-menu-demo")
                .code_rust_from_file_region(
                    snippets::prompt_input_action_menu_demo::SOURCE,
                    "example",
                ),
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
