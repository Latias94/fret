use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::switch as snippets;

pub(super) fn preview_switch(cx: &mut ElementContext<'_, App>, model: Model<bool>) -> Vec<AnyElement> {
    let sizes = snippets::sizes::render(cx);
    let airplane_mode = snippets::airplane_mode::render(cx, model.clone());
    let bluetooth = snippets::bluetooth::render(cx);
    let label_card = snippets::label_card::render(cx);
    let rtl = snippets::rtl::render(cx);
    let extras = snippets::extras::render(cx, model.clone());

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
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-switch-sizes")
                .code_rust_from_file_region(include_str!("../snippets/switch/sizes.rs"), "example"),
            DocSection::new("Airplane Mode", airplane_mode)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-switch-airplane")
                .code_rust_from_file_region(
                    include_str!("../snippets/switch/airplane_mode.rs"),
                    "example",
                ),
            DocSection::new("Bluetooth", bluetooth)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-switch-bluetooth")
                .code_rust_from_file_region(
                    include_str!("../snippets/switch/bluetooth.rs"),
                    "example",
                ),
            DocSection::new("Label Card", label_card)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-switch-label-card")
                .code_rust_from_file_region(
                    include_str!("../snippets/switch/label_card.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-switch-rtl")
                .code_rust_from_file_region(include_str!("../snippets/switch/rtl.rs"), "example"),
            DocSection::new("Extras", extras)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-switch-extras")
                .code_rust_from_file_region(include_str!("../snippets/switch/extras.rs"), "example"),
            DocSection::new("Notes", notes).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-switch")]
}
