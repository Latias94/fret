use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::navigation_menu as snippets;

pub(super) fn preview_navigation_menu(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let docs_demo = snippets::docs_demo::render(cx);
    let usage = snippets::usage::render(cx);
    let link_component = snippets::link_component::render(cx);
    let demo_with_toggle = snippets::demo::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes_block([
        "The main preview now follows the official new-york-v4 Navigation Menu demo structure (Home, Components, Docs, List, Simple, With Icon), while the container-query toggle remains a Fret-specific follow-up.",
        "`navigation_menu(cx, model, |cx| ..)` is now the default first-party root helper, while `NavigationMenu::new(model)` remains available when callers want the explicit root builder seam.",
        "`NavigationMenuRoot/List/Item/Trigger/Content/Link/Viewport/Indicator` remain the upstream-shaped lane on the same family rather than an advanced escape hatch.",
        "`NavigationMenuLink::{new, child}` now accept the same narrow single-selection bridge as the compact root lane.",
        "Top-level docs-style links stay on a contentless `NavigationMenuItem` with `href` / `target` / `rel`, and `trigger` / `trigger_child` / `trigger_children` already cover the custom trigger-child composition seam without adding a DOM-style children API.",
        "`navigation_menu_trigger_style()` intentionally stays a typed layout helper; hover/open chrome remains recipe-owned, while `w-full` / `min-w-0` / width negotiation remain caller-owned.",
        "Container query toggle is a Fret-specific extra used to audit viewport-vs-container breakpoint behavior.",
    ]);
    let notes =
        DocSection::build(cx, "Notes", notes).test_id_prefix("ui-gallery-navigation-menu-notes");
    let demo = DocSection::build(cx, "Demo", docs_demo)
        .description(
            "Official new-york-v4-style preview adapted to the UI Gallery teaching surface.",
        )
        .code_rust_from_file_region(snippets::docs_demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable `navigation_menu(...)` usage for Navigation Menu.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let link_component = DocSection::build(cx, "Link Component", link_component)
        .description(
            "Use `NavigationMenuItem::href(...)` together with `trigger_child(...)` to model the top-level Link Component outcome while keeping trigger chrome recipe-owned.",
        )
        .code_rust_from_file_region(snippets::link_component::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Navigation Menu should preserve placement and viewport alignment under RTL.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let demo_with_toggle = DocSection::build(cx, "Container Query Toggle", demo_with_toggle)
        .description("Compare viewport-driven and container-driven md breakpoint behavior.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the shadcn Navigation Menu docs path: Demo, Usage, Link Component, RTL. Container Query Toggle remains a focused Fret-specific extra.",
        ),
        vec![demo, usage, link_component, rtl, demo_with_toggle, notes],
    );

    vec![body.test_id("ui-gallery-navigation-menu").into_element(cx)]
}
