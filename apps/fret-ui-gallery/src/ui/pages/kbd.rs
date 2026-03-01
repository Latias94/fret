use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::kbd as snippets;

pub(super) fn preview_kbd(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let group = snippets::group::render(cx);
    let button = snippets::button::render(cx);
    let tooltip = snippets::tooltip::render(cx);
    let input_group = snippets::input_group::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Kbd is a fixed-height control; text placement uses bounds-as-line-box to keep the glyph visually centered.",
            "Tooltip and Input Group examples follow the upstream shadcn docs structure (v4 / New York).",
            "Each section has stable test_id for diag scripts and future gates.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Kbd docs order: Demo, Group, Button, Tooltip, Input Group, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Two shortcut display patterns (icons and chord).")
                .code_rust_from_file_region(include_str!("../snippets/kbd/demo.rs"), "example"),
            DocSection::new("Group", group)
                .description("Use `KbdGroup` to keep spacing consistent across tokens.")
                .code_rust_from_file_region(include_str!("../snippets/kbd/group.rs"), "example"),
            DocSection::new("Button", button)
                .description("kbd tokens can be composed into button labels for discoverability.")
                .code_rust_from_file_region(include_str!("../snippets/kbd/button.rs"), "example"),
            DocSection::new("Tooltip", tooltip)
                .description("Tooltips often include shortcut hints for expert users.")
                .code_rust_from_file_region(include_str!("../snippets/kbd/tooltip.rs"), "example"),
            DocSection::new("Input Group", input_group)
                .description("Trailing kbd hints can be rendered inside an input group.")
                .code_rust_from_file_region(
                    include_str!("../snippets/kbd/input_group.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .description("kbd token order should respect right-to-left direction context.")
                .code_rust_from_file_region(include_str!("../snippets/kbd/rtl.rs"), "example"),
            DocSection::new("Notes", notes)
                .description("Implementation notes and regression guidelines."),
        ],
    )
    .test_id("ui-gallery-kbd-component");

    vec![body]
}

