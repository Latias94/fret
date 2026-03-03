use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::button_group as snippets;

pub(super) fn preview_button_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let orientation = snippets::orientation::render(cx);
    let size = snippets::size::render(cx);
    let nested = snippets::nested::render(cx);
    let separator = snippets::separator::render(cx);
    let split = snippets::split::render(cx);
    let text = snippets::text::render(cx);
    let flex_1 = snippets::flex_1_items::render(cx);
    let input = snippets::input::render(cx);
    let input_group = snippets::input_group::render(cx);
    let dropdown = snippets::dropdown_menu::render(cx);
    let select = snippets::button_group_select::render(cx);
    let popover = snippets::popover::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn ButtonGroup demo (new-york-v4).",
            "Fret provides `ButtonGroupSeparator` / `ButtonGroupText` to match upstream docs recipes.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the upstream intent: compose buttons + inputs/menus into a shared control group.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .test_id_prefix("ui-gallery-button-group-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Orientation", orientation)
                .test_id_prefix("ui-gallery-button-group-orientation")
                .code_rust_from_file_region(snippets::orientation::SOURCE, "example"),
            DocSection::new("Size", size)
                .test_id_prefix("ui-gallery-button-group-size")
                .code_rust_from_file_region(snippets::size::SOURCE, "example"),
            DocSection::new("Nested", nested)
                .test_id_prefix("ui-gallery-button-group-nested")
                .code_rust_from_file_region(snippets::nested::SOURCE, "example"),
            DocSection::new("Separator", separator)
                .test_id_prefix("ui-gallery-button-group-separator")
                .code_rust_from_file_region(snippets::separator::SOURCE, "example"),
            DocSection::new("Split", split)
                .test_id_prefix("ui-gallery-button-group-split")
                .code_rust_from_file_region(snippets::split::SOURCE, "example"),
            DocSection::new("Text", text)
                .test_id_prefix("ui-gallery-button-group-text")
                .code_rust_from_file_region(snippets::text::SOURCE, "example"),
            DocSection::new("Flex-1 items", flex_1)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-button-group-flex1")
                .code_rust_from_file_region(snippets::flex_1_items::SOURCE, "example"),
            DocSection::new("Input", input)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-button-group-input")
                .code_rust_from_file_region(snippets::input::SOURCE, "example"),
            DocSection::new("Input Group", input_group)
                .test_id_prefix("ui-gallery-button-group-input-group")
                .code_rust_from_file_region(snippets::input_group::SOURCE, "example"),
            DocSection::new("Dropdown Menu", dropdown)
                .test_id_prefix("ui-gallery-button-group-dropdown")
                .code_rust_from_file_region(snippets::dropdown_menu::SOURCE, "example"),
            DocSection::new("Select", select)
                .test_id_prefix("ui-gallery-button-group-select")
                .code_rust_from_file_region(snippets::button_group_select::SOURCE, "example"),
            DocSection::new("Popover", popover)
                .test_id_prefix("ui-gallery-button-group-popover")
                .code_rust_from_file_region(snippets::popover::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-button-group-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-button-group-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-button-group")]
}
