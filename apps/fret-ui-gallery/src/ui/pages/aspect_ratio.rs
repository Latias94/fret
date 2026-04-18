use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::aspect_ratio as snippets;

pub(super) fn preview_aspect_ratio(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let wide_image = Some(cx.local_model(|| None::<fret_core::ImageId>));
    let square_image = Some(cx.local_model(|| None::<fret_core::ImageId>));
    let tall_image = Some(cx.local_model(|| None::<fret_core::ImageId>));

    let demo = snippets::demo::render_preview(cx, wide_image.clone());
    let usage = snippets::usage::render(cx);
    let square = snippets::square::render_preview(cx, square_image);
    let portrait = snippets::portrait::render_preview(cx, tall_image);
    let rtl = snippets::rtl::render_preview(cx, wide_image);
    let composable_children = snippets::composable_children::render(cx);

    let api_reference = doc_layout::notes_block([
        "`AspectRatio::new(ratio, child)` stays as the minimal explicit constructor; `AspectRatio::with_child(child).ratio(...)` reads closest to the shadcn/Radix docs shape.",
        "`AspectRatio::new_children(ratio, [...])` and `AspectRatio::with_children([...]).ratio(...)` support direct image + overlay compositions without forcing an extra wrapper at the call site.",
        "`ratio` defaults to `1 / 1` on `with_child(...)` and `with_children(...)`, matching the upstream optional-ratio contract.",
        "The primitive owns ratio geometry plus the full-size absolute content host; caller-owned refinements still include max width, radius, border, background, and inner media sizing such as `w_full()` / `h_full()`.",
        "Overflow remains visible by default, matching Radix. Only opt into clipping when clipping is part of the caller-owned chrome contract.",
        "Copyable snippets now use deterministic in-memory image sources so the code tabs stay complete outside UI Gallery; the lead demo preview still keeps its gallery-owned richer media path.",
        "API surface owner: `ecosystem/fret-ui-shadcn/src/aspect_ratio.rs` re-exports `ecosystem/fret-ui-kit/src/primitives/aspect_ratio.rs`.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("docsec-api-reference")
        .description("Public surface summary and ownership notes.");

    let sections = vec![
        DocSection::build(cx, "Demo", demo)
            .description("Official docs-shaped 16:9 media frame with rounded corners and muted background.")
            .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
        DocSection::build(cx, "Usage", usage)
            .description("Copyable minimal usage mirroring the upstream docs shape with a self-contained image source.")
            .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
        DocSection::build(cx, "Square", square)
            .description("Docs-aligned 1:1 square media example for avatars and thumbnails.")
            .code_rust_from_file_region(snippets::square::SOURCE, "example"),
        DocSection::build(cx, "Portrait", portrait)
            .description("Docs-aligned 9:16 portrait media example for reels and short-form cards.")
            .code_rust_from_file_region(snippets::portrait::SOURCE, "example"),
        DocSection::build(cx, "RTL", rtl)
            .description("Docs-aligned RTL figure: the ratio stays stable while the Arabic caption remains direction-aware.")
            .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
        api_reference,
        DocSection::build(cx, "Composable Children (Fret)", composable_children)
            .description("Fret-specific follow-up: use `with_children([...])` for direct image + overlay composition without an extra wrapper.")
            .test_id_prefix("ui-gallery-aspect-ratio-composable-children")
            .code_rust_from_file_region(snippets::composable_children::SOURCE, "example"),
    ];

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Aspect Ratio docs path after skipping Installation: Demo, Usage, Square, Portrait, RTL, and API Reference. `Composable Children (Fret)` stays as an explicit follow-up.",
        ),
        sections,
    );

    let body = body.test_id("ui-gallery-aspect-ratio-component");
    vec![body.into_element(cx)]
}
