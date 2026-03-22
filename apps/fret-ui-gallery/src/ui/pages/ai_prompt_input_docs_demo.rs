use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::{UiChild, UiCx};

fn parts_table(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    crate::ui::doc_layout::text_table(
        cx,
        ["Surface", "Notes"],
        [
            [
                "PromptInput",
                "Root authoring surface now supports docs-shaped `children([...])` composition while the older slot API stays available on `PromptInputRoot`.",
            ],
            [
                "PromptInputBody + PromptInputTextarea",
                "Textarea remains root-owned internally, but the public surface now exposes an explicit body/control lane that is closer to AI Elements.",
            ],
            [
                "PromptInputHeader / PromptInputFooter / PromptInputTools",
                "Header/footer stay policy-level layout parts; spacing and block routing are recipe-owned rather than runtime-owned.",
            ],
            [
                "PromptInputSelect*",
                "Prompt-input-scoped Select naming now exists so docs snippets do not fall back to generic `shadcn::Select*` names.",
            ],
            [
                "PromptInputProvider",
                "Still the lifted-state lane today when app code needs to read text/attachments outside the root send hook.",
            ],
        ],
        false,
    )
}

pub(super) fn preview_ai_prompt_input_docs_demo(
    cx: &mut UiCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::prompt_input_docs_demo::render(cx);
    let features = crate::ui::doc_layout::notes_block([
        "This page now teaches the docs-shaped children lane first: header, body textarea, footer tools, model picker, and submit.",
        "The audit outcome was mainly public-surface drift, not a `crates/fret-ui` mechanism bug: the runtime/drop handling/test-id seams were already mostly healthy.",
        "The prompt textarea now exposes placeholder authoring instead of forcing blank controls in first-party examples.",
        "The prompt-input-scoped Select naming is now in place so examples can stay on the AI surface instead of teaching generic shadcn Select imports.",
    ])
    .test_id("ui-gallery-ai-prompt-input-docs-features");
    let parts = parts_table(cx).test_id("ui-gallery-ai-prompt-input-docs-parts");
    let notes = crate::ui::doc_layout::notes_block([
        "Remaining parity gaps versus the official AI Elements Prompt Input docs are still explicit: `PromptInputActionAddScreenshot`, file-paste attachments, and an upstream-like `onSubmit(message)` payload surface are not landed in this pass.",
        "The low-level `PromptInputRoot::into_element_with_slots(...)` adapter remains for existing demos/diagnostics, but the docs page now avoids teaching that seam as the default copyable path.",
        "When app code needs prompt text/attachments at submit time today, the practical Fret lane is still `PromptInputProvider` + lifted models rather than an upstream-style message payload callback.",
        "Existing diagnostics coverage stays valid because the docs demo keeps stable `test_id` anchors for the search toggle, tooltip panel, and model trigger.",
    ])
    .test_id("ui-gallery-ai-prompt-input-docs-notes");

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned PromptInput coverage for AI Elements: default children composition first, then the public-surface findings and the remaining parity gaps.",
        ),
        vec![
            DocSection::build(cx, "Usage with AI SDK", demo)
                .test_id_prefix("ui-gallery-ai-prompt-input-docs-demo")
                .description("Copyable prompt-input example using the docs-shaped children lane on the `fret_ui_ai` surface.")
                .code_rust_from_file_region(snippets::prompt_input_docs_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("High-signal parity notes for this Prompt Input pass.")
                .no_shell(),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Which layer owns the default authoring path and why the docs surface changed.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("What is aligned now, what remains intentionally explicit, and what is still missing."),
        ],
    );

    vec![body.into_element(cx)]
}
