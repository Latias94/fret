use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::sidebar as snippets;

pub(super) fn preview_sidebar(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let usage = snippets::usage::render(cx);
    let controlled = snippets::controlled::render(cx);
    let demo = snippets::demo::render(cx);
    let use_sidebar = snippets::use_sidebar::render(cx);
    let mobile = snippets::mobile::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes_block([
        "Width ownership follows upstream: use `SidebarProvider::width`, `width_icon`, and `width_mobile` first; `Sidebar` keeps theme-token fallback defaults.",
        "Keep `test_id_prefix` stable: `tools/diag-scripts/ui-gallery/sidebar/*` depend on DocSection tab trigger IDs.",
        "Mobile example forces `is_mobile(true)` for deterministic overlay + focus-restore diagnostics.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .test_id_prefix("ui-gallery-sidebar-notes");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Minimal `SidebarProvider + Sidebar + SidebarInset` composition with provider-owned width defaults.")
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example")
        .no_shell();
    let controlled = DocSection::build(cx, "SidebarProvider", controlled)
        .description(
            "Controlled open state via `SidebarProvider`; width overrides also belong here.",
        )
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-controlled")
        .code_rust_from_file_region(snippets::controlled::SOURCE, "example")
        .no_shell();
    let demo = DocSection::build(cx, "Sidebar", demo)
        .description("Desktop sidebar shell with icon collapse, groups, content, and inset layout.")
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example")
        .no_shell();
    let use_sidebar = DocSection::build(cx, "useSidebar", use_sidebar)
        .description("Read provider state and resolved widths from `use_sidebar(cx)` inside the provider subtree.")
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-use-sidebar")
        .code_rust_from_file_region(snippets::use_sidebar::SOURCE, "example")
        .no_shell();
    let mobile = DocSection::build(cx, "Extras: Mobile", mobile)
        .description("Forced-mobile sheet path kept for deterministic overlay/focus diagnostics.")
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-mobile")
        .code_rust_from_file_region(snippets::mobile::SOURCE, "example")
        .no_shell();
    let rtl = DocSection::build(cx, "Extras: RTL", rtl)
        .description("RTL composition retained as a gallery extension for parity spot-checks.")
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example")
        .no_shell();

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the shadcn Sidebar docs flow: Usage -> SidebarProvider -> Sidebar -> useSidebar. Mobile and RTL remain gallery extras.",
        ),
        vec![usage, controlled, demo, use_sidebar, mobile, rtl, notes],
    );

    vec![body.test_id("ui-gallery-sidebar")]
}
