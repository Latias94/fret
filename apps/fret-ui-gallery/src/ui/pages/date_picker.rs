use super::super::*;
use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::date_picker as snippets;
use fret::UiCx;

pub(super) fn preview_date_picker(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let basic = snippets::basic::render(cx);
    let usage = snippets::usage::render(cx);
    let range = snippets::range::render(cx);
    let dob = snippets::dob::render(cx);
    let input = snippets::input::render(cx);
    let time_picker = snippets::time_picker::render(cx);
    let natural_language = snippets::natural_language::render(cx);
    let rtl = snippets::rtl::render(cx);
    let presets = snippets::presets::render(cx);
    let label = snippets::label::render(cx);
    let dropdowns = snippets::dropdowns::render(cx);
    let notes_stack = snippets::notes::render(cx);
    let notes_stack = DocSection::build(cx, "Notes", notes_stack)
        .description("Guidelines and parity notes for date picker recipes.")
        .max_w(Px(980.0))
        .code_rust_from_file_region(snippets::notes::SOURCE, "example");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("A compact date picker trigger (docs: Date Picker demo).")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for the compact `DatePicker` builder surface.")
        .test_id_prefix("ui-gallery-date-picker-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();
    let basic = DocSection::build(cx, "Basic", basic)
        .description("A basic date picker component (docs: Date Picker Basic).")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();
    let range = DocSection::build(cx, "Range Picker", range)
        .description("A date picker component for selecting a range of dates.")
        .code_rust_from_file_region(snippets::range::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();
    let dob = DocSection::build(cx, "Date of Birth", dob)
        .description(
            "A date picker component with a dropdown caption layout for month/year selection.",
        )
        .code_rust_from_file_region(snippets::dob::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();
    let input = DocSection::build(cx, "Input", input)
        .description("InputGroup + calendar button + popover calendar (docs: Date Picker Input).")
        .code_rust_from_file_region(snippets::input::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();
    let time_picker = DocSection::build(cx, "Time Picker", time_picker)
        .description("Date + time fields side-by-side (docs: Date Picker Time).")
        .code_rust_from_file_region(snippets::time_picker::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();
    let natural_language = DocSection::build(cx, "Natural Language Picker", natural_language)
        .description("This example parses natural language into a date (subset).")
        .code_rust_from_file_region(snippets::natural_language::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("All shadcn components should work under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();
    let presets = DocSection::build(cx, "With Presets", presets)
        .description(
            "Upstream shadcn registry example `date-picker-with-presets` (Select + Calendar in a popover).",
        )
        .code_rust_from_file_region(snippets::presets::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();
    let label = DocSection::build(cx, "Label Association", label)
        .description("Use `FieldLabel::for_control`, `DatePicker::control_id`, and `DatePicker::test_id_prefix` to focus the trigger and keep derived automation anchors stable.")
        .test_id_prefix("ui-gallery-date-picker-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();
    let dropdowns = DocSection::build(cx, "Extras: With Dropdowns", dropdowns)
        .description(
            "Gallery-only: desktop uses a Popover; mobile uses a Drawer. Calendar caption uses dropdown month/year selection.",
        )
        .code_rust_from_file_region(snippets::dropdowns::SOURCE, "example")
        .max_w(Px(980.0))
        .no_shell();

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the shadcn Date Picker docs flow first: Demo -> Usage -> Basic -> Range -> Date of birth -> Input -> Time -> Natural language -> RTL. `With Presets` stays as the upstream registry follow-up, while `Label Association` and `With Dropdowns` remain focused Fret/gallery extensions.",
        ),
        vec![
            demo,
            usage,
            basic,
            range,
            dob,
            input,
            time_picker,
            natural_language,
            rtl,
            presets,
            label,
            dropdowns,
            notes_stack,
        ],
    );

    vec![body.into_element(cx)]
}
