use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::skeleton as snippets;

pub(super) fn preview_skeleton(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let avatar = snippets::avatar::render(cx);
    let card = snippets::card::render(cx);
    let text_section = snippets::text::render(cx);
    let form = snippets::form::render(cx);
    let table = snippets::table::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes(
        cx,
        [
            "`Skeleton::new()` matches the upstream leaf primitive path where the caller owns size and shape.",
            "`Skeleton::block()` is a Fret convenience for the common `w-full h-4` placeholder row, but it is intentionally not the default upstream path.",
            "Skeleton remains a visual placeholder primitive, so no extra generic `compose()` API is needed here.",
            "Default chrome (`accent`, `rounded-md`, pulse animation) stays recipe-owned, while explicit width, height, aspect ratio, and fully rounded avatar shapes remain caller-owned.",
            "This page is docs/public-surface parity work, not a mechanism-layer fix.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Skeleton docs path first: Demo, Usage, Avatar, Card, Text, Form, Table, RTL, and API Reference.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Avatar row with two text lines.")
                .test_id_prefix("ui-gallery-skeleton-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .description("Basic skeleton block.")
                .test_id_prefix("ui-gallery-skeleton-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Avatar", avatar)
                .description("Avatar placeholder with supporting text lines.")
                .test_id_prefix("ui-gallery-skeleton-avatar")
                .code_rust_from_file_region(snippets::avatar::SOURCE, "example"),
            DocSection::new("Card", card)
                .description("Skeletons inside a card layout.")
                .test_id_prefix("ui-gallery-skeleton-card")
                .code_rust_from_file_region(snippets::card::SOURCE, "example"),
            DocSection::new("Text", text_section)
                .description("Multiple lines with varying widths.")
                .test_id_prefix("ui-gallery-skeleton-text")
                .code_rust_from_file_region(snippets::text::SOURCE, "example"),
            DocSection::new("Form", form)
                .description("Form-like blocks.")
                .test_id_prefix("ui-gallery-skeleton-form")
                .code_rust_from_file_region(snippets::form::SOURCE, "example"),
            DocSection::new("Table", table)
                .description("Row skeletons.")
                .test_id_prefix("ui-gallery-skeleton-table")
                .code_rust_from_file_region(snippets::table::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Direction provider should not break skeleton layout.")
                .test_id_prefix("ui-gallery-skeleton-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .test_id_prefix("ui-gallery-skeleton-api-reference")
                .description("Public surface summary and ownership notes."),
        ],
    );

    vec![body.test_id("ui-gallery-skeleton")]
}
