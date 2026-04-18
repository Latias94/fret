use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::skeleton as snippets;

pub(super) fn preview_skeleton(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let avatar = snippets::avatar::render(cx);
    let card = snippets::card::render(cx);
    let text_section = snippets::text::render(cx);
    let form = snippets::form::render(cx);
    let table = snippets::table::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Skeleton::new()` matches the upstream leaf primitive path where the caller owns size and shape.",
        "`Skeleton::block()` is a Fret convenience for the common `w-full h-4` placeholder row, but it is intentionally not the default upstream path.",
        "Skeleton remains a visual placeholder primitive, so no extra generic `compose()` or composable children API is needed here; neither Radix primitives nor Base UI define a richer headless contract for this family.",
        "Default chrome (`accent`, `rounded-md`, pulse animation) stays recipe-owned, while explicit width, height, aspect ratio, and fully rounded avatar shapes remain caller-owned.",
        "Existing layout + reduced-motion tests already cover runtime parity, so this page is docs/public-surface parity work rather than a mechanism-layer fix.",
    ]);
    let notes = doc_layout::notes_block([
        "Reference stack: shadcn Skeleton docs plus the default and base/radix registry recipes.",
        "Neither Radix Primitives nor Base UI defines a dedicated Skeleton primitive, so there is no missing mechanism contract to port into `fret-ui` or `fret-ui-kit` here.",
        "Preview mirrors the shadcn Skeleton docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Examples` (`Avatar`, `Card`, `Text`, `Form`, `Table`), `RTL`, then Fret-only `API Reference` and `Notes`.",
        "Examples stay split into dedicated copyable sections so each code tab remains self-contained instead of hiding width, aspect-ratio, or rounded-shape ownership in page-local helpers.",
        "Existing `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/skeleton.rs` and `ecosystem/fret-ui-shadcn/tests/reduced_motion_continuous_frames.rs` already cover layout + motion parity; the remaining work here is docs/public-surface alignment.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-skeleton-api-reference")
        .description("Public surface summary and ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .test_id_prefix("ui-gallery-skeleton-notes")
        .description("Source axes, docs-path mapping, and why no extra children API is needed.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Avatar row with two text lines.")
        .test_id_prefix("ui-gallery-skeleton-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Leaf placeholder with caller-owned size and shape.")
        .test_id_prefix("ui-gallery-skeleton-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let avatar = DocSection::build(cx, "Avatar", avatar)
        .description("Avatar placeholder with supporting text lines.")
        .test_id_prefix("ui-gallery-skeleton-avatar")
        .code_rust_from_file_region(snippets::avatar::SOURCE, "example");
    let card = DocSection::build(cx, "Card", card)
        .description("Skeletons inside a card layout.")
        .test_id_prefix("ui-gallery-skeleton-card")
        .code_rust_from_file_region(snippets::card::SOURCE, "example");
    let text_section = DocSection::build(cx, "Text", text_section)
        .description("Multiple lines with varying widths.")
        .test_id_prefix("ui-gallery-skeleton-text")
        .code_rust_from_file_region(snippets::text::SOURCE, "example");
    let form = DocSection::build(cx, "Form", form)
        .description("Form-like blocks.")
        .test_id_prefix("ui-gallery-skeleton-form")
        .code_rust_from_file_region(snippets::form::SOURCE, "example");
    let table = DocSection::build(cx, "Table", table)
        .description("Row skeletons.")
        .test_id_prefix("ui-gallery-skeleton-table")
        .code_rust_from_file_region(snippets::table::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Direction provider should not break skeleton layout.")
        .test_id_prefix("ui-gallery-skeleton-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Use to show a placeholder while content is loading. Preview mirrors the shadcn Skeleton docs path after collapsing the top `ComponentPreview` into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Examples` (`Avatar`, `Card`, `Text`, `Form`, `Table`), `RTL`, then Fret-only `API Reference` and `Notes`.",
        ),
        vec![
            demo,
            usage,
            avatar,
            card,
            text_section,
            form,
            table,
            rtl,
            api_reference,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-skeleton").into_element(cx)]
}
