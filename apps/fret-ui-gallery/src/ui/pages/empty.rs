use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::empty as snippets;
use fret::UiCx;

pub(super) fn preview_empty(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let outline = snippets::outline::render(cx);
    let background = snippets::background::render(cx);
    let avatar = snippets::avatar::render(cx);
    let avatar_group = snippets::avatar_group::render(cx);
    let input_group = snippets::input_group::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`empty(|cx| ui::children![cx; ...])` plus `empty_header`, `empty_media`, `empty_title`, `empty_description`, and `empty_content` keeps the upstream slot model intact while preserving typed builder affordances before the landing seam.",
        "`empty_media(...).variant(...)` covers the documented `default` and `icon` media variants without widening the public surface.",
        "Gallery section order now mirrors the upstream Empty docs first: `Demo`, `Usage`, the example set through `RTL`, then `API Reference`.",
        "Current recipe defaults intentionally remain aligned to the in-repo `new-york-v4` web geometry gates (`p-6 md:p-12`, `gap-6`, dashed card chrome) rather than re-translating the base source classes one-to-one in this pass.",
        "Caller-owned refinements stay explicit for preview height, background paint, inline content layout, embedded `InputGroup` width, and page/grid placement constraints.",
        "No extra generic `asChild` / `compose()` surface is needed here: the existing children-based slot API already matches the upstream composition model.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Ownership notes, source-of-truth mapping, and public-surface guidance.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("A primary empty state with actions and a secondary link.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for the `empty(...)` wrapper family and slot parts.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let outline = DocSection::build(cx, "Outline", outline)
        .description("Outlined empty state for low-emphasis surfaces.")
        .code_rust_from_file_region(snippets::outline::SOURCE, "example");
    let background = DocSection::build(cx, "Background", background)
        .description("Muted background recipe for empty states embedded in cards.")
        .code_rust_from_file_region(snippets::background::SOURCE, "example");
    let avatar = DocSection::build(cx, "Avatar", avatar)
        .description("Empty state media can be an avatar instead of an icon.")
        .code_rust_from_file_region(snippets::avatar::SOURCE, "example");
    let avatar_group = DocSection::build(cx, "Avatar Group", avatar_group)
        .description("Media can also be a composed row of avatars.")
        .code_rust_from_file_region(snippets::avatar_group::SOURCE, "example");
    let input_group = DocSection::build(cx, "InputGroup", input_group)
        .description("Empty states can include search inputs and trailing affordances.")
        .code_rust_from_file_region(snippets::input_group::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Empty layout should follow right-to-left direction context.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Empty docs path first, then records the current Fret ownership and source-of-truth choices in API Reference.",
        ),
        vec![demo, usage, outline, background, avatar, avatar_group, input_group, rtl, api_reference],
    )
    .test_id("ui-gallery-empty-component");

    vec![body.into_element(cx)]
}
