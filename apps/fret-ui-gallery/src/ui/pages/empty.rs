use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::empty as snippets;

pub(super) fn preview_empty(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let outline = snippets::outline::render(cx);
    let background = snippets::background::render(cx);
    let avatar = snippets::avatar::render(cx);
    let avatar_group = snippets::avatar_group::render(cx);
    let input_group = snippets::input_group::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Empty page mirrors docs example sequence so parity audit can compare section-by-section.",
            "Outline/background recipes mirror upstream: dashed borders plus a muted-to-background linear gradient (via `Paint`).",
            "Avatar and InputGroup scenarios keep state local to this page and expose stable test IDs for automation.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Empty docs order: Demo, Outline, Background, Avatar, Avatar Group, InputGroup, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A primary empty state with actions and a secondary link.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
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
            DocSection::new("Notes", notes)
                .description("Implementation notes and regression guidelines."),
        ],
    )
    .test_id("ui-gallery-empty-component");

    vec![body]
}
