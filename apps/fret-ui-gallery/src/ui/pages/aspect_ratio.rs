use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::aspect_ratio as snippets;

pub(super) fn preview_aspect_ratio(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let wide_image = Some(cx.local_model(|| None::<fret_core::ImageId>));
    let square_image = Some(cx.local_model(|| None::<fret_core::ImageId>));
    let tall_image = Some(cx.local_model(|| None::<fret_core::ImageId>));

    let demo = snippets::demo::render_preview(cx, wide_image.clone());
    let usage = snippets::usage::render(cx);
    let square = snippets::square::render_preview(cx, square_image);
    let portrait = snippets::portrait::render_preview(cx, tall_image);
    let rtl = snippets::rtl::render_preview(cx, wide_image);

    let api_reference = doc_layout::notes_block([
        "`AspectRatio::new(ratio, child)` stays as the minimal explicit constructor; `AspectRatio::with_child(child).ratio(...)` reads closest to the shadcn/Radix docs shape.",
        "`AspectRatio::new_children(ratio, [...])` and `AspectRatio::with_children([...]).ratio(...)` support direct image + overlay compositions without forcing an extra wrapper at the call site.",
        "`ratio` defaults to `1 / 1` on `with_child(...)` and `with_children(...)`, matching the upstream optional-ratio contract.",
        "The primitive owns ratio geometry plus the full-size absolute content host; caller-owned refinements still include max width, radius, border, background, and inner media sizing such as `w_full()` / `h_full()`.",
        "Overflow remains visible by default, matching Radix. Only opt into clipping when clipping is part of the caller-owned chrome contract.",
        "API surface owner: `ecosystem/fret-ui-shadcn/src/aspect_ratio.rs` re-exports `ecosystem/fret-ui-kit/src/primitives/aspect_ratio.rs`.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("docsec-api-reference")
        .description("Public surface summary and ownership notes.");

    let sections = vec![
        DocSection::build(cx, "Demo", demo)
            .description("16:9 landscape media frame (image `object-cover` style).")
            .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
        DocSection::build(cx, "Usage", usage)
            .description("Minimal usage mirroring the upstream docs shape: content first, ratio as a prop-like builder.")
            .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
        DocSection::build(cx, "Square", square)
            .description("1:1 square media for avatars/thumbnails.")
            .code_rust_from_file_region(snippets::square::SOURCE, "example"),
        DocSection::build(cx, "Portrait", portrait)
            .description("9:16 portrait media for reels/short video cards.")
            .code_rust_from_file_region(snippets::portrait::SOURCE, "example"),
        DocSection::build(cx, "RTL", rtl)
            .description("AspectRatio should remain direction-agnostic while Arabic caption text still reads naturally in RTL.")
            .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
        api_reference,
    ];

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Aspect Ratio docs path first: Demo, Usage, Square, Portrait, RTL, and API Reference.",
        ),
        sections,
    );

    let body = body.test_id("ui-gallery-aspect-ratio-component");
    vec![body.into_element(cx)]
}
