use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::switch as snippets;

pub(super) fn preview_switch(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    let sizes = snippets::sizes::render(cx);
    let airplane_mode = snippets::airplane_mode::render(cx);
    let bluetooth = snippets::bluetooth::render(cx);
    let label_card = snippets::label_card::render(cx);
    let rtl = snippets::rtl::render(cx);
    let extras = snippets::extras::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows shadcn Switch demo (new-york-v4).",
            "Switch sizes are controlled via `SwitchSize` to match upstream `size=\"sm\" | \"default\"`.",
            "Use `SwitchStyle` (ADR 0220 override slots) for token-safe styling like checked track background changes.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Switch demo order: Sizes, Airplane Mode, Bluetooth, Label Card. Extras include invalid/disabled/RTL.",
        ),
        vec![
            DocSection::new("Sizes", sizes)
                .test_id_prefix("ui-gallery-switch-sizes")
                .code_rust_from_file_region(snippets::sizes::SOURCE, "example"),
            DocSection::new("Airplane Mode", airplane_mode)
                .test_id_prefix("ui-gallery-switch-airplane")
                .code_rust_from_file_region(snippets::airplane_mode::SOURCE, "example"),
            DocSection::new("Bluetooth", bluetooth)
                .test_id_prefix("ui-gallery-switch-bluetooth")
                .code_rust_from_file_region(snippets::bluetooth::SOURCE, "example"),
            DocSection::new("Label Card", label_card)
                .test_id_prefix("ui-gallery-switch-label-card")
                .code_rust_from_file_region(snippets::label_card::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .test_id_prefix("ui-gallery-switch-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Extras", extras)
                .test_id_prefix("ui-gallery-switch-extras")
                .code_rust_from_file_region(snippets::extras::SOURCE, "example"),
            DocSection::new("Notes", notes),
        ],
    );

    vec![body.test_id("ui-gallery-switch")]
}
