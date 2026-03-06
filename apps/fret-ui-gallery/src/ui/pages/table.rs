use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::table as snippets;

pub(super) fn preview_table(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let footer = snippets::footer::render(cx);
    let actions = snippets::actions::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/table.rs`.",
            "Table already exposes the upstream-shaped parts surface (`Table`, `TableHeader`, `TableBody`, `TableFooter`, `TableRow`, `TableHead`, `TableCell`, `TableCaption`), so the main parity gap here was usage clarity rather than missing mechanism or an extra compose builder.",
            "Horizontal overflow handling lives in the root table container recipe, matching shadcn's responsive wrapper outcome.",
            "For sorting, filtering, selection, and pagination, prefer `DataTable` recipes rather than pushing policy into the base `Table` surface.",
        ],
    );

    let page = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Table docs flow: Demo -> Usage. Footer, actions, and RTL remain Fret gallery extensions around the same parts surface."),
        vec![
            DocSection::new("Demo", demo)
                .description("Matches the shadcn table demo structure (header + body + caption).")
                .test_id_prefix("ui-gallery-table-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Copyable minimal usage for the composable table parts API.")
                .test_id_prefix("ui-gallery-table-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Footer", footer)
                .description("Adds a `<TableFooter />` section.")
                .test_id_prefix("ui-gallery-table-footer")
                .code_rust_from_file_region(snippets::footer::SOURCE, "example"),
            DocSection::new("Actions", actions)
                .description("Uses `<DropdownMenu />` as an actions column.")
                .test_id_prefix("ui-gallery-table-actions")
                .code_rust_from_file_region(snippets::actions::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Validates right-to-left direction support.")
                .test_id_prefix("ui-gallery-table-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API surface and parity notes."),
        ],
    )
    .test_id("ui-gallery-table-root");

    vec![page]
}
