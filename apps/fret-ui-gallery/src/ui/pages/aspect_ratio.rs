use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::aspect_ratio as snippets;

pub(super) fn preview_aspect_ratio(
    cx: &mut UiCx<'_>,
    wide_image: Option<Model<Option<ImageId>>>,
    tall_image: Option<Model<Option<ImageId>>>,
    square_image: Option<Model<Option<ImageId>>>,
) -> Vec<AnyElement> {
    let demo = snippets::demo::render_preview(cx, wide_image.clone());
    let usage = snippets::usage::render(cx);
    let square = snippets::square::render_preview(cx, square_image);
    let portrait = snippets::portrait::render_preview(cx, tall_image);
    let rtl = snippets::rtl::render_preview(cx, wide_image);

    let notes = doc_layout::notes(
        cx,
        [
            "Preview follows the shadcn docs flow more closely: demo first, then a minimal usage example.",
            "API reference: `ecosystem/fret-ui-shadcn/src/aspect_ratio.rs`.",
            "Prefer `AspectRatio::with_child(content).ratio(...)` when you want a prop-like builder that reads closer to shadcn/Radix usage.",
            "Use `AspectRatio` to lock geometry first, then style radius/border/background around it.",
            "For richer composition, build a single wrapper element (for example a stack with image + overlay chrome) and pass that wrapper as the child.",
            "Pick ratio by content type: 16:9 for landscape previews, 1:1 for avatars/thumbnails, 9:16 for reels or short video cards.",
            "Keep max width explicit on narrow ratios to avoid over-tall layouts in dense editor sidebars.",
            "Validate RTL and constrained width together so captions and controls remain stable during localization.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Displays content within a desired ratio. Gallery keeps the official 16:9 demo first, then adds Fret-specific square/portrait/RTL examples.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("16:9 landscape media frame (image `object-cover` style).")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Minimal usage mirroring the upstream docs shape: content first, ratio as a prop-like builder.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Square", square)
                .description("1:1 square media for avatars/thumbnails.")
                .code_rust_from_file_region(snippets::square::SOURCE, "example"),
            DocSection::new("Portrait", portrait)
                .description("9:16 portrait media for reels/short video cards.")
                .code_rust_from_file_region(snippets::portrait::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("AspectRatio should remain direction-agnostic.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Notes", notes).description("API reference pointers and usage notes."),
        ],
    );

    vec![body.test_id("ui-gallery-aspect-ratio-component")]
}
