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
                "Still the lifted-state lane for advanced external orchestration, but the docs page now prefers the closer-to-upstream `on_submit(message)` lane first.",
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
    let button_tooltips = snippets::prompt_input_tooltip_demo::render(cx);
    let features = crate::ui::doc_layout::notes_block([
        "This page now teaches the docs-shaped children lane first: transcript shell, header attachments row, body textarea, footer tools, model picker, and submit.",
        "The audit outcome remained mainly public-surface drift, not a `crates/fret-ui` mechanism bug: runtime drop handling and stable automation seams were already mostly healthy.",
        "The prompt input now exposes an upstream-like `on_submit(message)` payload lane so first-party docs no longer have to teach `PromptInputProvider` just to read submit data.",
        "The prompt action menu content now accepts docs-shaped add-attachments / add-screenshot builders without explicit per-item callback wiring, while capture policy itself stays app-owned.",
    ])
    .test_id("ui-gallery-ai-prompt-input-docs-features");
    let cursor_style = crate::ui::doc_layout::notes_block([
        "The official AI Elements page includes a `prompt-input-cursor` example on the same docs surface.",
        "Fret currently exposes the equivalent seams across dedicated Prompt Input family pages: `Prompt Input Provider` for lifted state ownership and `Prompt Input Referenced Sources` for the block-start sources row.",
        "The remaining gap is policy-layer composition rather than runtime mechanism health: AI-specific `PromptInputHoverCard*` and `PromptInputTab*` compounds are not yet ported into `fret_ui_ai`.",
    ])
    .test_id("ui-gallery-ai-prompt-input-docs-cursor-style");
    let parts = parts_table(cx).test_id("ui-gallery-ai-prompt-input-docs-parts");
    let notes = crate::ui::doc_layout::notes_block([
        "The remaining parity gap versus the official AI Elements Prompt Input docs is now mostly runtime-owned: file-paste attachments still need a clipboard-files contract in the platform/runner stack.",
        "The screenshot action is intentionally intent-driven in Fret today: the menu taxonomy lives in `fret-ui-ai`, while actual capture policy stays app-owned instead of leaking platform behavior into the component layer.",
        "The low-level `PromptInputRoot::into_element_with_slots(...)` adapter remains for existing demos/diagnostics, but the docs page now avoids teaching that seam as the default copyable path.",
        "Diagnostics coverage now has stable anchors for transcript messages, screenshot/add-attachment menu items, the search tooltip, and the model trigger.",
    ])
    .test_id("ui-gallery-ai-prompt-input-docs-notes");

    let body = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(
            "Docs-aligned PromptInput coverage for AI Elements: usage first, then the official example lanes, followed by public-surface findings and the remaining parity gaps.",
        ),
        vec![
            DocSection::build(cx, "Usage with AI SDK", demo)
                .test_id_prefix("ui-gallery-ai-prompt-input-docs-demo")
                .description("Copyable chat-like example using the docs-shaped children lane plus the upstream-like `on_submit(message)` payload surface on `fret_ui_ai`.")
                .code_rust_from_file_region(snippets::prompt_input_docs_demo::SOURCE, "example"),
            DocSection::build(cx, "Features", features)
                .description("High-signal parity notes for this Prompt Input pass.")
                .no_shell(),
            DocSection::build(cx, "Cursor Style", cursor_style)
                .description("Current Fret mapping for the official `prompt-input-cursor` example and why it is still split across dedicated family demos.")
                .no_shell(),
            DocSection::build(cx, "Button Tooltips", button_tooltips)
                .description("Rust/Fret analogue of the official tooltip examples using `PromptInputButtonTooltip` instead of raw shadcn tooltip wiring.")
                .test_id_prefix("ui-gallery-ai-prompt-input-docs-tooltips")
                .code_rust_from_file_region(
                    snippets::prompt_input_tooltip_demo::SOURCE,
                    "example",
                ),
            DocSection::build(cx, "Parts & Props", parts)
                .description("Which layer owns the default authoring path and why the docs surface changed.")
                .no_shell(),
            DocSection::build(cx, "Notes", notes)
                .description("What is aligned now, what remains intentionally explicit, and what is still missing."),
        ],
    );

    vec![body.into_element(cx)]
}
