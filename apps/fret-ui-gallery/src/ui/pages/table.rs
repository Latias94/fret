use super::super::*;

use crate::ui::doc_layout::DocSection;
use crate::ui::snippets::table as snippets;

pub(super) fn preview_table(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let footer = snippets::footer::render(cx);
    let actions = snippets::actions::render(cx);
    let rtl = snippets::rtl::render(cx);

    let page = crate::ui::doc_layout::render_doc_page(
        cx,
        Some("A responsive table component."),
        vec![
            DocSection::new("Demo", demo)
                .description("Matches the shadcn table demo structure (header + body + caption).")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-table-demo")
                .code_rust_from_file_region(include_str!("../snippets/table/demo.rs"), "example"),
            DocSection::new("Footer", footer)
                .description("Adds a <TableFooter /> section.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-table-footer")
                .code_rust_from_file_region(include_str!("../snippets/table/footer.rs"), "example"),
            DocSection::new("Actions", actions)
                .description("Uses <DropdownMenu /> as an actions column.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-table-actions")
                .code_rust_from_file_region(
                    include_str!("../snippets/table/actions.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .description("Validates right-to-left direction support.")
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-table-rtl")
                .code_rust_from_file_region(include_str!("../snippets/table/rtl.rs"), "example"),
        ],
    )
    .test_id("ui-gallery-table-root");

    vec![page]
}

