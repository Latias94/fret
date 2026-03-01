use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::hover_card as snippets;

pub(super) fn preview_hover_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let trigger_delays = snippets::trigger_delays::render(cx);
    let positioning = snippets::positioning::render(cx);
    let basic = snippets::basic::render(cx);
    let sides = snippets::sides::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Hover card interactions depend on hover-intent delays, so examples include both instant and delayed scenarios.",
            "Sides and positioning are separated to make placement parity checks deterministic.",
            "RTL sample is included because side resolution can differ in right-to-left layouts.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Hover Card docs order: Demo, Trigger Delays, Positioning, Basic, Sides, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description(
                    "Basic hover card surface with a short open delay and a longer close delay.",
                )
                .code_rust_from_file_region(
                    include_str!("../snippets/hover_card/demo.rs"),
                    "example",
                ),
            DocSection::new("Trigger Delays", trigger_delays)
                .description("Compare instant vs delayed open/close behavior.")
                .code_rust_from_file_region(
                    include_str!("../snippets/hover_card/trigger_delays.rs"),
                    "example",
                ),
            DocSection::new("Positioning", positioning)
                .description("Placement is controlled by `side` and `align`.")
                .code_rust_from_file_region(
                    include_str!("../snippets/hover_card/positioning.rs"),
                    "example",
                ),
            DocSection::new("Basic", basic)
                .description("A basic hover card surface matching the upstream example.")
                .code_rust_from_file_region(
                    include_str!("../snippets/hover_card/basic.rs"),
                    "example",
                ),
            DocSection::new("Sides", sides)
                .description("Visual sweep of side placements.")
                .code_rust_from_file_region(
                    include_str!("../snippets/hover_card/sides.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .description("Hover card should respect right-to-left direction context.")
                .code_rust_from_file_region(
                    include_str!("../snippets/hover_card/rtl.rs"),
                    "example",
                ),
            DocSection::new("Notes", notes)
                .description("Implementation notes and regression guidelines."),
        ],
    );

    vec![body]
}
