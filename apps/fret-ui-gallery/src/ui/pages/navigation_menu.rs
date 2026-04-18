use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::navigation_menu as snippets;

pub(super) fn preview_navigation_menu(cx: &mut AppComponentCx<'_>) -> Vec<AnyElement> {
    let docs_demo = snippets::docs_demo::render(cx);
    let usage = snippets::usage::render(cx);
    let link_component = snippets::link_component::render(cx);
    let demo_with_toggle = snippets::demo::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "Reference stack: shadcn Navigation Menu docs, the default registry chrome, and the default docs demo.",
        "`navigation_menu(cx, model, |cx| ..)` is the compact first-party root helper, while `NavigationMenu::new(model)` remains the explicit root builder seam.",
        "`NavigationMenuRoot/List/Item/Trigger/Content/Link/Viewport/Indicator` remain the upstream-shaped lane on the same family rather than an advanced-only escape hatch.",
        "Top-level docs-style links use contentless `NavigationMenuItem` with `href` / `target` / `rel`; add `trigger_child(...)` or `trigger_children(...)` only when you need custom trigger composition.",
        "`NavigationMenuLink::{new, child}` already accept arbitrary children for rich rows/cards, so a separate DOM-flavored generic children API is not warranted here.",
        "`NavigationMenu` keeps `viewport`, `indicator`, `delay_ms`, `close_delay_ms`, `skip_delay_ms`, `on_value_change`, and `on_open_change_complete` on the public surface.",
    ]);
    let notes = doc_layout::notes_block([
        "The main preview now follows the official new-york-v4 Navigation Menu demo structure (Home, Components, Docs, List, Simple, With Icon), while the container-query toggle remains a Fret-specific follow-up.",
        "Container Query Toggle is the explicit query-axis teaching surface for this page: keep the window wide, change only the local demo card, and compare viewport-vs-container `md` behavior without mixing the two.",
        "`navigation_menu(cx, model, |cx| ..)` is now the default first-party root helper, while `NavigationMenu::new(model)` remains available when callers want the explicit root builder seam.",
        "`NavigationMenuRoot/List/Item/Trigger/Content/Link/Viewport/Indicator` remain the upstream-shaped lane on the same family rather than an advanced escape hatch.",
        "`NavigationMenu` keeps viewport placement on logical `align=start` by default; under `DirectionProvider(Rtl)` that yields the same visual outcome as the upstream RTL example's explicit `align={dir === \"rtl\" ? \"end\" : \"start\"}` wiring.",
        "`NavigationMenuLink::{new, child}` now accept the same narrow single-selection bridge as the compact root lane.",
        "Top-level docs-style links stay on a contentless `NavigationMenuItem` with `href` / `target` / `rel`, and `trigger` / `trigger_child` / `trigger_children` already cover the custom trigger-child composition seam without adding a DOM-style children API.",
        "`navigation_menu_trigger_style()` intentionally stays a typed layout helper; hover/open chrome remains recipe-owned, while `w-full` / `min-w-0` / width negotiation remain caller-owned.",
        "Container query toggle is a Fret-specific extra used to audit viewport-vs-container breakpoint behavior.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-navigation-menu-api-reference")
        .description("Public surface summary, ownership notes, and the children API conclusion.");
    let notes =
        DocSection::build(cx, "Notes", notes).test_id_prefix("ui-gallery-navigation-menu-notes");
    let demo = DocSection::build(cx, "Demo", docs_demo)
        .description(
            "Official new-york-v4-style preview adapted to the UI Gallery teaching surface.",
        )
        .test_id_prefix("ui-gallery-navigation-menu-demo")
        .code_rust_from_file_region(snippets::docs_demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Minimal copyable usage for the Fret-first root helper.")
        .test_id_prefix("ui-gallery-navigation-menu-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let link_component = DocSection::build(cx, "Link Component", link_component)
        .description(
            "Use `NavigationMenuItem::href(...)` for the top-level docs-link outcome, and add `trigger_child(...)` only when you need custom trigger content while keeping trigger chrome recipe-owned.",
        )
        .test_id_prefix("ui-gallery-navigation-menu-link-component")
        .code_rust_from_file_region(snippets::link_component::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description(
            "Navigation Menu should preserve logical start placement and viewport alignment under RTL without requiring an extra physical align prop.",
        )
        .test_id_prefix("ui-gallery-navigation-menu-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let demo_with_toggle = DocSection::build(cx, "Container Query Toggle", demo_with_toggle)
        .description(
            "Keep the window wide, shrink only the local demo card, and compare viewport-driven versus container-driven md breakpoint behavior.",
        )
        .test_id_prefix("ui-gallery-navigation-menu-container-query-toggle")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the upstream shadcn Navigation Menu docs path first: Demo, Usage, Link Component, RTL, and API Reference. Container Query Toggle remains an explicit Fret follow-up.",
        ),
        vec![
            demo,
            usage,
            link_component,
            rtl,
            api_reference,
            demo_with_toggle,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-navigation-menu").into_element(cx)]
}
