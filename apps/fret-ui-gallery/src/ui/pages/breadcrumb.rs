use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::breadcrumb as snippets;

const BREADCRUMB_PAGE_INTRO: &str = "Preview mirrors the shadcn Breadcrumb docs path after skipping `Installation`: `Demo`, `Usage`, `Basic`, `Custom Separator`, `Dropdown`, `Collapsed`, `Link Component`, `RTL`, and `API Reference`. `Responsive` and `Notes` stay as focused Fret follow-ups.";

pub(super) fn preview_breadcrumb(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let custom_separator = snippets::custom_separator::render(cx);
    let dropdown = snippets::dropdown::render(cx);
    let collapsed = snippets::collapsed::render(cx);
    let link_component = snippets::link_component::render(cx);
    let responsive = snippets::responsive::render(cx);
    let rtl = snippets::rtl::render(cx);
    let api_reference = doc_layout::notes_block([
        "`shadcn::Breadcrumb` remains the compact builder for standard trails: `Breadcrumb::new().items([...])`.",
        "`shadcn::{BreadcrumbRoot, BreadcrumbList, BreadcrumbItemPart, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparatorPart, BreadcrumbEllipsis}` is the preferred docs-path composition lane for copyable examples.",
        "`BreadcrumbLink` keeps navigation typed with `href(...)`, `action(...)`, and `on_activate(...)`; `.children(|cx| [...])` is the default Fret alternative to upstream `render` / `asChild` when you need richer inline content.",
        "`BreadcrumbPage` and `BreadcrumbSeparatorPart` also expose `.children(|cx| [...])`, while `children_raw(...)` remains the explicit advanced seam for already-landed content.",
        "`breadcrumb::primitives::*` remains available as the explicit raw escape hatch for overlay-heavy or source-alignment-specific examples that need to show the low-level composition seam directly.",
    ]);

    let notes = doc_layout::notes_block([
        "API implementation: `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`.",
        "Gallery now keeps the shadcn docs-path examples (`Usage`, `Basic`, `Custom Separator`, `Collapsed`) on the curated facade lane, and reopens raw breadcrumb primitives only for advanced seams such as dropdown trigger composition, richer router-shaped inline children, responsive drawer handoff, and RTL-specific overlay alignment.",
        "Breadcrumb already exposes a compact builder plus an upstream-shaped composition lane; the main gap here was the teaching surface, not the mechanism layer or default recipe ownership.",
        "The `RTL` preview now stays closer to the upstream translated example too: Arabic labels, dot separators, and an end-aligned dropdown attached to the middle breadcrumb item.",
        "Default `BreadcrumbSeparator` chevrons already mirror toward logical `inline-end` in `fret-ui-shadcn`; the docs-aligned RTL preview overrides separators with dots because upstream does, not because the default chevron separator needs a manual RTL fix.",
        "Prefer short, task-oriented labels and keep only the current page as non-clickable text.",
        "Use separators and collapse strategy (`BreadcrumbItem::ellipsis`) to keep paths readable in narrow sidebars.",
        "Dropdown, router-link, and custom-separator samples use typed pressables/links plus `.children(|cx| [...])` for composable inline content without widening the public surface into a generic Slot/`asChild` mechanism (ADR 0115), while `children_raw(...)` remains the explicit advanced seam.",
        "The live gallery keeps usage links deterministic via `on_click(\"ui_gallery.app.open\")`, so clicking `Home` no longer launches the system browser while the code still demonstrates the semantic-link path.",
        "The root/list/current-page semantics now approximate upstream `nav/ol/li` more closely; separators and ellipsis stay presentation-only in the semantics tree.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .title_test_id("ui-gallery-breadcrumb-section-title-api-reference")
        .no_shell()
        .description("Public surface summary and ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes)
        .title_test_id("ui-gallery-breadcrumb-section-title-notes")
        .description("Implementation notes and regression guidelines.");
    let responsive = DocSection::build(cx, "Responsive", responsive)
        .title_test_id("ui-gallery-breadcrumb-section-title-responsive")
        .description("Fret-specific responsive breadcrumb: dropdown on desktop, drawer on mobile.")
        .code_rust_from_file_region(snippets::responsive::SOURCE, "example");
    let demo = DocSection::build(cx, "Demo", demo)
        .title_test_id("ui-gallery-breadcrumb-section-title-demo")
        .description("Basic breadcrumb recipe with ellipsis and current page.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-breadcrumb-section-title-usage")
        .description("Copyable curated Breadcrumb composition on the shadcn facade.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .title_test_id("ui-gallery-breadcrumb-section-title-basic")
        .description(
            "A basic breadcrumb with a home link and a components link on the curated docs lane.",
        )
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let custom_separator = DocSection::build(cx, "Custom Separator", custom_separator)
        .title_test_id("ui-gallery-breadcrumb-section-title-custom-separator")
        .description("Use `.children(|cx| [...])` on `BreadcrumbSeparatorPart` for the docs-style custom separator.")
        .code_rust_from_file_region(snippets::custom_separator::SOURCE, "example");
    let dropdown = DocSection::build(cx, "Dropdown", dropdown)
        .title_test_id("ui-gallery-breadcrumb-section-title-dropdown")
        .description("Compose a dropdown menu inside the breadcrumb trail.")
        .code_rust_from_file_region(snippets::dropdown::SOURCE, "example");
    let collapsed = DocSection::build(cx, "Collapsed", collapsed)
        .title_test_id("ui-gallery-breadcrumb-section-title-collapsed")
        .description("Use `BreadcrumbEllipsis` to show a collapsed breadcrumb state.")
        .code_rust_from_file_region(snippets::collapsed::SOURCE, "example");
    let link_component = DocSection::build(cx, "Link Component", link_component)
        .title_test_id("ui-gallery-breadcrumb-section-title-link-component")
        .description(
            "Use the typed link surface to model router-integrated breadcrumb items; `.children(|cx| [...])` is the default Fret alternative to upstream `render` / `asChild` for richer inline content.",
        )
        .code_rust_from_file_region(snippets::link_component::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .title_test_id("ui-gallery-breadcrumb-section-title-rtl")
        .description(
            "Translated upstream RTL breadcrumb with dot separators, logical inline-end trigger icon placement, and end-aligned dropdown content.",
        )
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(BREADCRUMB_PAGE_INTRO),
        vec![
            demo,
            usage,
            basic,
            custom_separator,
            dropdown,
            collapsed,
            link_component,
            rtl,
            api_reference,
            responsive,
            notes,
        ],
    )
    .test_id("ui-gallery-breadcrumb-component");

    vec![body.into_element(cx)]
}
