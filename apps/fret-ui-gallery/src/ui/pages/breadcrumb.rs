use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::breadcrumb as snippets;

pub(super) fn preview_breadcrumb(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let basic = snippets::basic::render(cx);
    let custom_separator = snippets::custom_separator::render(cx);
    let dropdown = snippets::dropdown::render(cx);
    let collapsed = snippets::collapsed::render(cx);
    let link_component = snippets::link_component::render(cx);
    let responsive = snippets::responsive::render(cx);
    let rtl = snippets::rtl::render(cx);
    let api_reference = doc_layout::notes(
        cx,
        [
            "`fret_ui_shadcn::Breadcrumb` remains the compact builder for standard trails: `Breadcrumb::new().items([...])`.",
            "For upstream parity, `breadcrumb::primitives::{Breadcrumb, BreadcrumbList, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, BreadcrumbEllipsis}` keep the shadcn-shaped composition surface.",
            "`BreadcrumbLink` and `BreadcrumbPage` now expose `.children([...])` as the explicit Fret alternative to upstream arbitrary React children, while `href(...)`, `on_click(...)`, and `on_activate(...)` keep navigation typed instead of introducing generic `Slot` / `asChild` prop merging.",
            "`BreadcrumbSeparator` keeps a narrow typed override surface through `BreadcrumbSeparatorKind` because the documented parity cases are icon/text separators rather than arbitrary slot-prop merging.",
        ],
    );

    let notes = doc_layout::notes(
        cx,
        [
            "API implementation: `ecosystem/fret-ui-shadcn/src/breadcrumb.rs`.",
            "Gallery sections now mirror the shadcn Breadcrumb docs order more directly: Demo, Usage, Basic, Custom Separator, Dropdown, Collapsed, Link Component, RTL, API Reference. `Responsive` remains a Fret-specific extra appended afterward.",
            "Breadcrumb already exposes both upstream-shaped primitives and a compact builder; the docs `Usage` section prefers primitives for parity, while the compact builder remains a Fret ergonomic shortcut.",
            "Prefer short, task-oriented labels and keep only the current page as non-clickable text.",
            "Use separators and collapse strategy (`BreadcrumbItem::ellipsis`) to keep paths readable in narrow sidebars.",
            "Dropdown and router-link samples use typed pressables/links; `.children([...])` covers composable inline content without widening the public surface into a generic Slot/`asChild` mechanism (ADR 0115).",
            "The live gallery keeps usage links deterministic via `on_click(\"ui_gallery.app.open\")`, so clicking `Home` no longer launches the system browser while the code still demonstrates the semantic-link path.",
            "The root/list/current-page semantics now approximate upstream `nav/ol/li` more closely; separators and ellipsis stay presentation-only in the semantics tree.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the shadcn Breadcrumb docs path first, then appends a Fret-specific responsive recipe for viewport-vs-drawer behavior.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .title_test_id("ui-gallery-breadcrumb-section-title-demo")
                .description("Basic breadcrumb recipe with ellipsis and current page.")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-breadcrumb-section-title-usage")
                .description("Copyable upstream-shaped primitives usage for Breadcrumb.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Basic", basic)
                .title_test_id("ui-gallery-breadcrumb-section-title-basic")
                .description("A basic breadcrumb with a home link and a components link.")
                .code_rust_from_file_region(snippets::basic::SOURCE, "example"),
            DocSection::new("Custom Separator", custom_separator)
                .title_test_id("ui-gallery-breadcrumb-section-title-custom-separator")
                .description("Use a custom separator component for parity with docs.")
                .code_rust_from_file_region(snippets::custom_separator::SOURCE, "example"),
            DocSection::new("Dropdown", dropdown)
                .title_test_id("ui-gallery-breadcrumb-section-title-dropdown")
                .description("Compose a dropdown menu inside the breadcrumb trail.")
                .code_rust_from_file_region(snippets::dropdown::SOURCE, "example"),
            DocSection::new("Collapsed", collapsed)
                .title_test_id("ui-gallery-breadcrumb-section-title-collapsed")
                .description("Use `BreadcrumbEllipsis` to show a collapsed breadcrumb state.")
                .code_rust_from_file_region(snippets::collapsed::SOURCE, "example"),
            DocSection::new("Link Component", link_component)
                .title_test_id("ui-gallery-breadcrumb-section-title-link-component")
                .description(
                    "Use the typed link surface to model router-integrated breadcrumb items; `.children([...])` is the explicit Fret alternative to upstream `render` / `asChild`.",
                )
                .code_rust_from_file_region(snippets::link_component::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .title_test_id("ui-gallery-breadcrumb-section-title-rtl")
                .description("Breadcrumb layout should follow right-to-left direction context.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .title_test_id("ui-gallery-breadcrumb-section-title-api-reference")
                .no_shell()
                .description("Public surface summary and ownership notes."),
            DocSection::new("Responsive", responsive)
                .title_test_id("ui-gallery-breadcrumb-section-title-responsive")
                .description("Fret-specific responsive breadcrumb: dropdown on desktop, drawer on mobile.")
                .code_rust_from_file_region(snippets::responsive::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .title_test_id("ui-gallery-breadcrumb-section-title-notes")
                .description("Implementation notes and regression guidelines."),
        ],
    )
    .test_id("ui-gallery-breadcrumb-component");

    vec![body]
}
