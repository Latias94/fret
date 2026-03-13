use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::badge as snippets;

pub(super) fn preview_badge(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let usage = snippets::usage::render(cx);
    let demo = snippets::demo::render(cx);
    let variants = snippets::variants::render(cx);
    let with_icon = snippets::icon::render(cx);
    let with_spinner = snippets::spinner::render(cx);
    let link = snippets::link::render(cx);
    let colors = snippets::colors::render(cx);
    let rtl = snippets::rtl::render(cx);
    let counts = snippets::counts::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Badge::new(label)` and `variant(...)` cover the documented `default`, `secondary`, `destructive`, `outline`, `ghost`, and `link` recipe surface.",
        "`BadgeRender::Link` is the Fret equivalent of the upstream `render` / `asChild` outcome and keeps link semantics on the badge-owned render surface without widening the mechanism layer.",
        "Icons, spinners, and custom color overrides stay on the badge recipe surface, while page-level width negotiation remains caller-owned.",
        "`Counts (Fret)` intentionally stays after the upstream path so compact numeric badge diagnostics remain stable without polluting the docs-aligned example sequence.",
        "This page is docs/public-surface parity work, not a mechanism-layer fix.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary and ownership notes.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description("Docs-aligned badge preview with the four primary variants.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal usage for `Badge`.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let variants = DocSection::build(cx, "Variants", variants)
        .description("Use the `variant` prop to change the badge variant.")
        .code_rust_from_file_region(snippets::variants::SOURCE, "example");
    let with_icon = DocSection::build(cx, "With Icon", with_icon)
        .description("Render an icon inside the badge with inline-start / inline-end placement.")
        .code_rust_from_file_region(snippets::icon::SOURCE, "example");
    let with_spinner = DocSection::build(cx, "With Spinner", with_spinner)
        .description("Render a spinner inside the badge for loading states.")
        .code_rust_from_file_region(snippets::spinner::SOURCE, "example");
    let link = DocSection::build(cx, "Link", link)
        .description(
            "Badges can be composed with link semantics through the badge-owned render surface.",
        )
        .code_rust_from_file_region(snippets::link::SOURCE, "example");
    let colors = DocSection::build(cx, "Custom Colors", colors)
        .description("Customize badge colors with explicit background and text overrides.")
        .code_rust_from_file_region(snippets::colors::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Render the badge under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let counts = DocSection::build(cx, "Counts (Fret)", counts)
        .description("Compact numeric badges kept as a focused Fret follow-up for diag coverage.")
        .code_rust_from_file_region(snippets::counts::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Badge docs path first: Demo, Usage, Variants, With Icon, With Spinner, Link, Custom Colors, RTL, and API Reference. `Counts (Fret)` stays as an explicit follow-up.",
        ),
        vec![
            demo,
            usage,
            variants,
            with_icon,
            with_spinner,
            link,
            colors,
            rtl,
            api_reference,
            counts,
        ],
    );

    vec![body.test_id("ui-gallery-badge")]
}
