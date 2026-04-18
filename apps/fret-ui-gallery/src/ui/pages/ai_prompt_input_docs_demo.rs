use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::ai as snippets;
use fret::{AppComponentCx, UiChild};

fn parts_table(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
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
                "Prompt-input-scoped Select now ships as a fully typed wrapper lane, so docs snippets do not need raw `shadcn::Select*` conversions for trigger/content/items/value.",
            ],
            [
                "PromptInputHoverCard* / PromptInputTab* / PromptInputCommand*",
                "Cursor-style header compounds now live on the typed `fret_ui_ai` surface, so the docs page no longer has to explain the official `prompt-input-cursor` example away as a missing policy layer.",
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
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let demo = snippets::prompt_input_docs_demo::render(cx);
    let cursor_style = snippets::prompt_input_cursor_demo::render(cx);
    let button_tooltips = snippets::prompt_input_tooltip_demo::render(cx);
    let features = crate::ui::doc_layout::notes_block([
        "This page now teaches the docs-shaped children lane first: transcript shell, header attachments row, body textarea, footer tools, model picker, and submit.",
        "The audit outcome remained mainly public-surface drift, not a `crates/fret-ui` mechanism bug: runtime drop handling and stable automation seams were already mostly healthy.",
        "The prompt input now exposes an upstream-like `on_submit(message)` payload lane so first-party docs no longer have to teach `PromptInputProvider` just to read submit data.",
        "The prompt action menu content now accepts docs-shaped add-attachments / add-screenshot builders without explicit per-item callback wiring, while capture policy itself stays app-owned.",
        "Cursor-style hover cards, tab groups, and inline command menus now have typed `PromptInput*` wrappers, so the docs page can render the official `prompt-input-cursor` family on the same surface instead of deferring to separate follow-up pages.",
    ])
    .test_id("ui-gallery-ai-prompt-input-docs-features");
    let parts = parts_table(cx).test_id("ui-gallery-ai-prompt-input-docs-parts");
    let notes = crate::ui::doc_layout::notes_block([
        "The remaining parity gap versus the official AI Elements Prompt Input docs is now mostly runtime-owned: file-paste attachments still need a clipboard-files contract in the platform/runner stack.",
        "The screenshot action is intentionally intent-driven in Fret today: the menu taxonomy lives in `fret-ui-ai`, while actual capture policy stays app-owned instead of leaking platform behavior into the component layer.",
        "The low-level `PromptInputRoot::into_element_with_slots(...)` adapter remains for existing demos/diagnostics, but the docs page now avoids teaching that seam as the default copyable path.",
        "Diagnostics coverage now has stable anchors for transcript messages, screenshot/add-attachment menu items, the search tooltip, the model trigger, and the cursor preview hover-card triggers.",
    ])
    .test_id("ui-gallery-ai-prompt-input-docs-notes");
    let usage_section = DocSection::build(cx, "Usage with AI SDK", demo)
        .test_id_prefix("ui-gallery-ai-prompt-input-docs-demo")
        .description("Copyable chat-like example using the docs-shaped children lane plus the upstream-like `on_submit(message)` payload surface on `fret_ui_ai`.")
        .code_rust_from_file_region(snippets::prompt_input_docs_demo::SOURCE, "example");
    let features_section = DocSection::build(cx, "Features", features)
        .description("High-signal parity notes for this Prompt Input pass.")
        .no_shell();
    let cursor_style_section = DocSection::build(cx, "Cursor Style", cursor_style)
        .description("Rust/Fret analogue of the official `prompt-input-cursor` example using typed `PromptInputHoverCard*`, `PromptInputTab*`, and `PromptInputCommand*` wrappers.")
        .test_id_prefix("ui-gallery-ai-prompt-input-cursor-demo")
        .code_rust_from_file_region(snippets::prompt_input_cursor_demo::SOURCE, "example");
    let tooltips_section = DocSection::build(cx, "Button Tooltips", button_tooltips)
        .description("Rust/Fret analogue of the official tooltip examples using `PromptInputButtonTooltip` instead of raw shadcn tooltip wiring.")
        .test_id_prefix("ui-gallery-ai-prompt-input-docs-tooltips")
        .code_rust_from_file_region(
            snippets::prompt_input_tooltip_demo::SOURCE,
            "example",
        );
    let parts_section = DocSection::build(cx, "Parts & Props", parts)
        .description(
            "Which layer owns the default authoring path and why the docs surface changed.",
        )
        .no_shell();
    let notes_section = DocSection::build(cx, "Notes", notes).description(
        "What is aligned now, what remains intentionally explicit, and what is still missing.",
    );

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Docs-aligned PromptInput coverage for AI Elements: usage first, then the official example lanes, followed by public-surface findings and the remaining parity gaps.",
        ),
        vec![
            usage_section,
            features_section,
            cursor_style_section,
            tooltips_section,
            parts_section,
            notes_section,
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
