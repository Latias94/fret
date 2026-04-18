use super::super::*;

use crate::ui::doc_layout::{self, DocSection, DocTabsSizing};
use crate::ui::snippets::ai as snippets;
use fret::AppComponentCx;

pub(super) fn preview_ai_code_block_demo(
    cx: &mut AppComponentCx<'_>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    let usage = snippets::code_block_usage::render(cx);
    let language_selector = snippets::code_block_demo::render(cx);
    let about = doc_layout::notes_block([
        "Matches the official AI Elements compound-parts surface: `CodeBlock` owns code/context, while `CodeBlockHeader`, `CodeBlockTitle`, `CodeBlockFilename`, and `CodeBlockActions` stay caller-composable.",
        "Syntax highlighting, scroll behavior, and long-code rendering stay in `fret-code-view`; `fret-ui-ai` only owns the AI/product-layer composition surface.",
        "Use the gallery theme preset switcher to inspect the same example under dark mode instead of duplicating a docs-only dark preview section.",
    ]);
    let notes = doc_layout::notes_block([
        "Mechanism/defaults looked healthy here: the parity gap was the public surface and the doc page shape, not a missing `fret-ui` runtime contract.",
        "`CodeBlockLanguageSelector*` now gives a fully typed wrapper lane, so the copyable snippet no longer has to spell raw `shadcn::Select*` conversions just to wire trigger/content/items together.",
        "`into_element_with_children(...)` remains the composable children lane because provider context must be installed before parts like `CodeBlockCopyButton::from_context()` resolve inherited code state.",
    ]);
    let about_section = DocSection::build(cx, "About", about)
        .no_shell()
        .test_id_prefix("ui-gallery-ai-code-block-about")
        .description("Outcome + ownership summary for the docs-aligned Fret port.");
    let usage_section = DocSection::build(cx, "Usage", usage)
        .test_id_prefix("ui-gallery-ai-code-block-usage")
        .descriptions([
            "Matches the official basic usage example: composable header/title/actions with a context-driven copy button.",
            "Keeps the authoring surface close to upstream while still using Fret's explicit `into_element_with_children(...)` provider seam.",
        ])
        .code_rust_from_file_region(snippets::code_block_usage::SOURCE, "example");
    let language_selector_section = DocSection::build(cx, "Language Selector", language_selector)
        .tabs_sizing(DocTabsSizing::FillRemaining)
        .max_w(Px(680.0))
        .descriptions([
            "Matches the official multi-language example with a docs-aligned `CodeBlockLanguageSelector*` typed wrapper surface.",
            "Language switching stays in the policy layer while `CodeBlock` continues to delegate syntax rendering and scrolling to `fret-code-view`.",
            "Copy affordance consumes the nearest CodeBlock context instead of duplicating code props.",
            "The hidden language marker exists only for deterministic diag gating and does not affect the visible teaching surface.",
        ])
        .test_id_prefix("ui-gallery-ai-code-block-demo")
        .code_rust_from_file_region(snippets::code_block_demo::SOURCE, "example");
    let notes_section = DocSection::build(cx, "Notes", notes)
        .description("Layering and public-surface findings from the CodeBlock parity pass.");

    let body = crate::ui::doc_layout::render_doc_page_after(
        Some(
            "Preview follows the official AI Elements Code Block page after skipping Installation/Props tables: About, Usage, Language Selector, Notes. Dark-mode validation is covered by the gallery theme preset switcher rather than a duplicate preview block.",
        ),
        vec![
            about_section,
            usage_section,
            language_selector_section,
            notes_section,
        ],
        cx,
    );

    vec![body.into_element(cx)]
}
