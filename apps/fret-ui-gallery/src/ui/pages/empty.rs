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

    let api_reference = doc_layout::notes(
        cx,
        [
            "`Empty::new([...])` plus `EmptyHeader`, `EmptyMedia`, `EmptyTitle`, `EmptyDescription`, and `EmptyContent` matches the upstream slot model directly.",
            "`EmptyMedia::variant(...)` covers the documented `default` and `icon` media variants without widening the public surface.",
            "Gallery section order now mirrors the upstream Empty docs first: `Demo`, `Usage`, the example set through `RTL`, then `API Reference`.",
            "Current recipe defaults intentionally remain aligned to the in-repo `new-york-v4` web geometry gates (`p-6 md:p-12`, `gap-6`, dashed card chrome) rather than re-translating the base source classes one-to-one in this pass.",
            "Caller-owned refinements stay explicit for preview height, background paint, inline content layout, embedded `InputGroup` width, and page/grid placement constraints.",
            "No extra generic `asChild` / `compose()` surface is needed here: the existing children-based slot API already matches the upstream composition model.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Empty docs path first, then records the current Fret ownership and source-of-truth choices in API Reference.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A primary empty state with actions and a secondary link.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for `Empty` and its slot parts.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Outline", outline)
                .description("Outlined empty state for low-emphasis surfaces.")
                .code_rust_from_file_region(snippets::outline::SOURCE, "example"),
            DocSection::new("Background", background)
                .description("Muted background recipe for empty states embedded in cards.")
                .code_rust_from_file_region(snippets::background::SOURCE, "example"),
            DocSection::new("Avatar", avatar)
                .description("Empty state media can be an avatar instead of an icon.")
                .code_rust_from_file_region(snippets::avatar::SOURCE, "example"),
            DocSection::new("Avatar Group", avatar_group)
                .description("Media can also be a composed row of avatars.")
                .code_rust_from_file_region(snippets::avatar_group::SOURCE, "example"),
            DocSection::new("InputGroup", input_group)
                .description("Empty states can include search inputs and trailing affordances.")
                .code_rust_from_file_region(snippets::input_group::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Empty layout should follow right-to-left direction context.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .description("Ownership notes, source-of-truth mapping, and public-surface guidance."),
        ],
    )
    .test_id("ui-gallery-empty-component");

    vec![body]
}
