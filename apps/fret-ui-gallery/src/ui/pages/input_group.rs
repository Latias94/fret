use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::input_group as snippets;

pub(super) fn preview_input_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let align_inline_start = snippets::align_inline_start::render(cx);
    let align_inline_end = snippets::align_inline_end::render(cx);
    let align_block_start = snippets::align_block_start::render(cx);
    let align_block_end = snippets::align_block_end::render(cx);
    let icon = snippets::icon::render(cx);
    let text = snippets::text::render(cx);
    let button = snippets::button::render(cx);
    let tooltip = snippets::tooltip::render(cx);
    let kbd = snippets::kbd::render(cx);
    let label = snippets::label::render(cx);
    let dropdown = snippets::dropdown::render(cx);
    let button_group = snippets::button_group::render(cx);
    let spinner = snippets::spinner::render(cx);
    let textarea = snippets::textarea::render(cx);
    let custom_input = snippets::custom_input::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/input_group.rs` (InputGroup).",
            "InputGroup API is slot based (`leading/trailing/block_start/block_end`) rather than explicit addon-align enums.",
            "`Custom Input` is expressed as composition via slots (no dedicated \"custom control\" type).",
            "Keep `ui-gallery-input-group-text-*` test IDs stable for non-overlap regression scripts.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Input Group docs order: Demo, Align (inline-start/inline-end/block-start/block-end), Icon, Text, Button, Tooltip, Textarea, Spinner, Label Association, Dropdown, Button Group, Custom Input, RTL (plus an extra Kbd section).",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("A compact input group and a textarea-style input group.")
                .test_id_prefix("ui-gallery-input-group-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Align / inline-start", align_inline_start)
                .description("Inline-start addon (leading slot).")
                .test_id_prefix("ui-gallery-input-group-align-inline-start")
                .code_rust_from_file_region(snippets::align_inline_start::SOURCE, "example"),
            DocSection::new("Align / inline-end", align_inline_end)
                .description("Inline-end addon (trailing slot).")
                .test_id_prefix("ui-gallery-input-group-align-inline-end")
                .code_rust_from_file_region(snippets::align_inline_end::SOURCE, "example"),
            DocSection::new("Align / block-start", align_block_start)
                .description("Block-start helper text with a divider.")
                .test_id_prefix("ui-gallery-input-group-align-block-start")
                .code_rust_from_file_region(snippets::align_block_start::SOURCE, "example"),
            DocSection::new("Align / block-end", align_block_end)
                .description("Textarea-style block-end footer with buttons.")
                .test_id_prefix("ui-gallery-input-group-align-block-end")
                .code_rust_from_file_region(snippets::align_block_end::SOURCE, "example"),
            DocSection::new("Icon", icon)
                .description("Icon-like leading adornment.")
                .test_id_prefix("ui-gallery-input-group-icon")
                .code_rust_from_file_region(snippets::icon::SOURCE, "example"),
            DocSection::new("Text", text)
                .description("Leading/trailing text addons should not overlap the control.")
                .test_id_prefix("ui-gallery-input-group-text")
                .code_rust_from_file_region(snippets::text::SOURCE, "example"),
            DocSection::new("Button", button)
                .description("Trailing button; set `trailing_has_button(true)` for layout.")
                .test_id_prefix("ui-gallery-input-group-button")
                .code_rust_from_file_region(snippets::button::SOURCE, "example"),
            DocSection::new("Tooltip", tooltip)
                .description("Tooltips can wrap icon buttons inside input group addons.")
                .test_id_prefix("ui-gallery-input-group-tooltip")
                .code_rust_from_file_region(snippets::tooltip::SOURCE, "example"),
            DocSection::new("Textarea", textarea)
                .description("Textarea mode with a footer row and min height.")
                .test_id_prefix("ui-gallery-input-group-textarea")
                .code_rust_from_file_region(snippets::textarea::SOURCE, "example"),
            DocSection::new("Spinner", spinner)
                .description("Leading spinner while fetching results.")
                .test_id_prefix("ui-gallery-input-group-spinner")
                .code_rust_from_file_region(snippets::spinner::SOURCE, "example"),
            DocSection::new("Label Association", label)
                .description(
                    "Use `Label::for_control` + `InputGroup::control_id` so label clicks focus the control and preserve `labelled-by` semantics.",
                )
                .test_id_prefix("ui-gallery-input-group-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example"),
            DocSection::new("Dropdown", dropdown)
                .description("Leading button with a chevron icon (wire it to a menu in app code).")
                .test_id_prefix("ui-gallery-input-group-dropdown")
                .code_rust_from_file_region(snippets::dropdown::SOURCE, "example"),
            DocSection::new("Button Group", button_group)
                .description(
                    "Wrap input groups with button groups to create prefixes and suffixes.",
                )
                .test_id_prefix("ui-gallery-input-group-button-group")
                .code_rust_from_file_region(snippets::button_group::SOURCE, "example"),
            DocSection::new("Kbd", kbd)
                .description("Kbd-like addons (layout hints for monospace pills).")
                .test_id_prefix("ui-gallery-input-group-kbd")
                .code_rust_from_file_region(snippets::kbd::SOURCE, "example"),
            DocSection::new("Custom Input", custom_input)
                .description("Custom/extended input chrome via slots.")
                .test_id_prefix("ui-gallery-input-group-custom-input")
                .code_rust_from_file_region(snippets::custom_input::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("InputGroup layout under an RTL direction provider.")
                .test_id_prefix("ui-gallery-input-group-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .test_id_prefix("ui-gallery-input-group-notes")
                .description("API reference pointers and invariants."),
        ],
    );

    vec![body.test_id("ui-gallery-input-group")]
}
