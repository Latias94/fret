use super::super::super::super::super::*;

pub(in crate::ui) fn preview_button_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use crate::ui::doc_layout::{self, DocSection};
    use crate::ui::snippets::button_group as snippets;

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
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/demo.rs"),
                    "example",
                ),
            DocSection::new("Orientation", orientation)
                .test_id_prefix("ui-gallery-button-group-orientation")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/orientation.rs"),
                    "example",
                ),
            DocSection::new("Size", size)
                .test_id_prefix("ui-gallery-button-group-size")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/size.rs"),
                    "example",
                ),
            DocSection::new("Nested", nested)
                .test_id_prefix("ui-gallery-button-group-nested")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/nested.rs"),
                    "example",
                ),
            DocSection::new("Separator", separator)
                .test_id_prefix("ui-gallery-button-group-separator")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/separator.rs"),
                    "example",
                ),
            DocSection::new("Split", split)
                .test_id_prefix("ui-gallery-button-group-split")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/split.rs"),
                    "example",
                ),
            DocSection::new("Text", text)
                .test_id_prefix("ui-gallery-button-group-text")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/text.rs"),
                    "example",
                ),
            DocSection::new("Flex-1 items", flex_1)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-button-group-flex1")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/flex_1_items.rs"),
                    "example",
                ),
            DocSection::new("Input", input)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-button-group-input")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/input.rs"),
                    "example",
                ),
            DocSection::new("Input Group", input_group)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-input-group")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/input_group.rs"),
                    "example",
                ),
            DocSection::new("Dropdown Menu", dropdown)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-dropdown")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/dropdown_menu.rs"),
                    "example",
                ),
            DocSection::new("Select", select)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-select")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/button_group_select.rs"),
                    "example",
                ),
            DocSection::new("Popover", popover)
                .max_w(Px(820.0))
                .test_id_prefix("ui-gallery-button-group-popover")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/popover.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-button-group-rtl")
                .code_from_file_region(
                    "rust",
                    include_str!("../../../../snippets/button_group/rtl.rs"),
                    "example",
                ),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-button-group-notes"),
        ],
    );

    vec![body]
}
