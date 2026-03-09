use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::hover_card as snippets;

pub(super) fn preview_hover_card(
    cx: &mut ElementContext<'_, App>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx, avatar_image);
    let usage = snippets::usage::render(cx);
    let trigger_delays = snippets::trigger_delays::render(cx);
    let positioning = snippets::positioning::render(cx);
    let basic = snippets::basic::render(cx);
    let sides = snippets::sides::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Hover card already exposes shadcn-style part names (`HoverCardTrigger`, `HoverCardContent`), and `HoverCard::new(trigger, content)` is the recipe-level composition entry point.",
            "Hover card interactions depend on hover-intent delays, so examples include both instant and delayed scenarios.",
            "Sides and positioning are separated to make placement parity checks deterministic.",
            "RTL sample is included because side resolution can differ in right-to-left layouts.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Hover Card docs order: Demo, Usage, then Fret-specific parity checks for Trigger Delays, Positioning, Basic, Sides, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description(
                    "Upstream shadcn demo composition: link trigger + 320px content (w-80) with avatar and text.",
                )
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable shadcn-style composition reference for Hover Card.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Trigger Delays", trigger_delays)
                .description("Compare instant vs delayed open/close behavior.")
                .code_rust_from_file_region(snippets::trigger_delays::SOURCE, "example"),
            DocSection::new("Positioning", positioning)
                .description("Placement is controlled by `side` and `align`.")
                .code_rust_from_file_region(snippets::positioning::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .description("Minimal hover card usage surface.")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Sides", sides)
                .description("Visual sweep of side placements.")
                .code_rust_from_file_region(snippets::sides::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Hover card should respect right-to-left direction context.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .description("Implementation notes and regression guidelines.")
                .test_id_prefix("ui-gallery-hover-card-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-hover-card")]
}
