use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::aspect_ratio as snippets;

pub(super) fn preview_aspect_ratio(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let square = snippets::square::render(cx);
    let portrait = snippets::portrait::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/aspect_ratio.rs`.",
            "Use `AspectRatio` to lock geometry first, then style radius/border/background around it.",
            "Pick ratio by content type: 16:9 for landscape previews, 1:1 for avatars/thumbnails, 9:16 for reels or short video cards.",
            "Keep max width explicit on narrow ratios to avoid over-tall layouts in dense editor sidebars.",
            "Validate RTL and constrained width together so captions and controls remain stable during localization.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("Preview follows shadcn Aspect Ratio docs order: Demo, Square, Portrait, RTL."),
        vec![
            DocSection::new("Demo", demo)
                .description("16:9 landscape media frame with an inline caption.")
                .code_rust_from_file_region(
                    include_str!("../snippets/aspect_ratio/demo.rs"),
                    "example",
                ),
            DocSection::new("Square", square)
                .description("1:1 square media for avatars/thumbnails.")
                .code_rust_from_file_region(
                    include_str!("../snippets/aspect_ratio/square.rs"),
                    "example",
                ),
            DocSection::new("Portrait", portrait)
                .description("9:16 portrait media for reels/short video cards.")
                .code_rust_from_file_region(
                    include_str!("../snippets/aspect_ratio/portrait.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .description("AspectRatio should remain direction-agnostic.")
                .code_rust_from_file_region(
                    include_str!("../snippets/aspect_ratio/rtl.rs"),
                    "example",
                ),
            DocSection::new("Notes", notes).description("API reference pointers and usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-aspect-ratio-component")]
}
