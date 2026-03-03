use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::checkbox as snippets;

pub(super) fn preview_checkbox(
    cx: &mut ElementContext<'_, App>,
    _model: Model<bool>,
) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let checked_state = snippets::checked_state::render(cx);
    let invalid_state = snippets::invalid_state::render(cx);
    let basic = snippets::basic::render(cx);
    let description_section = snippets::description::render(cx);
    let disabled_section = snippets::disabled::render(cx);
    let with_title_section = snippets::with_title::render(cx);
    let group = snippets::group::render(cx);
    let table = snippets::table::render(cx);
    let rtl_section = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/checkbox.rs` (Checkbox).",
            "Use Field composition (FieldLabel/FieldDescription) to keep label, helper text, and toggle target aligned.",
            "For indeterminate behavior, prefer `Checkbox::new_optional(Model<Option<bool>>)`, where `None` maps to mixed state.",
            "Table selection patterns should keep row-level and header-level states explicit; avoid hidden coupling in demos.",
            "When validating parity, test both keyboard focus ring and RTL label alignment in addition to pointer clicks.",
        ],
    )
    .test_id("ui-gallery-checkbox-notes");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Checkbox docs flow: Demo -> Checked State -> Invalid State -> Basic -> Description -> Disabled -> With Title -> Group -> Table -> RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Single checkbox with a label.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Checked State", checked_state)
                .description("Controlled checked model and optional/indeterminate model.")
                .code_rust_from_file_region(snippets::checked_state::SOURCE, "example"),
            DocSection::new("Invalid State", invalid_state)
                .description("Invalid styling uses `aria_invalid` on the checkbox and destructive label text.")
                .code_rust_from_file_region(snippets::invalid_state::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("Field + checkbox + label composition.")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Description", description_section)
                .description("FieldContent keeps label and helper text aligned with the control.")
                .code_rust_from_file_region(snippets::description::SOURCE, "example"),
            DocSection::new("Disabled", disabled_section)
                .description("Disabled checkbox should block interaction and use muted styling.")
                .code_rust_from_file_region(snippets::disabled::SOURCE, "example"),
            DocSection::new("With Title", with_title_section)
                .description("FieldLabel can wrap a full Field layout (card-style label).")
                .code_rust_from_file_region(snippets::with_title::SOURCE, "example"),
            DocSection::new("Group", group)
                .description("Checkbox group pattern with per-item descriptions.")
                .code_rust_from_file_region(snippets::group::SOURCE, "example"),
            DocSection::new("Table", table)
                .description("Table selection pattern with header and row checkboxes.")
                .code_rust_from_file_region(snippets::table::SOURCE, "example"),
            DocSection::new("RTL", rtl_section)
                .description("Checkbox + label alignment under an RTL direction provider.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API reference pointers and parity notes."),
        ],
    );

    vec![body.test_id("ui-gallery-checkbox")]
}
