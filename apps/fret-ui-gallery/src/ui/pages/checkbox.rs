use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::checkbox as snippets;

pub(super) fn preview_checkbox(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let checked_state = snippets::checked_state::render(cx);
    let label = snippets::label::render(cx);
    let invalid_state = snippets::invalid_state::render(cx);
    let basic = snippets::basic::render(cx);
    let description_section = snippets::description::render(cx);
    let disabled_section = snippets::disabled::render(cx);
    let with_title_section = snippets::with_title::render(cx);
    let group = snippets::group::render(cx);
    let table = snippets::table::render(cx);
    let rtl_section = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Checkbox::new(Model<bool>)`, `Checkbox::new_optional(Model<Option<bool>>)` and `Checkbox::new_tristate(...)` cover the important checked and mixed-state authoring paths.",
        "Checkbox remains a leaf control surface: label, helper text, and larger click targets are composed through `FieldLabel`, `FieldDescription`, and surrounding field/layout recipes rather than a generic `compose()` API.",
        "Visual defaults such as control size, border, focus ring, and indicator chrome stay recipe-owned, while row width and form layout remain caller-owned.",
        "`Label Association` and `With Title` stay after the upstream docs path because they document Fret-specific control-registry and wrapped-field composition patterns.",
        "This page is docs/public-surface parity work, not a mechanism-layer fix.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary and ownership notes.");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Checkbox docs path first, then keeps `Label Association` and `With Title` as focused Fret follow-ups.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Single checkbox with a label.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable minimal usage for `Checkbox`.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Checked State", checked_state)
                .description("Controlled checked model and optional/indeterminate model.")
                .code_rust_from_file_region(snippets::checked_state::SOURCE, "example"),
            DocSection::new("Invalid State", invalid_state)
                .description("Invalid styling uses `aria_invalid` on the checkbox and destructive field text.")
                .code_rust_from_file_region(snippets::invalid_state::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("Field plus checkbox plus label composition.")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Description", description_section)
                .description("`FieldContent` keeps label and helper text aligned with the control.")
                .code_rust_from_file_region(snippets::description::SOURCE, "example"),
            DocSection::new("Disabled", disabled_section)
                .description("Disabled checkboxes block interaction and use muted styling.")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("Group", group)
                .description("Checkbox list pattern with per-item descriptions.")
                .code_rust_from_file_region(snippets::group::SOURCE, "example"),
            DocSection::new("Table", table)
                .description("Table selection pattern with header and row checkboxes.")
                .code_rust_from_file_region(snippets::table::SOURCE, "example"),
            DocSection::new("RTL", rtl_section)
                .description("Checkbox and label alignment under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            api_reference,
            DocSection::new("Label Association (Fret)", label)
                .description("Use `FieldLabel::for_control` plus `Checkbox::control_id` so label clicks toggle the checkbox.")
                .test_id_prefix("ui-gallery-checkbox-label")
                .code_rust_from_file_region(snippets::label::SOURCE, "example"),
            DocSection::new("With Title (Fret)", with_title_section)
                .description("`FieldLabel` can wrap a full field layout for card-style checkbox rows.")
                .code_rust_from_file_region(snippets::with_title::SOURCE, "example"),
        ],
    );

    vec![body.test_id("ui-gallery-checkbox")]
}
