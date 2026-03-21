use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::table as snippets;

const TABLE_PAGE_INTRO: &str = "Preview mirrors the shadcn Table docs path after `Installation`: `Demo`, `Usage`, `Footer`, `Actions`, `Data Table`, `RTL`, and `API Reference`; `Children (Fret)` and `Notes` stay as focused follow-ups for the remaining public-surface guidance.";

pub(super) fn preview_table(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let footer = snippets::footer::render(cx);
    let actions = snippets::actions::render(cx);
    let rtl = snippets::rtl::render(cx);
    let children = snippets::children::render(cx);

    let data_table = doc_layout::notes_block([
        "Use `DataTable` for sorting, filtering, selection, column visibility, and pagination; keep those policy-heavy behaviors out of the base `Table` recipe.",
        "The base `Table` surface should stay the low-level shadcn leaf: responsive wrapper, row/cell chrome, and a copyable parts API.",
    ]);
    let api_reference = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/table.rs`.",
        "`Table` still owns the responsive `w-full overflow-x-auto` wrapper outcome, while page/container sizing remains caller-owned.",
        "`TableHead` and `TableCaption` now expose both compact text constructors (`table_head(...)`, `table_caption(...)`) and composable children helpers (`table_head_children(...)`, `table_caption_children(...)`).",
        "`TableCell` already accepts an arbitrary composed child root through `table_cell(...)`, so this pass did not widen it into a sibling-children collector.",
    ]);
    let notes = doc_layout::notes_block([
        "This review did not indicate a missing `fret-ui` mechanism-layer fix: the drift was in shadcn public-surface coverage and the gallery teaching surface.",
        "Horizontal overflow handling, header/body/footer ownership, and the handoff to `DataTable` remain in the right layer.",
        "Checkbox-column padding parity and mixed-height body-row vertical centering now both live in `fret-ui-shadcn` recipe defaults (`TableHead` / `TableCell`) without widening any runtime contract.",
    ]);

    let demo = DocSection::build(cx, "Demo", demo)
        .description("Matches the shadcn table demo structure (header + body + footer + caption).")
        .test_id_prefix("ui-gallery-table-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for the upstream-shaped table parts API.")
        .test_id_prefix("ui-gallery-table-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let footer = DocSection::build(cx, "Footer", footer)
        .description("Adds a `<TableFooter />` section.")
        .test_id_prefix("ui-gallery-table-footer")
        .code_rust_from_file_region(snippets::footer::SOURCE, "example");
    let actions = DocSection::build(cx, "Actions", actions)
        .description("Uses `<DropdownMenu />` as an actions column.")
        .test_id_prefix("ui-gallery-table-actions")
        .code_rust_from_file_region(snippets::actions::SOURCE, "example");
    let data_table = DocSection::build(cx, "Data Table", data_table)
        .description("Guide handoff to the TanStack-backed extension surface.")
        .no_shell()
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-table-data-table");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Validates right-to-left direction support.")
        .test_id_prefix("ui-gallery-table-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .description("Public surface summary and ownership notes.")
        .no_shell()
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-table-api-reference");
    let children = DocSection::build(cx, "Children (Fret)", children)
        .description(
            "Composable header/caption lane when the compact text constructors are too narrow.",
        )
        .test_id_prefix("ui-gallery-table-children")
        .code_rust_from_file_region(snippets::children::SOURCE, "example");
    let notes = DocSection::build(cx, "Notes", notes)
        .description("Parity diagnosis and follow-up notes.")
        .no_shell()
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-table-notes");

    let page = crate::ui::doc_layout::render_doc_page(
        cx,
        Some(TABLE_PAGE_INTRO),
        vec![
            demo,
            usage,
            footer,
            actions,
            data_table,
            rtl,
            api_reference,
            children,
            notes,
        ],
    )
    .test_id("ui-gallery-table-root");

    vec![page.into_element(cx)]
}
