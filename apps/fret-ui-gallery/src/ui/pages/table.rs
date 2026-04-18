use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::table as snippets;

const TABLE_PAGE_INTRO: &str = "Preview mirrors the shadcn Table docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Footer`, `Actions`, `Data Table`, `RTL`, and `API Reference`; `Children (Fret)` and `Notes` stay as focused follow-ups for the composable-children decision and remaining public-surface guidance.";

pub(super) fn preview_table(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
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
        "Reference baseline: shadcn base Table docs.",
        "Visual/chrome baseline: the default shadcn registry table plus the demo, footer, actions, and RTL examples.",
        "`Table` still owns the responsive `w-full overflow-x-auto` wrapper outcome, while page/container sizing remains caller-owned.",
        "`table(...)`, `table_header(...)`, `table_body(...)`, `table_row(...)`, `table_head(...)`, `table_cell(...)`, and `table_caption(...)` stay the default docs-shaped lane.",
        "`TableHead` and `TableCaption` expose focused composable helpers (`table_head_children(...)` and `table_caption_children(...)`) for the upstream-shaped children pressure, while `TableCell` intentionally remains a single-child root surface.",
        "No broader generic root `children(...)` / `compose()` API is warranted here: upstream pressure is already covered by the wrapper family plus the focused head/caption children lane.",
        "Unlike overlay/listbox components, this pass did not find a separate Radix/Base UI primitive contract to port for `Table`; the remaining drift was recipe/docs-surface work rather than a missing `fret-ui` mechanism.",
        "`TableCell::text_align_end()` now aligns both plain text and a composed child root (for example an actions trigger) without an extra wrapper helper.",
        "API reference: `ecosystem/fret-ui-shadcn/src/table.rs`.",
    ]);
    let notes = doc_layout::notes_block([
        "Preview now mirrors the upstream shadcn Table docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Footer`, `Actions`, `Data Table`, `RTL`, and `API Reference`.",
        "`Children (Fret)` stays after `API Reference` as an explicit follow-up for the focused `table_head_children(...)` / `table_caption_children(...)` lane instead of widening the whole table family to a generic root children API.",
        "This pass did not identify a `fret-ui` mechanism or default-style regression: the remaining drift lived in `fret-ui-shadcn` recipe semantics and the UI Gallery teaching surface.",
        "Horizontal overflow handling, header/body/footer ownership, and the handoff to `DataTable` remain in the right layer.",
        "Checkbox-column padding parity, mixed-height body-row vertical centering, and non-text `TableCell::text_align_end()` alignment now all live in `fret-ui-shadcn` recipe defaults without widening any runtime contract.",
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
        .description(
            "Matches the docs actions story with a right-aligned `<DropdownMenu />` trigger.",
        )
        .test_id_prefix("ui-gallery-table-actions")
        .code_rust_from_file_region(snippets::actions::SOURCE, "example");
    let data_table = DocSection::build(cx, "Data Table", data_table)
        .description("Guide handoff to the TanStack-backed extension surface.")
        .no_shell()
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-table-data-table");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description(
            "Matches the docs RTL story with translated copy, footer, and full invoice rows.",
        )
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
